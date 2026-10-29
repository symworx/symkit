// Copyright (c) 2026, Nathaniel T. Berry
// Licensed under the Apache License, Version 2.0.

use std::{
    env,
    io::{
        self,
        IsTerminal,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
    process::ExitCode,
};

use clap::{
    Args,
    Parser,
    Subcommand,
};
use symkit::{
    adapters::resolve_adapters,
    catalog::Catalog,
    error::{
        Error,
        Result,
    },
    help,
    install::{
        self,
        InstallRequest,
    },
    kit::{
        find_kit_root,
        is_kit_root,
    },
    uninstall,
};

#[derive(Parser)]
#[command(
    name = "symkit",
    about = "Install agent instructions, skills, and optional workspace files into another repo. Does not commit.",
    long_about = "Install a named harness (education, research, engineering, …) and role into a target directory.\n\nHarness AGENTS.md is written as AGENTS-SYMKIT.md (last pack wins). A pointer is appended to AGENTS.md; existing AGENTS.md is never replaced. Canonical trees also include .agents/. Vendor adapters (default: grok) are mirrors. Preview always prints. The installer never commits.",
    after_help = help::AFTER_HELP,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Harnesses, roles, pack summaries
    List,
    /// Role matrix and on-disk paths
    Show {
        /// Harness id (`symkit list`)
        harness: String,
    },
    /// Create or activate a workspace
    #[command(
        long_about = "Create the target directory if needed, then install the harness.\nOn a TTY, missing target / harness / role / scaffold are prompted.\nAdd --scaffold to copy workspace folder stubs (skipped if they already exist).\nAdd --docs <id> to copy a catalogued faculty-owned blank (slos, aims, …) into docs/ or documents/."
    )]
    Init(WorkArgs),
    /// Install packs into an existing repo
    #[command(
        long_about = "Install a harness into an existing directory. The target must already exist.\nPass --scaffold only if you also want workspace stubs.\nPass --docs <id> to copy a catalogued blank into docs/ or documents/ (no overwrite unless --force).\nPass --root instead of a directory to install into $HOME for every Grok, Claude, and Codex session."
    )]
    Install(WorkArgs),
    /// Remove a symkit install recorded in .symkit/state.yaml
    #[command(
        long_about = "Remove the harness symkit recorded in .symkit/state.yaml.\nManaged skills, agents, and rules are deleted. Other files stay.\nPass --root to uninstall the home install. Scaffold, docs, and migration-docs are left in place."
    )]
    Uninstall(UninstallArgs),
    /// Rewrite vendor adapters only
    Adapt {
        /// Target directory (already installed)
        target: PathBuf,
        /// Adapter set: grok, claude, codex, all, none
        #[arg(long)]
        adapters: Option<String>,
        /// Add one adapter (repeatable)
        #[arg(long = "adapter", action = clap::ArgAction::Append)]
        adapter: Vec<String>,
        /// Same as --adapters all
        #[arg(long = "adapters-all")]
        adapters_all: bool,
    },
    /// Big-picture flow, GitHub Release vs cargo vs clone, what to commit
    Guide,
}

#[derive(Args, Default)]
struct UninstallArgs {
    /// Target directory
    target: Option<PathBuf>,
    /// Uninstall the home install ($HOME)
    #[arg(long)]
    root: bool,
    /// Remove without prompting
    #[arg(long = "yes", short = 'y')]
    yes: bool,
    /// Print the plan and exit
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Default)]
struct WorkArgs {
    /// Target directory
    target: Option<PathBuf>,
    /// Install into $HOME so Grok, Claude, and Codex load the harness in every session
    #[arg(long)]
    root: bool,
    /// Harness id (`symkit list`)
    #[arg(long)]
    harness: Option<String>,
    /// Role in that harness (`symkit show <harness>`)
    #[arg(long)]
    role: Option<String>,
    /// Replace the role pack list (role skills are not applied)
    #[arg(long = "pack", action = clap::ArgAction::Append)]
    pack: Vec<String>,
    /// Extra in-catalog pack on top of the role
    #[arg(long = "also", action = clap::ArgAction::Append)]
    also: Vec<String>,
    /// Adapter set: grok, claude, codex, all, none (default: grok)
    #[arg(long)]
    adapters: Option<String>,
    /// Add one adapter (repeatable)
    #[arg(long = "adapter", action = clap::ArgAction::Append)]
    adapter: Vec<String>,
    /// Same as --adapters all
    #[arg(long = "adapters-all")]
    adapters_all: bool,
    /// Copy harness workspace stubs (no overwrite unless --force)
    #[arg(long)]
    scaffold: bool,
    /// Create migration-docs/ for PDF/DOCX import (dumps gitignored)
    #[arg(long)]
    migration_docs: bool,
    /// Do not create migration-docs/
    #[arg(long)]
    no_migration_docs: bool,
    /// Copy a catalogued doc template (repeatable). `symkit show <harness>` lists ids
    #[arg(long = "docs", action = clap::ArgAction::Append, value_name = "ID")]
    docs: Vec<String>,
    /// Directory for --docs files (`docs` or `documents`). Required if both exist with --yes
    #[arg(long = "docs-root", value_name = "DIR")]
    docs_root: Option<String>,
    /// Keep leftover role skills / agents / rules
    #[arg(long = "no-prune")]
    no_prune: bool,
    /// Write without prompting
    #[arg(long = "yes", short = 'y')]
    yes: bool,
    /// Overwrite existing scaffold files
    #[arg(long)]
    force: bool,
    /// Print the preview and exit
    #[arg(long)]
    dry_run: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let kit_root = find_kit_root()?;
    let catalog = Catalog::load(kit_root.join("catalog.yaml"))?;

    match cli.command {
        None => {
            // clap already printed help if -h; bare invoke → long help (includes FLOW)
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_long_help()?;
            println!();
            Ok(())
        }
        Some(Command::Guide) => {
            print!("{}", help::guide());
            Ok(())
        }
        Some(Command::List) => {
            print!("{}", catalog.format_list());
            Ok(())
        }
        Some(Command::Show { harness }) => {
            if harness.is_empty() {
                return Err(Error::ShowUsage);
            }
            print!("{}", catalog.format_show(&harness)?);
            Ok(())
        }
        Some(Command::Init(args)) => cmd_init(&kit_root, &catalog, args),
        Some(Command::Install(args)) => cmd_install(&kit_root, &catalog, args),
        Some(Command::Uninstall(args)) => cmd_uninstall(&catalog, args),
        Some(Command::Adapt {
            target,
            adapters,
            adapter,
            adapters_all,
        }) => {
            let target = std::fs::canonicalize(&target).map_err(|_| Error::TargetNotDir(target.clone()))?;
            let adapters = resolve_adapters(adapters.as_deref(), &adapter, adapters_all, &catalog.adapters.default)?;
            install::adapt_only(&target, &adapters)
        }
    }
}

fn cmd_install(kit_root: &Path, catalog: &Catalog, args: WorkArgs) -> Result<()> {
    let (harness, target) = if args.root {
        reject_root_project_flags(&args)?;
        if args.target.is_some() {
            return Err(Error::Msg("pass either a target directory or --root, not both".into()));
        }
        let home = user_home()?;
        if !home.is_dir() {
            return Err(Error::TargetNotDir(home));
        }
        let target = std::fs::canonicalize(&home)?;
        warn_relocated_config_homes(&target);
        let harness = args.harness.clone().ok_or(Error::MissingHarness)?;
        (harness, target)
    } else {
        let target_arg = args.target.clone().ok_or(Error::MissingTarget)?;
        let harness = args.harness.clone().ok_or(Error::MissingHarness)?;
        if !target_arg.is_dir() {
            return Err(Error::TargetNotDir(target_arg));
        }
        let target = std::fs::canonicalize(&target_arg)?;
        (harness, target)
    };
    run_work(kit_root, catalog, args, harness, target, false)
}

fn cmd_uninstall(catalog: &Catalog, args: UninstallArgs) -> Result<()> {
    if args.root && args.target.is_some() {
        return Err(Error::Msg("pass either a target directory or --root, not both".into()));
    }
    let target = if args.root {
        let home = user_home()?;
        if !home.is_dir() {
            return Err(Error::TargetNotDir(home));
        }
        std::fs::canonicalize(&home)?
    } else {
        let target_arg = args.target.clone().ok_or(Error::MissingTarget)?;
        if !target_arg.is_dir() {
            return Err(Error::TargetNotDir(target_arg));
        }
        std::fs::canonicalize(&target_arg)?
    };
    uninstall::run(&uninstall::UninstallRequest {
        target,
        catalog: catalog.clone(),
        yes: args.yes,
        dry_run: args.dry_run,
        root: args.root,
    })
}

fn cmd_init(kit_root: &Path, catalog: &Catalog, mut args: WorkArgs) -> Result<()> {
    if args.root {
        return Err(Error::Msg(
            "init does not take --root; use symkit install --root".into(),
        ));
    }
    if args.target.is_none() {
        if !io::stdin().is_terminal() {
            return Err(Error::MissingTarget);
        }
        print!("Target directory: ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let t = line.trim();
        if t.is_empty() {
            return Err(Error::MissingTarget);
        }
        args.target = Some(PathBuf::from(t));
    }

    if args.harness.is_none() && io::stdin().is_terminal() {
        println!("Active harnesses:");
        for (name, h) in &catalog.harnesses {
            if h.status == "active" {
                println!("  {name}");
            }
        }
        print!("Harness: ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        args.harness = Some(line.trim().to_string());
    }
    let harness = args
        .harness
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or(Error::MissingHarness)?;

    if args.role.is_none() && args.pack.is_empty() && io::stdin().is_terminal() {
        println!("Roles for {harness}:");
        if let Ok(h) = catalog.harness(&harness) {
            for role in h.roles.keys() {
                println!("  {role}");
            }
        }
        print!("Role [default]: ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let r = line.trim();
        if !r.is_empty() {
            args.role = Some(r.to_string());
        }
    }

    let target_arg = args.target.clone().ok_or(Error::MissingTarget)?;
    if !args.scaffold && io::stdin().is_terminal() && !target_arg.exists() {
        print!("Create workspace scaffold? [y/N] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        match line.trim() {
            "y" | "Y" | "yes" | "YES" => args.scaffold = true,
            _ => {}
        }
    }

    if !args.migration_docs
        && !args.no_migration_docs
        && !args.yes
        && io::stdin().is_terminal()
        && catalog
            .harness(&harness)
            .map(|h| h.offer_migration_docs)
            .unwrap_or(false)
    {
        print!("Create migration-docs/ for PDF/DOCX import (dumps gitignored)? [y/N] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        match line.trim() {
            "y" | "Y" | "yes" | "YES" => args.migration_docs = true,
            _ => {}
        }
    }

    std::fs::create_dir_all(&target_arg)?;
    let target = std::fs::canonicalize(&target_arg)?;
    run_work(kit_root, catalog, args, harness, target, true)
}

fn run_work(
    kit_root: &Path,
    catalog: &Catalog,
    args: WorkArgs,
    harness: String,
    target: PathBuf,
    _is_init: bool,
) -> Result<()> {
    if is_kit_root(&target) {
        return Err(Error::RefuseSelfInstall);
    }

    if args.migration_docs && args.no_migration_docs {
        return Err(Error::Msg(
            "pass only one of --migration-docs or --no-migration-docs".into(),
        ));
    }
    let migration_docs = args.migration_docs && !args.no_migration_docs;

    let adapter_default: Vec<String>;
    let adapter_default_ref: &[String] =
        if args.root && args.adapters.is_none() && args.adapter.is_empty() && !args.adapters_all {
            adapter_default = if catalog.adapters.all.is_empty() {
                vec!["grok".into(), "claude".into(), "codex".into()]
            } else {
                catalog.adapters.all.clone()
            };
            &adapter_default
        } else {
            &catalog.adapters.default
        };
    let adapters = resolve_adapters(
        args.adapters.as_deref(),
        &args.adapter,
        args.adapters_all,
        adapter_default_ref,
    )?;

    let mut resolved = catalog.resolve(&harness, args.role.as_deref(), &args.pack)?;
    for extra in args.also {
        if !resolved.packages.iter().any(|p| p == &extra) {
            // validate
            catalog.package_path(&harness, &extra)?;
            if !catalog.package_safe(&harness, &extra)? {
                resolved.private.push(extra.clone());
            } else {
                resolved.student_safe.push(extra.clone());
            }
            resolved.packages.push(extra);
        }
    }

    let req = InstallRequest {
        kit_root: kit_root.to_path_buf(),
        target,
        catalog: catalog.clone(),
        resolved,
        adapters,
        scaffold: args.scaffold,
        force: args.force,
        prune: !args.no_prune,
        yes: args.yes,
        dry_run: args.dry_run,
        docs: args.docs,
        docs_root: args.docs_root,
        migration_docs,
        root: args.root,
    };
    install::run(&req)
}

fn reject_root_project_flags(args: &WorkArgs) -> Result<()> {
    if args.scaffold {
        return Err(Error::Msg(
            "--root does not take --scaffold; that flag is for a project tree".into(),
        ));
    }
    if !args.docs.is_empty() || args.docs_root.is_some() {
        return Err(Error::Msg(
            "--root does not take --docs or --docs-root; those flags are for a project tree".into(),
        ));
    }
    if args.migration_docs || args.no_migration_docs {
        return Err(Error::Msg(
            "--root does not take --migration-docs; that flag is for a project tree".into(),
        ));
    }
    Ok(())
}

fn user_home() -> Result<PathBuf> {
    if cfg!(windows) {
        if let Some(path) = env::var_os("USERPROFILE") {
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }
    if let Some(path) = env::var_os("HOME") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    Err(Error::Msg("--root needs HOME (or USERPROFILE on Windows)".into()))
}

fn warn_relocated_config_homes(home: &Path) {
    warn_if_relocated("GROK_HOME", home.join(".grok"));
    warn_if_relocated("CODEX_HOME", home.join(".codex"));
}

fn warn_if_relocated(var: &str, expected: PathBuf) {
    let Some(raw) = env::var_os(var) else {
        return;
    };
    if raw.is_empty() || paths_equivalent(Path::new(&raw), &expected) {
        return;
    }
    eprintln!(
        "warn: {var} is {}; --root still writes {}.",
        Path::new(&raw).display(),
        expected.display()
    );
}

fn paths_equivalent(given: &Path, expected: &Path) -> bool {
    fn norm(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
    norm(given) == norm(expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_rejects_scaffold() {
        let args = WorkArgs {
            root: true,
            scaffold: true,
            ..WorkArgs::default()
        };
        let err = reject_root_project_flags(&args).unwrap_err().to_string();
        assert!(err.contains("--scaffold"), "{err}");
    }

    #[test]
    fn root_rejects_docs_and_migration() {
        let docs = WorkArgs {
            root: true,
            docs: vec!["slos".into()],
            ..WorkArgs::default()
        };
        let err = reject_root_project_flags(&docs).unwrap_err().to_string();
        assert!(err.contains("--docs"), "{err}");

        let migration = WorkArgs {
            root: true,
            migration_docs: true,
            ..WorkArgs::default()
        };
        let err = reject_root_project_flags(&migration).unwrap_err().to_string();
        assert!(err.contains("--migration-docs"), "{err}");
    }

    #[test]
    fn user_home_follows_home_when_set() {
        if env::var_os("HOME").is_some() {
            assert!(user_home().is_ok());
        }
    }
}
