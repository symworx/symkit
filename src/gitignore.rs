// Copyright (c) 2026, Nathaniel T. Berry
// Licensed under the Apache License, Version 2.0.

use std::{
    fs::{
        self,
        OpenOptions,
    },
    io::Write,
    path::Path,
};

use crate::error::Result;

const PATTERNS: &[&str] = &[
    ".agents/", ".grok/", ".claude/", ".codex/", ".symkit/", "*~", ".*.swp", ".*.swo", "*.un~", "\\#*\\#", ".#*",
];
const MARKER_BEGIN: &str = "# BEGIN symkit agent trees (do not commit)";
const MARKER_END: &str = "# END symkit agent trees";

pub fn ensure_agent_gitignore(target: &Path) -> Result<String> {
    let gi = target.join(".gitignore");
    if !gi.exists() {
        fs::write(&gi, "")?;
        println!("  created {}", gi.display());
    }

    let existing = fs::read_to_string(&gi)?;
    let missing: Vec<&str> = PATTERNS.iter().copied().filter(|p| !existing.contains(p)).collect();

    if missing.is_empty() {
        println!("  gitignore: agent trees already covered");
        return Ok("covered".into());
    }

    let mut f = OpenOptions::new().append(true).open(&gi)?;
    writeln!(f)?;
    writeln!(f, "{MARKER_BEGIN}")?;
    writeln!(
        f,
        "# Installed by symkit — local agent config and editor swap/backup files"
    )?;
    for p in &missing {
        writeln!(f, "{p}")?;
    }
    writeln!(f, "{MARKER_END}")?;
    println!("  gitignore: ensured {}", missing.join(" "));
    Ok(missing.join(" "))
}

/// Drop the symkit agent-tree block. Leave the rest of `.gitignore`.
/// Deletes the file when that block was the only content.
pub fn remove_agent_gitignore_block(target: &Path) -> Result<bool> {
    let gi = target.join(".gitignore");
    if !gi.is_file() {
        return Ok(false);
    }
    let existing = fs::read_to_string(&gi)?;
    let Some(stripped) = strip_marker_block(&existing, MARKER_BEGIN, MARKER_END) else {
        return Ok(false);
    };
    if stripped.trim().is_empty() {
        fs::remove_file(&gi)?;
        println!("  removed {}", gi.display());
    } else {
        fs::write(&gi, stripped)?;
        println!("  gitignore: removed symkit agent block");
    }
    Ok(true)
}

fn strip_marker_block(text: &str, begin: &str, end: &str) -> Option<String> {
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
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

const MIG_BEGIN: &str = "# BEGIN symkit migration-docs (do not commit dumps)";
const MIG_END: &str = "# END symkit migration-docs";
const MIG_PATTERNS: &[&str] = &["migration-docs/**", "!migration-docs/README.md"];

/// Ignore PDF/DOCX dumps; keep a tracked README if present.
pub fn ensure_migration_gitignore(target: &Path) -> Result<String> {
    let gi = target.join(".gitignore");
    if !gi.exists() {
        fs::write(&gi, "")?;
    }
    let existing = fs::read_to_string(&gi)?;
    if existing.contains(MIG_BEGIN) {
        println!("  gitignore: migration-docs already covered");
        return Ok("covered".into());
    }
    let mut f = OpenOptions::new().append(true).open(&gi)?;
    writeln!(f)?;
    writeln!(f, "{MIG_BEGIN}")?;
    writeln!(f, "# Local PDF/Word dumps for migrate-course; README may be tracked")?;
    for p in MIG_PATTERNS {
        writeln!(f, "{p}")?;
    }
    writeln!(f, "{MIG_END}")?;
    println!("  gitignore: migration-docs dumps ignored");
    Ok("migration-docs".into())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn additive() {
        let dir = tempdir().unwrap();
        ensure_agent_gitignore(dir.path()).unwrap();
        let text = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(text.contains(".agents/"));
        assert!(text.contains(".symkit/"));
        assert!(text.contains(".*.swp"));
        assert!(text.contains(".#*"));
        ensure_agent_gitignore(dir.path()).unwrap();
        let again = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert_eq!(again.matches("# BEGIN symkit agent trees").count(), 1);
    }

    #[test]
    fn migration_docs_marker_once() {
        let dir = tempdir().unwrap();
        ensure_migration_gitignore(dir.path()).unwrap();
        let text = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(text.contains("migration-docs/**"));
        assert!(text.contains("!migration-docs/README.md"));
        ensure_migration_gitignore(dir.path()).unwrap();
        let again = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert_eq!(again.matches("# BEGIN symkit migration-docs").count(), 1);
    }

    #[test]
    fn remove_agent_block_keeps_other_lines() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "data/private/\n").unwrap();
        ensure_agent_gitignore(dir.path()).unwrap();
        assert!(remove_agent_gitignore_block(dir.path()).unwrap());
        let text = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(text.contains("data/private/"));
        assert!(!text.contains("BEGIN symkit agent trees"));
        assert!(!remove_agent_gitignore_block(dir.path()).unwrap());
    }
}
