// Copyright (c) 2026, Nathaniel T. Berry
// Licensed under the Apache License, Version 2.0.

use std::{
    fs,
    path::Path,
};

use crate::{
    error::{
        Error,
        Result,
    },
    fsutil::{
        dir_nonempty,
        merge_tree,
        mirror_tree,
    },
};

pub fn parse_adapters_spec(spec: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for tok in spec.split(|c: char| c == ',' || c.is_whitespace()) {
        if tok.is_empty() {
            continue;
        }
        match tok {
            "all" => {
                return Ok(vec!["grok".into(), "claude".into(), "codex".into()]);
            }
            "none" => return Ok(Vec::new()),
            "grok" | "claude" | "codex" => {
                if !out.iter().any(|a| a == tok) {
                    out.push(tok.to_string());
                }
            }
            other => return Err(Error::UnknownAdapter(other.to_string())),
        }
    }
    Ok(out)
}

pub fn add_adapter(list: &mut Vec<String>, name: &str) -> Result<()> {
    match name {
        "all" => {
            *list = vec!["grok".into(), "claude".into(), "codex".into()];
        }
        "none" => list.clear(),
        "grok" | "claude" | "codex" => {
            if !list.iter().any(|a| a == name) {
                list.push(name.to_string());
            }
        }
        other => return Err(Error::UnknownAdapter(other.to_string())),
    }
    Ok(())
}

pub fn resolve_adapters(
    spec: Option<&str>,
    extra: &[String],
    adapters_all: bool,
    default: &[String],
) -> Result<Vec<String>> {
    if adapters_all {
        return Ok(vec!["grok".into(), "claude".into(), "codex".into()]);
    }
    let mut explicit = false;
    let mut out = Vec::new();
    if let Some(s) = spec {
        explicit = true;
        out = parse_adapters_spec(s)?;
    }
    for name in extra {
        if !explicit {
            out.clear();
            explicit = true;
        }
        add_adapter(&mut out, name)?;
    }
    if !explicit {
        out = default.to_vec();
    }
    Ok(out)
}

pub fn write_selected_adapters(target: &Path, adapters: &[String], root: bool) -> Result<()> {
    let label = if adapters.is_empty() {
        "none".to_string()
    } else {
        adapters.join(" ")
    };
    println!("Writing adapters ({label})...");
    if adapters.is_empty() {
        println!("  (canonical .agents/ only; no vendor mirrors)");
        return Ok(());
    }
    if adapters.iter().any(|a| a == "grok") {
        copy_vendor_tree(target, ".grok", root)?;
    }
    if adapters.iter().any(|a| a == "claude") {
        if !root {
            write_claude_pointer(target)?;
        }
        copy_vendor_tree(target, ".claude", root)?;
    }
    if adapters.iter().any(|a| a == "codex") {
        copy_codex_skills(target, root)?;
    }
    Ok(())
}

const POINTER_BEGIN: &str = "<!-- BEGIN symkit harness (do not edit this block) -->";
const POINTER_END: &str = "<!-- END symkit harness -->";

/// Point a user-level instructions file at the harness overlay.
///
/// Appends once. An existing file is left in place ahead of the block.
pub fn write_root_instruction_pointers(target: &Path, adapters: &[String], overlay_name: &str) -> Result<()> {
    if adapters.iter().any(|a| a == "claude") {
        let dir = target.join(".claude");
        if dir.join(overlay_name).is_file() {
            ensure_import_pointer(&dir.join("CLAUDE.md"), overlay_name)?;
            println!("  .claude/CLAUDE.md → {overlay_name}");
        }
    }
    if adapters.iter().any(|a| a == "codex") {
        let dir = target.join(".codex");
        let override_path = dir.join("AGENTS.override.md");
        if override_path.is_file() {
            eprintln!(
                "warn: {} exists; Codex reads it instead of AGENTS.md.",
                override_path.display()
            );
        }
        if dir.join(overlay_name).is_file() {
            ensure_import_pointer(&dir.join("AGENTS.md"), overlay_name)?;
            println!("  .codex/AGENTS.md → {overlay_name}");
        }
    }
    Ok(())
}

fn ensure_import_pointer(path: &Path, overlay: &str) -> Result<()> {
    let block = format!("{POINTER_BEGIN}\n@{overlay}\n{POINTER_END}\n");
    if path.is_file() {
        let existing = fs::read_to_string(path)?;
        if existing.contains(POINTER_BEGIN) {
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
        fs::write(path, out)?;
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, block)?;
    }
    Ok(())
}

fn write_claude_pointer(target: &Path) -> Result<()> {
    let overlay = target.join("AGENTS-SYMKIT.md").is_file();
    let agents = target.join("AGENTS.md").is_file();
    if !overlay && !agents {
        return Ok(());
    }
    let mut body = String::new();
    if overlay {
        body.push_str("@AGENTS-SYMKIT.md\n");
    }
    if agents {
        body.push_str("@AGENTS.md\n");
    }
    body.push_str(
        "\n<!-- Adapter for Claude Code. Harness rules in AGENTS-SYMKIT.md; repo rules in AGENTS.md (symkit). -->\n",
    );
    fs::write(target.join("CLAUDE.md"), body)?;
    println!("  adapter CLAUDE.md → harness/repo AGENTS files");
    Ok(())
}

fn copy_vendor_tree(target: &Path, vendor_root: &str, root: bool) -> Result<()> {
    for kind in ["rules", "skills", "agents"] {
        let src = target.join(".agents").join(kind);
        if dir_nonempty(&src) {
            let dest = target.join(vendor_root).join(kind);
            if root {
                // Keep skills and agents the user already had. Project installs
                // replace the vendor tree so it stays a mirror of .agents/.
                merge_tree(&src, &dest)?;
            } else {
                mirror_tree(&src, &dest)?;
            }
            println!("  adapter {vendor_root}/{kind}/ ← .agents/{kind}/");
        }
    }
    Ok(())
}

fn copy_codex_skills(target: &Path, root: bool) -> Result<()> {
    let src = target.join(".agents").join("skills");
    if dir_nonempty(&src) {
        let dest = target.join(".codex").join("skills");
        if root {
            merge_tree(&src, &dest)?;
        } else {
            mirror_tree(&src, &dest)?;
        }
        println!("  adapter .codex/skills/ ← .agents/skills/");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_pointer_appends_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        fs::write(&path, "keep me\n").unwrap();
        ensure_import_pointer(&path, "AGENTS-SYMKIT.md").unwrap();
        ensure_import_pointer(&path, "AGENTS-SYMKIT.md").unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("keep me"));
        assert_eq!(text.matches(POINTER_BEGIN).count(), 1);
        assert!(text.contains("@AGENTS-SYMKIT.md"));
    }

    #[test]
    fn spec_all_none() {
        assert_eq!(parse_adapters_spec("all").unwrap(), ["grok", "claude", "codex"]);
        assert!(parse_adapters_spec("none").unwrap().is_empty());
        assert_eq!(parse_adapters_spec("grok,claude").unwrap(), ["grok", "claude"]);
    }
}
