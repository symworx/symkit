// Copyright (c) 2026, Nathaniel T. Berry
// Licensed under the Apache License, Version 2.0.

use std::{
    fs,
    io::{
        self,
        IsTerminal,
        Write,
    },
    path::{
        Component,
        Path,
        PathBuf,
    },
};

use crate::{
    adapters::write_selected_adapters,
    catalog::{
        Catalog,
        PruneSpec,
        Resolve,
    },
    error::{
        Error,
        Result,
    },
    fsutil::{
        merge_tree,
        merge_tree_safe,
    },
    gitignore::{
        ensure_agent_gitignore,
        ensure_migration_gitignore,
    },
    kit::is_kit_root,
};

pub struct InstallRequest {
    pub kit_root: PathBuf,
    pub target: PathBuf,
    pub catalog: Catalog,
    pub resolved: Resolve,
    pub adapters: Vec<String>,
    pub scaffold: bool,
    pub force: bool,
    pub prune: bool,
    pub yes: bool,
    pub dry_run: bool,
    pub docs: Vec<String>,
    pub docs_root: Option<String>,
    pub migration_docs: bool,
    /// Install into the user home. Skips project-only files.
    pub root: bool,
}

pub fn preview(req: &InstallRequest, docs_root: &str, root_notes: &[String]) {
    println!("Kit:      {}", req.kit_root.display());
    println!("Target:   {}", req.target.display());
    if req.root {
        println!("Mode:     root (user home)");
        for note in root_notes {
            println!("Note:     {note}");
        }
    }
    println!("Harness:  {}", req.resolved.harness);
    if let Some(alias) = &req.resolved.deprecated_harness {
        println!(
            "Note:     harness '{alias}' is deprecated; use '{}'",
            req.resolved.harness
        );
    }
    let role = if req.resolved.role.is_empty() {
        "(explicit packs)"
    } else {
        &req.resolved.role
    };
    println!("Role:     {role}");
    let packs = if req.resolved.packages.is_empty() {
        "(none)".to_string()
    } else {
        req.resolved.packages.join(" ")
    };
    println!("Packs:    {packs}");
    let skills = if req.resolved.skills.is_empty() {
        "(none)".to_string()
    } else {
        req.resolved.skills.join(" ")
    };
    println!("Skills:   {skills}");
    for name in &req.resolved.deprecated_skills {
        if let Some(alias) = req.catalog.library.skill_aliases.get(name) {
            println!("Note:     skill '{name}' is deprecated; use '{}'", alias.target);
        }
    }
    if !req.resolved.may_spawn.is_empty() {
        println!("Spawn:    {}", req.resolved.may_spawn.join(" "));
    }
    let adapters = if req.adapters.is_empty() {
        "none".to_string()
    } else {
        req.adapters.join(" ")
    };
    println!("Adapters: {adapters}");
    let scaf = if req.scaffold { "1" } else { "0" };
    if req.resolved.workspace.is_empty() {
        println!("Scaffold: {scaf}");
    } else {
        println!("Scaffold: {scaf} ({})", req.resolved.workspace);
    }
    println!(
        "AGENTS.md: append pointer (never replace); {} last pack that ships one wins",
        req.catalog.canonical.agents_overlay()
    );
    if !req.resolved.private.is_empty() {
        println!(
            "Private:  {}  (do not commit to student-facing trees)",
            req.resolved.private.join(" ")
        );
    }
    if !req.resolved.student_safe.is_empty() {
        println!("Safe:     {}", req.resolved.student_safe.join(" "));
    }
    if !req.docs.is_empty() {
        println!("Docs:     {} → {docs_root}/", req.docs.join(" "));
        for id in unique_ids(&req.docs) {
            if let Ok(tmpl) = req.catalog.doc_template(&req.resolved.harness, id) {
                println!("          {id} → {docs_root}/{}", tmpl.dest_name());
            }
        }
    }
    if req.migration_docs {
        println!("Migration-docs: 1 (dumps gitignored; README tracked)");
    }
}

pub fn confirm(req: &InstallRequest) -> Result<()> {
    if req.dry_run {
        println!();
        println!("Dry run — no files written.");
        std::process::exit(0);
    }
    if req.yes {
        return Ok(());
    }
    if !io::stdin().is_terminal() {
        return Err(Error::NeedYes);
    }
    print!("\nProceed? [y/N] ");
    io::stdout().flush()?;
    let mut ans = String::new();
    io::stdin().read_line(&mut ans)?;
    match ans.trim() {
        "y" | "Y" | "yes" | "YES" => Ok(()),
        _ => {
            println!("Aborted.");
            std::process::exit(1);
        }
    }
}

pub fn run(req: &InstallRequest) -> Result<()> {
    if is_kit_root(&req.target) {
        return Err(Error::RefuseSelfInstall);
    }
    req.catalog.check_spawn_agents(&req.kit_root)?;
    req.catalog.check_skill_aliases(&req.kit_root)?;
    req.catalog.check_harness_aliases(&req.kit_root)?;

    if let Some(existing) = read_existing_harness(&req.target, &req.catalog) {
        if existing != req.resolved.harness {
            eprintln!(
                "warn: target already has harness '{existing}'; installing '{}'.",
                req.resolved.harness
            );
            eprintln!("      v1 assumes one harness per repo. Continue only if you mean to mix.");
        }
    }

    let docs_root = if req.docs.is_empty() {
        String::new()
    } else {
        for id in unique_ids(&req.docs) {
            let tmpl = req.catalog.doc_template(&req.resolved.harness, id)?;
            let src = req.kit_root.join(&tmpl.path);
            if !src.is_file() {
                return Err(Error::DocTemplateMissing(src));
            }
            validate_rel_path(tmpl.dest_name())?;
        }
        resolve_docs_root(&req.target, req.docs_root.as_deref(), req.yes)?
    };
    let planned = planned_managed_set(req)?;
    let previous = if req.root {
        read_managed_set(&req.target, &req.catalog)
    } else {
        ManagedSet::default()
    };
    let root_notes = if req.root {
        unmanaged_collisions(req, &planned, &previous)
    } else {
        Vec::new()
    };
    preview(req, &docs_root, &root_notes);
    confirm(req)?;

    println!();
    if req.scaffold {
        scaffold_workspace(&req.kit_root, &req.target, &req.resolved.workspace, req.force)?;
        println!();
    }

    if req.prune {
        if req.root {
            let prune = PruneSpec {
                skills: names_to_prune(&previous.skills, &planned.skills),
                agents: names_to_prune(&previous.agents, &planned.agents),
                rules: names_to_prune(&previous.rules, &planned.rules),
            };
            prune_for_resolve(&req.target, &prune)?;
        } else {
            let mut prune = req.resolved.prune.clone();
            prune.skills = derived_skill_prune(
                &req.kit_root,
                &req.resolved.library_skills,
                &req.resolved.skills,
                prune.skills,
            )?;
            for name in req.catalog.library.skill_aliases.keys() {
                if !prune.skills.iter().any(|existing| existing == name) {
                    prune.skills.push(name.clone());
                }
            }
            prune.skills.sort();
            prune.skills.dedup();
            prune_for_resolve(&req.target, &prune)?;
        }
    }

    if !req.resolved.core_rules.is_empty() {
        let src = req.kit_root.join(&req.resolved.core_rules);
        if src.is_dir() {
            println!("Installing core rules");
            merge_tree(&src, &req.target.join(".agents").join("rules"))?;
            println!("  .agents/rules/ ← {}/", req.resolved.core_rules);
        }
    }

    install_library_skills(req)?;

    let mut wrote_overlay = false;
    let mut last_overlay: Option<PathBuf> = None;
    for pkg in &req.resolved.packages {
        if let Some(overlay_src) = install_package(
            &req.kit_root,
            &req.target,
            &req.catalog,
            &req.resolved.harness,
            pkg,
            req.force,
            req.root,
        )? {
            last_overlay = Some(overlay_src);
            wrote_overlay = true;
        }
    }
    if wrote_overlay && !req.root {
        ensure_agents_md_pointer(
            &req.target,
            req.catalog.canonical.agents_md(),
            req.catalog.canonical.agents_overlay(),
        )?;
    }

    if !req.docs.is_empty() {
        println!();
        install_doc_templates(req, &docs_root)?;
    }

    println!();
    write_selected_adapters(&req.target, &req.adapters, req.root)?;

    if req.root {
        if let Some(src) = &last_overlay {
            println!();
            place_root_overlay(req, src)?;
        }
    }

    if !req.root {
        println!();
        println!("Ensuring agent trees are gitignored...");
        ensure_agent_gitignore(&req.target)?;
    }

    if req.migration_docs {
        println!();
        write_migration_docs(&req.kit_root, &req.target, req.force)?;
        ensure_migration_gitignore(&req.target)?;
    }

    println!();
    write_install_state(&req.target, &req.catalog, &req.resolved, &req.adapters, Some(&planned))?;

    if !req.resolved.private.is_empty() {
        println!();
        println!("Reminder: private packs were installed — keep them out of student-facing commits.");
    }

    println!();
    if req.root {
        println!("Done. Open a new Grok, Claude, or Codex session to load the harness.");
    } else {
        println!("Done. Review changes in the target (not committed).");
        println!("  cd \"{}\" && git status", req.target.display());
    }
    Ok(())
}

fn scaffold_workspace(kit: &Path, target: &Path, rel: &str, force: bool) -> Result<()> {
    if rel.is_empty() {
        return Ok(());
    }
    let src = kit.join(rel);
    if !src.is_dir() {
        eprintln!("warn: workspace template missing: {rel}");
        return Ok(());
    }
    println!("Scaffolding workspace from {rel} ...");
    let copied = merge_tree_safe(&src, target, force)?;
    for item in copied {
        if let Some(name) = item.strip_prefix("skip ") {
            println!("  skip existing {name}");
        } else {
            println!("  scaffold {item}");
        }
    }
    Ok(())
}

fn write_migration_docs(kit: &Path, target: &Path, force: bool) -> Result<()> {
    let dest_dir = target.join("migration-docs");
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join("README.md");
    if dest.exists() && !force {
        println!("  skip existing migration-docs/README.md");
        return Ok(());
    }
    let src = kit
        .join("core")
        .join("templates")
        .join("migration-docs")
        .join("README.md");
    if src.is_file() {
        fs::copy(&src, &dest)?;
    } else {
        fs::write(
            &dest,
            "# migration-docs\n\nDrop legacy PDF or Word files here. Convert with `migrate-course`.\nDumps are gitignored except this README.\n",
        )?;
    }
    println!("  migration-docs/README.md");
    Ok(())
}

fn install_package(
    kit: &Path,
    target: &Path,
    catalog: &Catalog,
    harness: &str,
    name: &str,
    force: bool,
    root: bool,
) -> Result<Option<PathBuf>> {
    let rel = catalog.package_path(harness, name)?;
    let src = kit.join(&rel);
    if !src.is_dir() {
        return Err(Error::PackagePathMissing(src));
    }
    let safe = catalog.package_safe(harness, name)?;
    println!("Installing package: {harness}/{name}");
    if !safe {
        println!("  note: '{name}' is staff-local — do not commit to student-facing trees");
    }
    let agents_md = src.join("AGENTS.md");
    let overlay_src = if agents_md.is_file() {
        if root {
            println!("  harness overlay ← {}", agents_md.display());
        } else {
            let overlay = catalog.canonical.agents_overlay();
            fs::copy(&agents_md, target.join(overlay))?;
            println!("  {overlay} ← {}", agents_md.display());
        }
        Some(agents_md)
    } else {
        None
    };
    merge_tree(
        &src.join(".agents").join("rules"),
        &target.join(".agents").join("rules"),
    )?;
    merge_tree(
        &src.join(".agents").join("skills"),
        &target.join(".agents").join("skills"),
    )?;
    merge_tree(
        &src.join(".agents").join("agents"),
        &target.join(".agents").join("agents"),
    )?;
    let docs = src.join("docs");
    if docs.is_dir() && !root {
        // Committed course docs (AI policy, literacy) skip if present, like
        // --docs and --scaffold. .agents/ still overwrites: the kit owns those.
        let copied = merge_tree_safe(&docs, &target.join("docs"), force)?;
        println!("  docs/ ← {}/docs/", rel.display());
        for item in copied {
            if let Some(skipped) = item.strip_prefix("skip ") {
                println!("  skip existing docs/{skipped}");
            }
        }
    }
    Ok(overlay_src)
}

const AGENTS_POINTER_BEGIN: &str = "<!-- BEGIN symkit harness (do not edit this block) -->";
const AGENTS_POINTER_END: &str = "<!-- END symkit harness -->";

fn agents_pointer_block(agents_md: &str, overlay: &str) -> String {
    format!(
        "{AGENTS_POINTER_BEGIN}\nRead [`{overlay}`]({overlay}) and follow it as additional always-on project rules from the installed symkit harness. Instructions in this `{agents_md}` take precedence when they conflict.\n{AGENTS_POINTER_END}\n"
    )
}

fn ensure_agents_md_pointer(target: &Path, agents_md: &str, overlay: &str) -> Result<()> {
    let path = target.join(agents_md);
    let block = agents_pointer_block(agents_md, overlay);
    if path.is_file() {
        let existing = fs::read_to_string(&path)?;
        if existing.contains(AGENTS_POINTER_BEGIN) {
            println!("  {agents_md}: harness pointer already present");
            return Ok(());
        }
        let mut out = existing;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&block);
        fs::write(&path, out)?;
        println!("  {agents_md}: appended harness pointer → {overlay}");
    } else {
        fs::write(&path, format!("# {agents_md}\n\n{block}"))?;
        println!("  {agents_md}: wrote harness pointer → {overlay}");
    }
    Ok(())
}

fn install_library_skills(req: &InstallRequest) -> Result<()> {
    if req.resolved.skills.is_empty() {
        return Ok(());
    }
    println!("Installing skills");
    for name in &req.resolved.skills {
        let rel = req.catalog.skill_path(name);
        let src = req.kit_root.join(&rel);
        if !src.is_dir() {
            return Err(Error::UnknownSkill {
                name: name.clone(),
                path: src,
            });
        }
        merge_tree(&src, &req.target.join(".agents").join("skills").join(name))?;
        println!("  .agents/skills/{name}/ ← {}/", rel.display());
    }
    Ok(())
}

fn derived_skill_prune(kit: &Path, library_rel: &str, keep: &[String], extra: Vec<String>) -> Result<Vec<String>> {
    let mut names = extra;
    if library_rel.is_empty() {
        names.sort();
        names.dedup();
        return Ok(names);
    }
    let dir = kit.join(library_rel);
    if dir.is_dir() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            if !keep.iter().any(|k| k == &name) {
                names.push(name);
            }
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

fn prune_for_resolve(target: &Path, prune: &PruneSpec) -> Result<()> {
    if prune.skills.is_empty() && prune.agents.is_empty() && prune.rules.is_empty() {
        return Ok(());
    }
    let mut any = false;
    for name in &prune.skills {
        any |= prune_named(target, "skills", name, any)?;
    }
    for name in &prune.agents {
        any |= prune_named(target, "agents", name, any)?;
    }
    for name in &prune.rules {
        any |= prune_named(target, "rules", name, any)?;
    }
    if any {
        println!();
    }
    Ok(())
}

fn prune_named(target: &Path, kind: &str, name: &str, header_done: bool) -> Result<bool> {
    if name.is_empty() {
        return Ok(false);
    }
    let mut hit = false;
    let vendors: &[&str] = match kind {
        "skills" => &[".agents", ".grok", ".claude", ".codex"],
        _ => &[".agents", ".grok", ".claude"],
    };
    for vendor in vendors {
        let rel = match kind {
            "skills" => format!("{vendor}/skills/{name}"),
            "agents" => format!("{vendor}/agents/{name}.md"),
            "rules" => format!("{vendor}/rules/{name}.md"),
            _ => continue,
        };
        let p = target.join(&rel);
        if p.exists() {
            if !header_done && !hit {
                println!("Pruning leftover role paths...");
            }
            if p.is_dir() {
                fs::remove_dir_all(&p)?;
            } else {
                fs::remove_file(&p)?;
            }
            println!("  prune {rel}");
            hit = true;
        }
    }
    Ok(hit)
}

fn write_install_state(
    target: &Path,
    catalog: &Catalog,
    resolved: &Resolve,
    adapters: &[String],
    managed: Option<&ManagedSet>,
) -> Result<()> {
    let rel = state_rel(catalog);
    let path = target.join(&rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut body = format!(
        "harness: {}\nrole: {}\npackages: [{}]\nskills: [{}]\nadapters: [{}]\n",
        resolved.harness,
        resolved.role,
        resolved.packages.join(" "),
        resolved.skills.join(" "),
        adapters.join(" ")
    );
    if let Some(managed) = managed {
        body.push_str(&format!(
            "managed_skills: [{}]\nmanaged_agents: [{}]\nmanaged_rules: [{}]\n",
            managed.skills.join(" "),
            managed.agents.join(" "),
            managed.rules.join(" ")
        ));
    }
    fs::write(&path, body)?;
    println!("  state → {rel}");
    Ok(())
}

fn state_rel(catalog: &Catalog) -> String {
    if catalog.canonical.state.is_empty() {
        ".symkit/state.yaml".to_string()
    } else {
        catalog.canonical.state.clone()
    }
}

fn read_existing_harness(target: &Path, catalog: &Catalog) -> Option<String> {
    let text = fs::read_to_string(target.join(state_rel(catalog))).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("harness:") {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(catalog.canonical_harness(v).to_string());
            }
        }
    }
    None
}

const RESERVED_DOCS_ROOT: &[&str] = &[".symkit", ".agents", ".grok", ".claude", ".codex"];

fn unique_ids(ids: &[String]) -> Vec<&str> {
    let mut out = Vec::new();
    for id in ids {
        if !id.is_empty() && !out.contains(&id.as_str()) {
            out.push(id.as_str());
        }
    }
    out
}

fn validate_rel_path(raw: &str) -> Result<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(Error::InvalidDocsRoot(raw.to_string()));
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(Error::InvalidDocsRoot(raw.to_string()));
    }
    for comp in path.components() {
        match comp {
            Component::Normal(name) => {
                let name = name.to_string_lossy();
                if RESERVED_DOCS_ROOT.contains(&name.as_ref()) {
                    return Err(Error::InvalidDocsRoot(raw.to_string()));
                }
            }
            _ => return Err(Error::InvalidDocsRoot(raw.to_string())),
        }
    }
    Ok(trimmed.to_string())
}

#[derive(Debug, PartialEq, Eq)]
enum DocsRootGuess {
    Docs,
    Documents,
    Both,
    Neither,
}

fn detect_docs_root(target: &Path) -> DocsRootGuess {
    let docs = target.join("docs").is_dir();
    let documents = target.join("documents").is_dir();
    match (docs, documents) {
        (true, false) => DocsRootGuess::Docs,
        (false, true) => DocsRootGuess::Documents,
        (true, true) => DocsRootGuess::Both,
        (false, false) => DocsRootGuess::Neither,
    }
}

pub fn resolve_docs_root(target: &Path, explicit: Option<&str>, yes: bool) -> Result<String> {
    if let Some(raw) = explicit {
        return validate_rel_path(raw);
    }
    match detect_docs_root(target) {
        DocsRootGuess::Docs | DocsRootGuess::Neither => Ok("docs".into()),
        DocsRootGuess::Documents => Ok("documents".into()),
        DocsRootGuess::Both => {
            if yes || !io::stdin().is_terminal() {
                return Err(Error::AmbiguousDocsRoot);
            }
            print!("Both docs/ and documents/ exist. Docs root [docs]: ");
            io::stdout().flush()?;
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            let t = line.trim();
            if t.is_empty() {
                Ok("docs".into())
            } else {
                validate_rel_path(t)
            }
        }
    }
}

fn install_doc_templates(req: &InstallRequest, docs_root: &str) -> Result<()> {
    println!("Copying doc templates into {docs_root}/ ...");
    for id in unique_ids(&req.docs) {
        let tmpl = req.catalog.doc_template(&req.resolved.harness, id)?;
        let src = req.kit_root.join(&tmpl.path);
        if !src.is_file() {
            return Err(Error::DocTemplateMissing(src));
        }
        let dest_name = tmpl.dest_name();
        validate_rel_path(dest_name)?;
        let dest = req.target.join(docs_root).join(dest_name);
        let rel = format!("{docs_root}/{dest_name}");
        if dest.exists() && !req.force {
            println!("  skip existing {rel}");
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&src, &dest)?;
        println!("  {id} → {rel}");
    }
    Ok(())
}

pub fn adapt_only(target: &Path, adapters: &[String]) -> Result<()> {
    println!("Target:   {}", target.display());
    write_selected_adapters(target, adapters, false)?;
    ensure_agent_gitignore(target)?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct ManagedSet {
    skills: Vec<String>,
    agents: Vec<String>,
    rules: Vec<String>,
}

fn names_to_prune(previous: &[String], planned: &[String]) -> Vec<String> {
    let mut out: Vec<String> = previous
        .iter()
        .filter(|name| !planned.iter().any(|kept| kept == *name))
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

fn planned_managed_set(req: &InstallRequest) -> Result<ManagedSet> {
    let mut skills = req.resolved.skills.clone();
    let mut agents = Vec::new();
    let mut rules = Vec::new();
    if !req.resolved.core_rules.is_empty() {
        collect_md_stems(&req.kit_root.join(&req.resolved.core_rules), &mut rules);
    }
    for pkg in &req.resolved.packages {
        let rel = req.catalog.package_path(&req.resolved.harness, pkg)?;
        let src = req.kit_root.join(&rel).join(".agents");
        collect_dir_names(&src.join("skills"), &mut skills);
        collect_md_stems(&src.join("agents"), &mut agents);
        collect_md_stems(&src.join("rules"), &mut rules);
    }
    sort_dedup(&mut skills);
    sort_dedup(&mut agents);
    sort_dedup(&mut rules);
    Ok(ManagedSet { skills, agents, rules })
}

fn sort_dedup(names: &mut Vec<String>) {
    names.sort();
    names.dedup();
}

fn collect_dir_names(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            out.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
}

fn collect_md_stems(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(stem) = name.strip_suffix(".md") else {
            continue;
        };
        if stem.is_empty() || stem.eq_ignore_ascii_case("readme") {
            continue;
        }
        out.push(stem.to_string());
    }
}

fn read_managed_set(target: &Path, catalog: &Catalog) -> ManagedSet {
    let text = fs::read_to_string(target.join(state_rel(catalog))).unwrap_or_default();
    ManagedSet {
        skills: parse_bracket_list(&text, "managed_skills:"),
        agents: parse_bracket_list(&text, "managed_agents:"),
        rules: parse_bracket_list(&text, "managed_rules:"),
    }
}

fn parse_bracket_list(text: &str, key: &str) -> Vec<String> {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim().trim_start_matches('[').trim_end_matches(']');
        return rest
            .split_whitespace()
            .map(str::to_string)
            .filter(|name| !name.is_empty())
            .collect();
    }
    Vec::new()
}

fn unmanaged_collisions(req: &InstallRequest, planned: &ManagedSet, previous: &ManagedSet) -> Vec<String> {
    let mut notes = Vec::new();
    push_collisions(req, "skills", &planned.skills, &previous.skills, true, &mut notes);
    push_collisions(req, "agents", &planned.agents, &previous.agents, false, &mut notes);
    push_collisions(req, "rules", &planned.rules, &previous.rules, false, &mut notes);
    if req.adapters.iter().any(|name| name == "codex") {
        let path = req.target.join(".codex").join("AGENTS.override.md");
        if path.is_file() {
            notes.push(format!(
                "{} exists; Codex reads it instead of AGENTS.md",
                path.display()
            ));
        }
    }
    notes
}

fn push_collisions(
    req: &InstallRequest,
    kind: &str,
    planned: &[String],
    previous: &[String],
    is_dir: bool,
    notes: &mut Vec<String>,
) {
    for name in planned {
        if previous.iter().any(|kept| kept == name) {
            continue;
        }
        for vendor in vendor_roots(kind, &req.adapters) {
            let path = match kind {
                "skills" => req.target.join(vendor).join("skills").join(name),
                "agents" => req.target.join(vendor).join("agents").join(format!("{name}.md")),
                "rules" => req.target.join(vendor).join("rules").join(format!("{name}.md")),
                _ => continue,
            };
            let exists = if is_dir { path.is_dir() } else { path.is_file() };
            if exists {
                notes.push(format!(
                    "overwrite {} (not previously managed by symkit)",
                    path.display()
                ));
            }
        }
    }
}

fn vendor_roots<'a>(kind: &str, adapters: &'a [String]) -> Vec<&'a str> {
    let mut roots = vec![".agents"];
    if adapters.iter().any(|name| name == "grok") {
        roots.push(".grok");
    }
    if adapters.iter().any(|name| name == "claude") {
        roots.push(".claude");
    }
    if kind == "skills" && adapters.iter().any(|name| name == "codex") {
        roots.push(".codex");
    }
    roots
}

fn place_root_overlay(req: &InstallRequest, src: &Path) -> Result<()> {
    let name = req.catalog.canonical.agents_overlay();
    if req.adapters.iter().any(|adapter| adapter == "grok") {
        let dest = req.target.join(".grok").join("rules").join(name);
        copy_file(src, &dest)?;
        println!("  .grok/rules/{name} ← harness overlay");
    }
    if req.adapters.iter().any(|adapter| adapter == "claude") {
        let dest = req.target.join(".claude").join(name);
        copy_file(src, &dest)?;
        println!("  .claude/{name} ← harness overlay");
    }
    if req.adapters.iter().any(|adapter| adapter == "codex") {
        let dest = req.target.join(".codex").join(name);
        copy_file(src, &dest)?;
        println!("  .codex/{name} ← harness overlay");
    }
    crate::adapters::write_root_instruction_pointers(&req.target, &req.adapters, name)?;
    Ok(())
}

fn copy_file(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn pointer_creates_agents_md() {
        let dir = tempdir().unwrap();
        ensure_agents_md_pointer(dir.path(), "AGENTS.md", "AGENTS-SYMKIT.md").unwrap();
        let text = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
        assert!(text.contains(AGENTS_POINTER_BEGIN));
        assert!(text.contains("AGENTS-SYMKIT.md"));
        assert!(text.contains("take precedence"));
    }

    #[test]
    fn pointer_appends_without_clobber() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, "repo-specific keep\n").unwrap();
        ensure_agents_md_pointer(dir.path(), "AGENTS.md", "AGENTS-SYMKIT.md").unwrap();
        ensure_agents_md_pointer(dir.path(), "AGENTS.md", "AGENTS-SYMKIT.md").unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("repo-specific keep"));
        assert_eq!(text.matches(AGENTS_POINTER_BEGIN).count(), 1);
        assert!(text.contains("AGENTS-SYMKIT.md"));
    }

    #[test]
    fn detect_docs_vs_documents() {
        let dir = tempdir().unwrap();
        assert_eq!(detect_docs_root(dir.path()), DocsRootGuess::Neither);
        fs::create_dir(dir.path().join("docs")).unwrap();
        assert_eq!(detect_docs_root(dir.path()), DocsRootGuess::Docs);
        fs::create_dir(dir.path().join("documents")).unwrap();
        assert_eq!(detect_docs_root(dir.path()), DocsRootGuess::Both);
    }

    #[test]
    fn resolve_prefers_documents_when_only_that_exists() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("documents")).unwrap();
        let root = resolve_docs_root(dir.path(), None, true).unwrap();
        assert_eq!(root, "documents");
    }

    #[test]
    fn resolve_defaults_to_docs_when_neither_exists() {
        let dir = tempdir().unwrap();
        let root = resolve_docs_root(dir.path(), None, true).unwrap();
        assert_eq!(root, "docs");
    }

    #[test]
    fn resolve_both_errors_without_explicit_root() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("docs")).unwrap();
        fs::create_dir(dir.path().join("documents")).unwrap();
        let err = resolve_docs_root(dir.path(), None, true).unwrap_err();
        assert!(matches!(err, Error::AmbiguousDocsRoot));
        let root = resolve_docs_root(dir.path(), Some("documents"), true).unwrap();
        assert_eq!(root, "documents");
    }

    #[test]
    fn validate_docs_root_rejects_reserved_and_parent() {
        assert!(validate_rel_path("docs").is_ok());
        assert!(validate_rel_path("documents").is_ok());
        assert!(validate_rel_path("slos.md").is_ok());
        assert!(validate_rel_path(".symkit").is_err());
        assert!(validate_rel_path("../docs").is_err());
        assert!(validate_rel_path("/tmp/docs").is_err());
        assert!(validate_rel_path("").is_err());
    }

    #[test]
    fn managed_difference_drops_kept_names() {
        let previous = vec!["old-skill".into(), "write-tests".into()];
        let planned = vec!["write-tests".into(), "write-docs".into()];
        assert_eq!(names_to_prune(&previous, &planned), vec!["old-skill".to_string()]);
    }

    #[test]
    fn parse_managed_list_reads_brackets() {
        let text = "harness: engineering\nmanaged_skills: [old-skill write-tests]\n";
        assert_eq!(
            parse_bracket_list(text, "managed_skills:"),
            vec!["old-skill".to_string(), "write-tests".to_string()]
        );
        assert!(parse_bracket_list(text, "managed_agents:").is_empty());
    }

    #[test]
    fn root_prune_removes_managed_skill_and_leaves_unmanaged() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".grok/skills/qc-things")).unwrap();
        fs::write(dir.path().join(".grok/skills/qc-things/SKILL.md"), "keep\n").unwrap();
        fs::create_dir_all(dir.path().join(".agents/skills/old-skill")).unwrap();
        fs::write(dir.path().join(".agents/skills/old-skill/SKILL.md"), "old\n").unwrap();
        fs::create_dir_all(dir.path().join(".grok/skills/old-skill")).unwrap();
        fs::write(dir.path().join(".grok/skills/old-skill/SKILL.md"), "old\n").unwrap();

        let prune = PruneSpec {
            skills: names_to_prune(&["old-skill".into()], &[]),
            ..PruneSpec::default()
        };
        prune_for_resolve(dir.path(), &prune).unwrap();

        assert!(dir.path().join(".grok/skills/qc-things/SKILL.md").is_file());
        assert!(!dir.path().join(".grok/skills/old-skill").exists());
        assert!(!dir.path().join(".agents/skills/old-skill").exists());
    }
}
