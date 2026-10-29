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
        Path,
        PathBuf,
    },
};

use crate::{
    catalog::Catalog,
    error::{
        Error,
        Result,
    },
    gitignore::remove_agent_gitignore_block,
    kit::is_kit_root,
};

const POINTER_BEGIN: &str = "<!-- BEGIN symkit harness (do not edit this block) -->";
const POINTER_END: &str = "<!-- END symkit harness -->";
const CLAUDE_ADAPTER_MARK: &str = "Adapter for Claude Code";

pub struct UninstallRequest {
    pub target: PathBuf,
    pub catalog: Catalog,
    pub yes: bool,
    pub dry_run: bool,
    /// Home install. Refuses to delete whole vendor trees when state has no managed list.
    pub root: bool,
}

struct RecordedInstall {
    harness: String,
    role: String,
    adapters: Vec<String>,
    /// `None` when this state file predates managed lists.
    managed: Option<ManagedNames>,
}

#[derive(Clone, Default)]
struct ManagedNames {
    skills: Vec<String>,
    agents: Vec<String>,
    rules: Vec<String>,
}

pub fn run(req: &UninstallRequest) -> Result<()> {
    if is_kit_root(&req.target) {
        return Err(Error::RefuseSelfInstall);
    }
    let state_path = req.target.join(state_rel(&req.catalog));
    let text = fs::read_to_string(&state_path).map_err(|_| {
        Error::Msg(format!(
            "no symkit install in {} (missing {})",
            req.target.display(),
            state_rel(&req.catalog)
        ))
    })?;
    let recorded = parse_state(&text);
    if req.root && recorded.managed.is_none() {
        return Err(Error::Msg(
            "this home install has no managed file list; run install --root once, then uninstall --root".into(),
        ));
    }

    println!("Uninstall: {}", req.target.display());
    if req.root {
        println!("Mode:      root (user home)");
    }
    if recorded.harness.is_empty() {
        println!("Harness:   (not recorded)");
    } else {
        println!("Harness:   {}", recorded.harness);
    }
    if !recorded.role.is_empty() {
        println!("Role:      {}", recorded.role);
    }
    println!("Removes symkit-managed skills, agents, rules, harness pointers, and state.");
    println!("Leaves scaffold, docs, and files symkit did not record.");
    if recorded.managed.is_none() {
        println!("Note:      state has no managed list; removing the agent trees for the recorded adapters.");
    }

    confirm(req)?;

    let overlay = req.catalog.canonical.agents_overlay().to_string();
    if let Some(managed) = &recorded.managed {
        remove_managed(&req.target, managed)?;
    } else {
        remove_legacy_trees(&req.target, &recorded.adapters)?;
    }
    remove_overlay_files(&req.target, &overlay)?;
    strip_pointer_file(&req.target.join(req.catalog.canonical.agents_md()))?;
    strip_pointer_file(&req.target.join(".claude").join("CLAUDE.md"))?;
    strip_pointer_file(&req.target.join(".codex").join("AGENTS.md"))?;
    remove_project_claude_md(&req.target)?;
    remove_agent_gitignore_block(&req.target)?;
    if state_path.is_file() {
        fs::remove_file(&state_path)?;
        println!("  removed {}", state_rel(&req.catalog));
    }
    let symkit_dir = req.target.join(".symkit");
    if symkit_dir.is_dir() && dir_is_empty(&symkit_dir)? {
        fs::remove_dir(&symkit_dir)?;
        println!("  removed .symkit/");
    }

    println!();
    println!("Done. Open a new session if this was a home install.");
    Ok(())
}

fn confirm(req: &UninstallRequest) -> Result<()> {
    if req.dry_run {
        println!();
        println!("Dry run — no files removed.");
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

fn state_rel(catalog: &Catalog) -> String {
    if catalog.canonical.state.is_empty() {
        ".symkit/state.yaml".to_string()
    } else {
        catalog.canonical.state.clone()
    }
}

fn parse_state(text: &str) -> RecordedInstall {
    let mut harness = String::new();
    let mut role = String::new();
    let mut adapters = Vec::new();
    let mut saw_managed = false;
    let mut skills = Vec::new();
    let mut agents = Vec::new();
    let mut rules = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("harness:") {
            harness = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("role:") {
            role = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("adapters:") {
            adapters = bracket_list(rest);
        } else if let Some(rest) = line.strip_prefix("managed_skills:") {
            saw_managed = true;
            skills = bracket_list(rest);
        } else if let Some(rest) = line.strip_prefix("managed_agents:") {
            saw_managed = true;
            agents = bracket_list(rest);
        } else if let Some(rest) = line.strip_prefix("managed_rules:") {
            saw_managed = true;
            rules = bracket_list(rest);
        }
    }
    RecordedInstall {
        harness,
        role,
        adapters,
        managed: if saw_managed {
            Some(ManagedNames { skills, agents, rules })
        } else {
            None
        },
    }
}

fn bracket_list(rest: &str) -> Vec<String> {
    let rest = rest.trim().trim_start_matches('[').trim_end_matches(']');
    rest.split_whitespace()
        .map(str::to_string)
        .filter(|name| !name.is_empty())
        .collect()
}

fn remove_managed(target: &Path, managed: &ManagedNames) -> Result<()> {
    for name in &managed.skills {
        remove_named(target, "skills", name)?;
    }
    for name in &managed.agents {
        remove_named(target, "agents", name)?;
    }
    for name in &managed.rules {
        remove_named(target, "rules", name)?;
    }
    Ok(())
}

fn remove_named(target: &Path, kind: &str, name: &str) -> Result<()> {
    if name.is_empty() {
        return Ok(());
    }
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
        let path = target.join(&rel);
        if !path.exists() {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
        println!("  removed {rel}");
    }
    Ok(())
}

fn remove_legacy_trees(target: &Path, adapters: &[String]) -> Result<()> {
    let mut dirs = vec![".agents".to_string()];
    for name in adapters {
        match name.as_str() {
            "grok" => dirs.push(".grok".into()),
            "claude" => dirs.push(".claude".into()),
            "codex" => dirs.push(".codex".into()),
            _ => {}
        }
    }
    for rel in dirs {
        let path = target.join(&rel);
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
            println!("  removed {rel}/");
        }
    }
    Ok(())
}

fn remove_overlay_files(target: &Path, overlay: &str) -> Result<()> {
    let rels = [
        overlay.to_string(),
        format!(".grok/rules/{overlay}"),
        format!(".claude/{overlay}"),
        format!(".codex/{overlay}"),
    ];
    for rel in rels {
        let path = target.join(&rel);
        if path.is_file() {
            fs::remove_file(&path)?;
            println!("  removed {rel}");
        }
    }
    Ok(())
}

fn strip_pointer_file(path: &Path) -> Result<()> {
    if !path.is_file() {
        return Ok(());
    }
    let text = fs::read_to_string(path)?;
    let Some(stripped) = strip_block(&text, POINTER_BEGIN, POINTER_END) else {
        return Ok(());
    };
    let trimmed = stripped.trim();
    let heading = path
        .file_name()
        .map(|name| format!("# {}", name.to_string_lossy()))
        .unwrap_or_default();
    if trimmed.is_empty() || trimmed == heading {
        fs::remove_file(path)?;
        println!("  removed {}", path.display());
    } else {
        let mut out = stripped;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        fs::write(path, out)?;
        println!("  stripped harness pointer {}", path.display());
    }
    Ok(())
}

fn remove_project_claude_md(target: &Path) -> Result<()> {
    let path = target.join("CLAUDE.md");
    if !path.is_file() {
        return Ok(());
    }
    let text = fs::read_to_string(&path)?;
    if text.contains(POINTER_BEGIN) {
        return Ok(());
    }
    if text.contains(CLAUDE_ADAPTER_MARK) && text.contains("symkit") {
        fs::remove_file(&path)?;
        println!("  removed CLAUDE.md");
    }
    Ok(())
}

fn strip_block(text: &str, begin: &str, end: &str) -> Option<String> {
    let start = text.find(begin)?;
    let after = start + begin.len();
    let end_at = text[after..].find(end)? + after + end.len();
    let mut out = String::new();
    out.push_str(text[..start].trim_end_matches('\n'));
    let rest = text[end_at..].trim_start_matches('\n');
    if !out.is_empty() && !rest.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str(rest);
    Some(out)
}

fn dir_is_empty(path: &Path) -> Result<bool> {
    Ok(fs::read_dir(path)?.next().is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_pointer_keeps_repo_text() {
        let text = "repo-specific keep\n\n<!-- BEGIN symkit harness (do not edit this block) -->\nRead it.\n<!-- END symkit harness -->\n";
        let out = strip_block(text, POINTER_BEGIN, POINTER_END).unwrap();
        assert_eq!(out.trim(), "repo-specific keep");
    }

    #[test]
    fn parse_state_without_managed_list() {
        let text = "harness: education\nrole: instructor\nadapters: [grok]\n";
        let recorded = parse_state(text);
        assert!(recorded.managed.is_none());
        assert_eq!(recorded.adapters, vec!["grok".to_string()]);
    }

    #[test]
    fn managed_uninstall_leaves_unmanaged_skill() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".grok/skills/qc-things")).unwrap();
        fs::write(dir.path().join(".grok/skills/qc-things/SKILL.md"), "personal\n").unwrap();
        fs::create_dir_all(dir.path().join(".agents/skills/write-tests")).unwrap();
        fs::write(dir.path().join(".agents/skills/write-tests/SKILL.md"), "kit\n").unwrap();
        fs::create_dir_all(dir.path().join(".grok/skills/write-tests")).unwrap();
        fs::create_dir_all(dir.path().join(".symkit")).unwrap();
        fs::write(
            dir.path().join(".symkit/state.yaml"),
            "harness: engineering\nrole: engineer\nadapters: [grok claude codex]\nmanaged_skills: [write-tests]\nmanaged_agents: []\nmanaged_rules: []\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("AGENTS.md"),
            "# AGENTS.md\n\n<!-- BEGIN symkit harness (do not edit this block) -->\nbody\n<!-- END symkit harness -->\n",
        )
        .unwrap();

        let catalog = Catalog::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("catalog.yaml")).unwrap();
        run(&UninstallRequest {
            target: dir.path().to_path_buf(),
            catalog,
            yes: true,
            dry_run: false,
            root: true,
        })
        .unwrap();

        assert!(dir.path().join(".grok/skills/qc-things/SKILL.md").is_file());
        assert!(!dir.path().join(".agents/skills/write-tests").exists());
        assert!(!dir.path().join(".grok/skills/write-tests").exists());
        assert!(!dir.path().join(".symkit/state.yaml").exists());
        assert!(!dir.path().join("AGENTS.md").exists());
    }
}
