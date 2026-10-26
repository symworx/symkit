// Copyright (c) 2026, Nathaniel T. Berry
// Licensed under the Apache License, Version 2.0.

use std::{
    collections::BTreeMap,
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use serde::Deserialize;

use crate::error::{
    Error,
    Result,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub kit: KitMeta,
    #[serde(default)]
    pub adapters: AdaptersConfig,
    #[serde(default)]
    pub canonical: Canonical,
    #[serde(default)]
    pub library: Library,
    #[serde(default)]
    pub core: CorePaths,
    #[serde(default)]
    pub harnesses: BTreeMap<String, Harness>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct KitMeta {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AdaptersConfig {
    #[serde(default)]
    pub default: Vec<String>,
    #[serde(default)]
    pub all: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Canonical {
    #[serde(default = "default_agents_md")]
    pub agents_md: String,
    #[serde(default = "default_agents_overlay")]
    pub agents_overlay: String,
    #[serde(default = "default_state")]
    pub state: String,
}

impl Default for Canonical {
    fn default() -> Self {
        Self {
            agents_md: default_agents_md(),
            agents_overlay: default_agents_overlay(),
            state: default_state(),
        }
    }
}

fn default_agents_md() -> String {
    "AGENTS.md".into()
}

fn default_agents_overlay() -> String {
    "AGENTS-SYMKIT.md".into()
}

fn default_state() -> String {
    ".symkit/state.yaml".into()
}

impl Canonical {
    pub fn agents_md(&self) -> &str {
        if self.agents_md.is_empty() {
            "AGENTS.md"
        } else {
            &self.agents_md
        }
    }

    pub fn agents_overlay(&self) -> &str {
        if self.agents_overlay.is_empty() {
            "AGENTS-SYMKIT.md"
        } else {
            &self.agents_overlay
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub skills: String,
    /// Old skill name → canonical skill. One body on disk.
    #[serde(default)]
    pub skill_aliases: BTreeMap<String, SkillAlias>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SkillAlias {
    pub target: String,
    #[serde(default)]
    pub deprecated: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CorePaths {
    #[serde(default)]
    pub rules: String,
    #[serde(default)]
    pub always_skills: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RoleSpec {
    Packages(Vec<String>),
    Assign(RoleAssign),
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoleAssign {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    /// Agent filename stems this role's session may spawn. Leaves stay empty.
    #[serde(default)]
    pub may_spawn: Vec<String>,
}

impl RoleSpec {
    pub fn packages(&self) -> &[String] {
        match self {
            RoleSpec::Packages(v) => v,
            RoleSpec::Assign(a) => &a.packages,
        }
    }

    pub fn skills(&self) -> &[String] {
        match self {
            RoleSpec::Packages(_) => &[],
            RoleSpec::Assign(a) => &a.skills,
        }
    }

    pub fn may_spawn(&self) -> &[String] {
        match self {
            RoleSpec::Packages(_) => &[],
            RoleSpec::Assign(a) => &a.may_spawn,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Harness {
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub default_role: String,
    #[serde(default)]
    pub workspace: String,
    /// Offer a local `migration-docs/` dump dir (PDF/DOCX import).
    #[serde(default)]
    pub offer_migration_docs: bool,
    #[serde(default)]
    pub templates: BTreeMap<String, DocTemplate>,
    #[serde(default)]
    pub packages: BTreeMap<String, PackageMeta>,
    #[serde(default)]
    pub roles: BTreeMap<String, RoleSpec>,
    #[serde(default)]
    pub prune: BTreeMap<String, PruneSpec>,
}

fn default_status() -> String {
    "active".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocTemplate {
    pub path: String,
    #[serde(default)]
    pub dest: String,
}

impl DocTemplate {
    pub fn dest_name(&self) -> &str {
        if self.dest.is_empty() {
            Path::new(&self.path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("doc.md")
        } else {
            &self.dest
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageMeta {
    #[serde(default)]
    pub description: String,
    pub path: String,
    #[serde(default)]
    pub student_safe: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PruneSpec {
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Resolve {
    pub harness: String,
    pub role: String,
    pub packages: Vec<String>,
    pub skills: Vec<String>,
    pub may_spawn: Vec<String>,
    /// Alias names requested by this role. Install still uses `skills`.
    pub deprecated_skills: Vec<String>,
    pub private: Vec<String>,
    pub student_safe: Vec<String>,
    pub workspace: String,
    pub prune: PruneSpec,
    pub core_rules: String,
    pub library_skills: String,
}

impl Catalog {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(Error::CatalogMissing(path.to_path_buf()));
        }
        let text = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&text)?)
    }

    pub fn harness(&self, name: &str) -> Result<&Harness> {
        self.harnesses.get(name).ok_or_else(|| Error::UnknownHarness {
            name: name.to_string(),
            known: join_keys(&self.harnesses),
        })
    }

    pub fn resolve(&self, harness: &str, role: Option<&str>, extra_packs: &[String]) -> Result<Resolve> {
        let h = self.harness(harness)?;
        if h.status != "active" {
            return Err(Error::NotInstallable(harness.to_string(), h.status.clone()));
        }

        let (used_role, selected, role_skills, may_spawn) = if extra_packs.is_empty() {
            let used = role
                .filter(|r| !r.is_empty())
                .map(str::to_string)
                .filter(|r| !r.is_empty())
                .or_else(|| {
                    if h.default_role.is_empty() {
                        None
                    } else {
                        Some(h.default_role.clone())
                    }
                })
                .ok_or_else(|| Error::RoleRequired(harness.to_string()))?;
            let spec = h.roles.get(&used).ok_or_else(|| Error::UnknownRole {
                role: used.clone(),
                harness: harness.to_string(),
                known: join_keys(&h.roles),
            })?;
            (
                used,
                spec.packages().to_vec(),
                spec.skills().to_vec(),
                spec.may_spawn().to_vec(),
            )
        } else {
            (
                role.unwrap_or("").to_string(),
                extra_packs.to_vec(),
                Vec::new(),
                Vec::new(),
            )
        };

        let unknown: Vec<_> = selected
            .iter()
            .filter(|p| !h.packages.contains_key(p.as_str()))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            return Err(Error::UnknownPackages {
                harness: harness.to_string(),
                names: unknown.join(", "),
            });
        }

        let private: Vec<String> = selected
            .iter()
            .filter(|p| !h.packages[*p].student_safe)
            .cloned()
            .collect();
        let student_safe: Vec<String> = selected
            .iter()
            .filter(|p| h.packages[*p].student_safe)
            .cloned()
            .collect();
        let prune = h.prune.get(&used_role).cloned().unwrap_or_default();
        let (role_skills, mut deprecated_skills) = self.canonicalize_skills(&role_skills);
        let (always_skills, always_deprecated) = self.canonicalize_skills(&self.core.always_skills);
        for name in always_deprecated {
            if !deprecated_skills.iter().any(|existing| existing == &name) {
                deprecated_skills.push(name);
            }
        }
        let skills = merge_skill_names(&always_skills, &role_skills);

        Ok(Resolve {
            harness: harness.to_string(),
            role: used_role,
            packages: selected,
            skills,
            may_spawn,
            deprecated_skills,
            private,
            student_safe,
            workspace: h.workspace.clone(),
            prune,
            core_rules: self.core.rules.clone(),
            library_skills: self.library.skills.clone(),
        })
    }

    pub fn package_path(&self, harness: &str, pkg: &str) -> Result<PathBuf> {
        let h = self.harness(harness)?;
        let meta = h.packages.get(pkg).ok_or_else(|| Error::UnknownPackage {
            harness: harness.to_string(),
            package: pkg.to_string(),
        })?;
        Ok(PathBuf::from(&meta.path))
    }

    pub fn package_safe(&self, harness: &str, pkg: &str) -> Result<bool> {
        let h = self.harness(harness)?;
        let meta = h.packages.get(pkg).ok_or_else(|| Error::UnknownPackage {
            harness: harness.to_string(),
            package: pkg.to_string(),
        })?;
        Ok(meta.student_safe)
    }

    pub fn format_list(&self) -> String {
        let name = if self.kit.name.is_empty() {
            "symkit"
        } else {
            &self.kit.name
        };
        let adapters = self.adapters.default.join(" ");

        let mut meta: Vec<(&str, &str)> = vec![("KIT", name)];
        if !self.kit.description.is_empty() {
            meta.push(("DESC", &self.kit.description));
        }
        meta.push(("ADAPTERS_DEFAULT", &adapters));

        let mut rows: Vec<Vec<String>> = Vec::new();
        for (hname, h) in &self.harnesses {
            let role = if h.default_role.is_empty() {
                "-"
            } else {
                h.default_role.as_str()
            };
            rows.push(vec![
                hname.clone(),
                h.status.clone(),
                role.to_string(),
                h.description.clone(),
            ]);
        }

        let mut out = aligned_kv(&meta);
        out.push('\n');
        out.push_str(&aligned_table(
            &["HARNESS", "STATUS", "DEFAULT_ROLE", "DESCRIPTION"],
            &rows,
        ));
        let aliases = self.format_skill_aliases();
        if !aliases.is_empty() {
            out.push('\n');
            out.push_str(&aliases);
        }
        out
    }

    pub fn format_show(&self, name: &str) -> Result<String> {
        let h = self.harness(name)?;
        let mut out = String::new();
        out.push_str(&format!("HARNESS={name}\n"));
        out.push_str(&format!("STATUS={}\n", h.status));
        out.push_str(&format!("DESCRIPTION={}\n", h.description));
        out.push_str(&format!("DEFAULT_ROLE={}\n", h.default_role));
        out.push_str(&format!("WORKSPACE={}\n", h.workspace));
        if !h.templates.is_empty() {
            out.push_str("TEMPLATE\tDEST\tPATH\n");
            for (id, tmpl) in &h.templates {
                out.push_str(&format!("{id}\t{}\t{}\n", tmpl.dest_name(), tmpl.path));
            }
        }
        out.push_str("PACKAGE\tSAFE\tPATH\tDESCRIPTION\n");
        for (pkg, meta) in &h.packages {
            let safe = if meta.student_safe { "yes" } else { "no" };
            out.push_str(&format!("{pkg}\t{safe}\t{}\t{}\n", meta.path, meta.description));
        }
        out.push_str("ROLE\tPACKAGES\tSKILLS\tMAY_SPAWN\n");
        for (role, spec) in &h.roles {
            let skills = spec
                .skills()
                .iter()
                .map(|name| self.canonical_skill(name))
                .collect::<Vec<_>>()
                .join(" ");
            out.push_str(&format!(
                "{role}\t{}\t{skills}\t{}\n",
                spec.packages().join(" "),
                spec.may_spawn().join(" ")
            ));
        }
        let aliases = self.format_skill_aliases();
        if !aliases.is_empty() {
            out.push('\n');
            out.push_str(&aliases);
        }
        Ok(out)
    }

    pub fn canonical_skill<'a>(&'a self, name: &'a str) -> &'a str {
        self.library
            .skill_aliases
            .get(name)
            .map(|alias| alias.target.as_str())
            .unwrap_or(name)
    }

    fn canonicalize_skills(&self, names: &[String]) -> (Vec<String>, Vec<String>) {
        let mut out = Vec::new();
        let mut deprecated = Vec::new();
        for name in names {
            let canonical = self.canonical_skill(name);
            if let Some(alias) = self.library.skill_aliases.get(name) {
                if alias.deprecated && !deprecated.iter().any(|existing| existing == name) {
                    deprecated.push(name.clone());
                }
            }
            if !canonical.is_empty() && !out.iter().any(|existing| existing == canonical) {
                out.push(canonical.to_string());
            }
        }
        (out, deprecated)
    }

    fn format_skill_aliases(&self) -> String {
        if self.library.skill_aliases.is_empty() {
            return String::new();
        }
        let rows: Vec<Vec<String>> = self
            .library
            .skill_aliases
            .iter()
            .map(|(name, alias)| {
                vec![
                    name.clone(),
                    alias.target.clone(),
                    if alias.deprecated { "yes" } else { "no" }.to_string(),
                ]
            })
            .collect();
        aligned_table(&["SKILL_ALIAS", "TARGET", "DEPRECATED"], &rows)
    }

    /// Alias targets are real skill directories. Alias names are not.
    pub fn check_skill_aliases(&self, kit: &Path) -> Result<()> {
        for (name, alias) in &self.library.skill_aliases {
            if !valid_agent_stem(name) {
                return Err(Error::InvalidSkillAlias {
                    name: name.clone(),
                    reason: "name must be a bare skill stem".into(),
                });
            }
            if !valid_agent_stem(&alias.target) || self.library.skill_aliases.contains_key(&alias.target) {
                return Err(Error::InvalidSkillAlias {
                    name: name.clone(),
                    reason: format!("target '{}' must be a skill directory, not another alias", alias.target),
                });
            }
            let target = kit.join(self.skill_path(&alias.target));
            if !target.is_dir() {
                return Err(Error::UnknownSkill {
                    name: alias.target.clone(),
                    path: target,
                });
            }
            let alias_dir = kit.join(&self.library.skills).join(name);
            if alias_dir.is_dir() {
                return Err(Error::InvalidSkillAlias {
                    name: name.clone(),
                    reason: format!("alias name is also a skill directory ({})", alias_dir.display()),
                });
            }
        }
        Ok(())
    }

    /// Each `may_spawn` name is an agent file in a package that role installs.
    /// Any other role that does not install such a package must prune the name.
    pub fn check_spawn_agents(&self, kit: &Path) -> Result<()> {
        for (harness_name, harness) in &self.harnesses {
            for (role, spec) in &harness.roles {
                let spawn = spec.may_spawn();
                if spawn.is_empty() {
                    continue;
                }
                let mut seen = Vec::new();
                for name in spawn {
                    if !valid_agent_stem(name) || seen.iter().any(|s| s == name) {
                        return Err(Error::InvalidSpawnAgent {
                            harness: harness_name.clone(),
                            role: role.clone(),
                            name: name.clone(),
                        });
                    }
                    seen.push(name.clone());
                    let shippers = packages_shipping_agent(kit, harness, name);
                    if !spec.packages().iter().any(|pkg| shippers.iter().any(|s| s == pkg)) {
                        return Err(Error::UnknownSpawnAgent {
                            harness: harness_name.clone(),
                            role: role.clone(),
                            name: name.clone(),
                        });
                    }
                    if harness
                        .prune
                        .get(role)
                        .is_some_and(|prune| prune.agents.iter().any(|agent| agent == name))
                    {
                        return Err(Error::SpawnAgentPrunedByOwner {
                            harness: harness_name.clone(),
                            role: role.clone(),
                            name: name.clone(),
                        });
                    }
                    for (other, other_spec) in &harness.roles {
                        if other == role {
                            continue;
                        }
                        let installs = other_spec
                            .packages()
                            .iter()
                            .any(|pkg| shippers.iter().any(|s| s == pkg));
                        let prunes = harness
                            .prune
                            .get(other)
                            .is_some_and(|prune| prune.agents.iter().any(|agent| agent == name));
                        if !installs && !prunes {
                            return Err(Error::SpawnAgentNotPruned {
                                harness: harness_name.clone(),
                                role: other.clone(),
                                name: name.clone(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn skill_path(&self, name: &str) -> PathBuf {
        PathBuf::from(&self.library.skills).join(self.canonical_skill(name))
    }

    pub fn doc_template(&self, harness: &str, id: &str) -> Result<&DocTemplate> {
        let h = self.harness(harness)?;
        h.templates.get(id).ok_or_else(|| Error::UnknownDocTemplate {
            name: id.to_string(),
            harness: harness.to_string(),
            known: join_keys(&h.templates),
        })
    }
}

fn merge_skill_names(always: &[String], role: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for name in always.iter().chain(role.iter()) {
        if !name.is_empty() && !out.iter().any(|s| s == name) {
            out.push(name.clone());
        }
    }
    out
}

fn aligned_kv(rows: &[(&str, &str)]) -> String {
    let width = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let mut out = String::new();
    for (key, value) in rows {
        out.push_str(&format!("{key:<width$}  {value}\n"));
    }
    out
}

fn aligned_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if let Some(w) = widths.get_mut(i) {
                *w = (*w).max(cell.len());
            }
        }
    }
    let mut out = pad_cells(headers.iter().copied(), &widths);
    out.push('\n');
    for row in rows {
        out.push_str(&pad_cells(row.iter().map(String::as_str), &widths));
        out.push('\n');
    }
    out
}

fn pad_cells<'a>(cells: impl IntoIterator<Item = &'a str>, widths: &[usize]) -> String {
    let cells: Vec<&str> = cells.into_iter().collect();
    cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            if i + 1 == cells.len() {
                (*cell).to_string()
            } else {
                format!("{:<width$}", cell, width = widths[i] + 2)
            }
        })
        .collect()
}

fn valid_agent_stem(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', '.'])
}

fn packages_shipping_agent(kit: &Path, harness: &Harness, name: &str) -> Vec<String> {
    harness
        .packages
        .iter()
        .filter(|(_, meta)| {
            kit.join(&meta.path)
                .join(".agents")
                .join("agents")
                .join(format!("{name}.md"))
                .is_file()
        })
        .map(|(pkg, _)| pkg.clone())
        .collect()
}

fn join_keys<V>(map: &BTreeMap<String, V>) -> String {
    if map.is_empty() {
        "(none)".into()
    } else {
        map.keys().cloned().collect::<Vec<_>>().join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_repo() -> Catalog {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("catalog.yaml");
        Catalog::load(path).expect("repo catalog")
    }

    #[test]
    fn resolve_instructor() {
        let cat = load_repo();
        let r = cat.resolve("teaching", Some("instructor"), &[]).unwrap();
        assert_eq!(r.packages, ["shared", "staff", "instructor"]);
        assert_eq!(r.may_spawn, ["materials-author"]);
        assert_eq!(r.private, ["staff", "instructor"]);
        assert_eq!(r.student_safe, ["shared"]);
        assert_eq!(r.prune.agents, ["ta"]);
        let faculty = cat.resolve("teaching", Some("faculty"), &[]).unwrap();
        assert_eq!(faculty.may_spawn, ["materials-author"]);
        assert!(r.skills.contains(&"check-citations".into()));
        assert!(r.skills.contains(&"write-gherkin".into()));
        assert!(r.skills.contains(&"course-prep".into()));
        assert!(r.skills.contains(&"migrate-course".into()));
        assert_eq!(r.library_skills, "core/library/skills");
    }

    #[test]
    fn resolve_researcher_skills() {
        let cat = load_repo();
        let r = cat.resolve("research", Some("researcher"), &[]).unwrap();
        assert_eq!(r.packages, ["shared", "researcher"]);
        assert_eq!(r.may_spawn, ["reviewer"]);
        assert!(r.skills.contains(&"write-gherkin".into()));
        assert!(r.skills.contains(&"write-manuscript".into()));
        assert!(r.skills.contains(&"review-manuscript".into()));
        assert!(r.skills.contains(&"reproducibility-check".into()));
        assert!(!r.skills.iter().any(|skill| skill == "repro-check"));
        assert!(r.deprecated_skills.is_empty());
        assert!(!r.skills.contains(&"course-prep".into()));
        let materials = cat.resolve("research", Some("materials"), &[]).unwrap();
        assert_eq!(materials.packages, ["shared"]);
        assert!(materials.may_spawn.is_empty());
        assert_eq!(materials.prune.agents, ["researcher", "reviewer"]);
    }

    #[test]
    fn repro_check_alias_installs_canonical_skill() {
        let cat = load_repo();
        assert!(
            cat.skill_path("repro-check")
                .ends_with("core/library/skills/reproducibility-check")
        );
        cat.check_skill_aliases(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let listed = cat.format_list();
        assert!(listed.contains("SKILL_ALIAS"));
        assert!(listed.contains("repro-check"));
        assert!(listed.contains("reproducibility-check"));

        let yaml = r#"
version: 1
library:
  skills: skills
  skill_aliases:
    repro-check:
      target: reproducibility-check
      deprecated: true
harnesses:
  demo:
    status: active
    default_role: lead
    packages:
      shared:
        path: pkg
        student_safe: true
    roles:
      lead:
        packages: [shared]
        skills: [repro-check, reproducibility-check]
"#;
        let cat: Catalog = serde_yaml::from_str(yaml).unwrap();
        let resolved = cat.resolve("demo", Some("lead"), &[]).unwrap();
        assert_eq!(resolved.skills, ["reproducibility-check"]);
        assert_eq!(resolved.deprecated_skills, ["repro-check"]);
    }

    #[test]
    fn skill_alias_refuses_missing_target_and_shadow_directory() {
        let dir = tempfile::tempdir().unwrap();
        let kit = dir.path();
        let yaml = r#"
version: 1
library:
  skills: skills
  skill_aliases:
    repro-check:
      target: reproducibility-check
      deprecated: true
harnesses: {}
"#;
        let cat: Catalog = serde_yaml::from_str(yaml).unwrap();
        let missing = cat.check_skill_aliases(kit).unwrap_err();
        assert!(missing.to_string().contains("reproducibility-check"), "{missing}");

        fs::create_dir_all(kit.join("skills/reproducibility-check")).unwrap();
        cat.check_skill_aliases(kit).unwrap();
        fs::create_dir_all(kit.join("skills/repro-check")).unwrap();
        let shadow = cat.check_skill_aliases(kit).unwrap_err();
        assert!(shadow.to_string().contains("repro-check"), "{shadow}");
    }

    #[test]
    fn resolve_learner_omits_staff_skills() {
        let cat = load_repo();
        let r = cat.resolve("teaching", Some("learner"), &[]).unwrap();
        assert!(r.skills.contains(&"lab-tutor".into()));
        assert!(!r.skills.contains(&"write-gherkin".into()));
        assert!(!r.skills.contains(&"evaluate-content".into()));
        assert!(!r.skills.contains(&"migrate-course".into()));
    }

    #[test]
    fn canonical_agents_overlay() {
        let cat = load_repo();
        assert_eq!(cat.canonical.agents_md(), "AGENTS.md");
        assert_eq!(cat.canonical.agents_overlay(), "AGENTS-SYMKIT.md");
    }

    #[test]
    fn resolve_default_role() {
        let cat = load_repo();
        let r = cat.resolve("teaching", None, &[]).unwrap();
        assert_eq!(r.role, "instructor");
    }

    #[test]
    fn resolve_later_refused() {
        let cat: Catalog = serde_yaml::from_str(
            "version: 1\nharnesses:\n  stub:\n    status: later\n    packages: {}\n    roles: {}\n",
        )
        .unwrap();
        let err = cat.resolve("stub", None, &[]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not installable"), "{msg}");
    }

    #[test]
    fn resolve_performance_coach() {
        let cat = load_repo();
        let r = cat.resolve("performance", Some("coach"), &[]).unwrap();
        assert_eq!(r.packages, ["shared", "coach"]);
        assert_eq!(r.may_spawn, ["analyst"]);
        assert!(r.skills.contains(&"session-plan".into()));
        assert!(r.skills.contains(&"movement-review".into()));
        assert!(r.skills.contains(&"physio-explain".into()));
        assert!(!r.skills.contains(&"course-prep".into()));
        let alias = cat.resolve("performance", Some("performance-coach"), &[]).unwrap();
        assert_eq!(alias.may_spawn, ["analyst"]);
        let materials = cat.resolve("performance", Some("materials"), &[]).unwrap();
        assert!(materials.may_spawn.is_empty());
        assert_eq!(materials.prune.agents, ["coach", "analyst"]);
        let packs = cat.resolve("performance", None, &["shared".into()]).unwrap();
        assert!(packs.may_spawn.is_empty());
    }

    #[test]
    fn repo_spawn_agents_match_packages_and_prune() {
        let cat = load_repo();
        cat.check_spawn_agents(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let shown = cat.format_show("performance").unwrap();
        assert!(shown.contains("ROLE\tPACKAGES\tSKILLS\tMAY_SPAWN"));
        assert!(
            shown
                .lines()
                .any(|line| line.starts_with("coach\t") && line.ends_with("\tanalyst"))
        );
    }

    #[test]
    fn spawn_refuses_missing_file_missing_prune_and_owner_prune() {
        let dir = tempfile::tempdir().unwrap();
        let kit = dir.path();
        let agent_dir = kit.join("pkg/lead/.agents/agents");
        fs::create_dir_all(&agent_dir).unwrap();
        fs::write(agent_dir.join("helper.md"), "# helper\n").unwrap();
        fs::create_dir_all(kit.join("pkg/shared")).unwrap();

        let missing_prune = demo_catalog(false, false, "[helper]");
        let err = missing_prune.check_spawn_agents(kit).unwrap_err();
        assert!(err.to_string().contains("materials"), "{err}");

        let ok = demo_catalog(false, true, "[helper]");
        ok.check_spawn_agents(kit).unwrap();

        let missing_file = demo_catalog(false, true, "[nope]");
        let err = missing_file.check_spawn_agents(kit).unwrap_err();
        assert!(err.to_string().contains("nope"), "{err}");

        let owner_prunes = demo_catalog(true, true, "[helper]");
        let err = owner_prunes.check_spawn_agents(kit).unwrap_err();
        assert!(err.to_string().contains("prunes"), "{err}");

        let bad_name = demo_catalog(false, true, "[helper.md]");
        let err = bad_name.check_spawn_agents(kit).unwrap_err();
        assert!(err.to_string().contains("invalid"), "{err}");
    }

    fn demo_catalog(lead_prunes: bool, materials_prunes: bool, may_spawn: &str) -> Catalog {
        let lead_prune = if lead_prunes { "{ agents: [helper] }" } else { "{}" };
        let materials_prune = if materials_prunes { "{ agents: [helper] }" } else { "{}" };
        let yaml = format!(
            r#"
version: 1
harnesses:
  demo:
    status: active
    packages:
      lead:
        path: pkg/lead
        student_safe: true
      shared:
        path: pkg/shared
        student_safe: true
    roles:
      lead:
        packages: [lead]
        may_spawn: {may_spawn}
      materials:
        packages: [shared]
    prune:
      lead: {lead_prune}
      materials: {materials_prune}
"#
        );
        serde_yaml::from_str(&yaml).unwrap()
    }

    #[test]
    fn resolve_unknown_role() {
        let cat = load_repo();
        let err = cat.resolve("teaching", Some("dean"), &[]).unwrap_err();
        assert!(err.to_string().contains("unknown role"));
    }

    #[test]
    fn resolve_explicit_packs() {
        let cat = load_repo();
        let r = cat
            .resolve("teaching", None, &["shared".into(), "learner".into()])
            .unwrap();
        assert_eq!(r.packages, ["shared", "learner"]);
        assert!(r.role.is_empty());
        assert_eq!(r.skills, ["check-citations"]);
    }

    #[test]
    fn list_has_header() {
        let cat = load_repo();
        let s = cat.format_list();
        assert!(s.lines().any(|l| l.starts_with("HARNESS")));
        assert!(s.contains("teaching"));
        assert!(s.contains("product"));
        assert!(s.contains("creative"));
        assert!(s.contains("performance"));
        assert!(s.contains("engineering"));
    }

    #[test]
    fn resolve_product_pm_alias() {
        let cat = load_repo();
        let r = cat.resolve("product", Some("pm"), &[]).unwrap();
        assert_eq!(r.packages, ["shared", "product-manager"]);
        assert!(r.skills.contains(&"write-prd".into()));
        assert!(r.skills.contains(&"write-gherkin".into()));
        assert!(!r.skills.contains(&"naming".into()));
        assert!(!r.private.iter().any(|p| p == "product-manager"));
    }

    #[test]
    fn resolve_creative_director() {
        let cat = load_repo();
        let r = cat.resolve("creative", Some("creative"), &[]).unwrap();
        assert_eq!(r.packages, ["shared", "creative-director"]);
        assert_eq!(r.may_spawn, ["copywriter", "editor"]);
        assert!(r.skills.contains(&"naming".into()));
        assert!(!r.skills.iter().any(|s| s == "write-prd"));
        let director = cat
            .resolve("creative", Some("creative-director"), &[])
            .unwrap();
        assert_eq!(director.may_spawn, ["copywriter", "editor"]);
    }

    #[test]
    fn resolve_creative_materials_prunes_specialists() {
        let cat = load_repo();
        let r = cat.resolve("creative", Some("materials"), &[]).unwrap();
        assert_eq!(r.packages, ["shared"]);
        assert_eq!(r.prune.agents, ["creative-director", "copywriter", "editor"]);
    }

    #[test]
    fn resolve_engineer() {
        let cat = load_repo();
        let r = cat.resolve("engineering", Some("swe"), &[]).unwrap();
        assert_eq!(r.packages, ["shared", "engineer"]);
        assert!(r.skills.contains(&"write-tests".into()));
        assert!(r.skills.contains(&"write-docs".into()));
        assert!(r.skills.contains(&"write-gherkin".into()));
        assert!(!r.skills.contains(&"write-prd".into()));
    }

    #[test]
    fn show_teaching_roles() {
        let cat = load_repo();
        let s = cat.format_show("teaching").unwrap();
        assert!(s.lines().any(|l| l.starts_with("instructor\t")));
    }

    #[test]
    fn show_lists_doc_templates() {
        let cat = load_repo();
        let teaching = cat.format_show("teaching").unwrap();
        assert!(teaching.contains("TEMPLATE\tDEST\tPATH"));
        assert!(teaching.lines().any(|l| l.starts_with("slos\t")));
        let research = cat.format_show("research").unwrap();
        assert!(research.lines().any(|l| l.starts_with("aims\t")));
        assert!(research.lines().any(|l| l.starts_with("protocol\t")));
        let product = cat.format_show("product").unwrap();
        assert!(!product.contains("TEMPLATE\t"));
    }

    #[test]
    fn doc_template_lookup() {
        let cat = load_repo();
        let slos = cat.doc_template("teaching", "slos").unwrap();
        assert_eq!(slos.dest_name(), "slos.md");
        let err = cat.doc_template("teaching", "aims").unwrap_err();
        assert!(err.to_string().contains("unknown doc template"));
    }
}
