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
    writeln!(
        f,
        "# Local PDF/Word dumps for migrate-course; README may be tracked"
    )?;
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
        assert_eq!(
            again.matches("# BEGIN symkit migration-docs").count(),
            1
        );
    }
}
