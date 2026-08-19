use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ui::marketplaces::list_enabled_plugin_dirs;
use crate::ui::plugin_source::ItemSource;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillOrAgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub built_in: bool,
    pub source: ItemSource,
}

#[derive(Debug, Default, Deserialize)]
struct Frontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

fn parse_frontmatter(contents: &str) -> Frontmatter {
    let Some(rest) = contents.strip_prefix("---\n") else {
        return Frontmatter::default();
    };
    let Some(end) = rest.find("\n---") else {
        return Frontmatter::default();
    };
    serde_yaml::from_str(&rest[..end]).unwrap_or_default()
}

fn scan_skill_dirs(dir: &Path, source: &ItemSource) -> Result<Vec<SkillOrAgentInfo>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let skill_md = path.join("SKILL.md");
        let frontmatter = if skill_md.is_file() {
            parse_frontmatter(&fs::read_to_string(&skill_md)?)
        } else {
            Frontmatter::default()
        };
        let built_in = entry.path().symlink_metadata()?.file_type().is_symlink();
        skills.push(SkillOrAgentInfo {
            name: frontmatter.name.unwrap_or_else(|| id.clone()),
            description: frontmatter.description.unwrap_or_default(),
            built_in,
            source: source.clone(),
            id,
        });
    }
    Ok(skills)
}

fn scan_agent_files(dir: &Path, source: &ItemSource) -> Result<Vec<SkillOrAgentInfo>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut agents = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let id = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let frontmatter = parse_frontmatter(&fs::read_to_string(&path)?);
        let built_in = entry.path().symlink_metadata()?.file_type().is_symlink();
        agents.push(SkillOrAgentInfo {
            name: frontmatter.name.unwrap_or_else(|| id.clone()),
            description: frontmatter.description.unwrap_or_default(),
            built_in,
            source: source.clone(),
            id,
        });
    }
    Ok(agents)
}

pub fn list_skills(env_dir: &Path) -> Result<Vec<SkillOrAgentInfo>> {
    let mut skills = scan_skill_dirs(&env_dir.join("skills"), &ItemSource::Native)?;
    for plugin_dir in list_enabled_plugin_dirs(env_dir)? {
        let source = ItemSource::Plugin {
            marketplace: plugin_dir.marketplace,
            plugin: plugin_dir.plugin,
        };
        skills.extend(scan_skill_dirs(&plugin_dir.path.join("skills"), &source)?);
    }
    skills.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(skills)
}

pub fn list_agents(env_dir: &Path) -> Result<Vec<SkillOrAgentInfo>> {
    let mut agents = scan_agent_files(&env_dir.join("agents"), &ItemSource::Native)?;
    for plugin_dir in list_enabled_plugin_dirs(env_dir)? {
        let source = ItemSource::Plugin {
            marketplace: plugin_dir.marketplace,
            plugin: plugin_dir.plugin,
        };
        agents.extend(scan_agent_files(&plugin_dir.path.join("agents"), &source)?);
    }
    agents.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(agents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_and_description_from_frontmatter() {
        let fm = parse_frontmatter("---\nname: example\ndescription: An example\n---\n\n# Body\n");
        assert_eq!(fm.name.as_deref(), Some("example"));
        assert_eq!(fm.description.as_deref(), Some("An example"));
    }

    #[test]
    fn frontmatter_defaults_when_absent() {
        let fm = parse_frontmatter("# No frontmatter here\n");
        assert_eq!(fm.name, None);
        assert_eq!(fm.description, None);
    }

    #[test]
    fn lists_regular_skill_directory_as_not_built_in() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("skills/example");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: example\ndescription: An example skill\n---\n",
        )
        .unwrap();

        let skills = list_skills(dir.path()).unwrap();

        assert_eq!(
            skills,
            vec![SkillOrAgentInfo {
                id: "example".to_string(),
                name: "example".to_string(),
                description: "An example skill".to_string(),
                built_in: false,
                source: ItemSource::Native,
            }]
        );
    }

    #[cfg(unix)]
    #[test]
    fn lists_symlinked_skill_directory_as_built_in() {
        let dir = tempfile::tempdir().unwrap();
        let global_skill = dir.path().join("global-skills/inherited");
        fs::create_dir_all(&global_skill).unwrap();
        fs::write(
            global_skill.join("SKILL.md"),
            "---\nname: inherited\ndescription: x\n---\n",
        )
        .unwrap();

        let skills_dir = dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        std::os::unix::fs::symlink(&global_skill, skills_dir.join("inherited")).unwrap();

        let skills = list_skills(dir.path()).unwrap();

        assert_eq!(skills.len(), 1);
        assert!(skills[0].built_in);
    }

    #[test]
    fn returns_empty_list_when_skills_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(list_skills(dir.path()).unwrap(), Vec::new());
    }

    #[test]
    fn lists_agent_markdown_files_by_file_stem() {
        let dir = tempfile::tempdir().unwrap();
        let agents_dir = dir.path().join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        fs::write(
            agents_dir.join("jira.md"),
            "---\nname: jira\ndescription: Jira specialist\n---\n",
        )
        .unwrap();

        let agents = list_agents(dir.path()).unwrap();

        assert_eq!(
            agents,
            vec![SkillOrAgentInfo {
                id: "jira".to_string(),
                name: "jira".to_string(),
                description: "Jira specialist".to_string(),
                built_in: false,
                source: ItemSource::Native,
            }]
        );
    }

    #[test]
    fn returns_empty_list_when_agents_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(list_agents(dir.path()).unwrap(), Vec::new());
    }

    fn write_plugin_skill_fixture(dir: &Path, skill_dir: &str, agent_file: Option<(&str, &str)>) {
        fs::create_dir_all(dir.join("plugins")).unwrap();
        fs::write(
            dir.join("plugins/known_marketplaces.json"),
            r#"{"acme":{"source":{"source":"github","repo":"acme/marketplace"},"installLocation":"/x","lastUpdated":"2026-01-01T00:00:00Z"}}"#,
        )
        .unwrap();
        fs::create_dir_all(dir.join("plugins/marketplaces/acme/.claude-plugin")).unwrap();
        fs::write(
            dir.join("plugins/marketplaces/acme/.claude-plugin/marketplace.json"),
            r#"{"name":"acme","plugins":[{"name":"tool"}]}"#,
        )
        .unwrap();
        fs::write(
            dir.join("plugins/installed_plugins.json"),
            r#"{"version":2,"plugins":{"tool@acme":[{"scope":"user","installPath":"/x","version":"1.2.3"}]}}"#,
        )
        .unwrap();
        fs::write(
            dir.join("settings.json"),
            r#"{"enabledPlugins":{"tool@acme":true}}"#,
        )
        .unwrap();
        let plugin_dir = dir.join("plugins/cache/acme/tool/1.2.3");
        let plugin_skill_dir = plugin_dir.join("skills").join(skill_dir);
        fs::create_dir_all(&plugin_skill_dir).unwrap();
        fs::write(
            plugin_skill_dir.join("SKILL.md"),
            "---\nname: plugin-skill\ndescription: From a plugin\n---\n",
        )
        .unwrap();
        if let Some((file_name, contents)) = agent_file {
            let agents_dir = plugin_dir.join("agents");
            fs::create_dir_all(&agents_dir).unwrap();
            fs::write(agents_dir.join(file_name), contents).unwrap();
        }
    }

    #[test]
    fn lists_plugin_provided_skill_with_plugin_source() {
        let dir = tempfile::tempdir().unwrap();
        write_plugin_skill_fixture(dir.path(), "example", None);

        let skills = list_skills(dir.path()).unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "plugin-skill");
        assert_eq!(
            skills[0].source,
            ItemSource::Plugin {
                marketplace: "acme".to_string(),
                plugin: "tool".to_string()
            }
        );
    }

    #[test]
    fn lists_plugin_provided_agent_with_plugin_source() {
        let dir = tempfile::tempdir().unwrap();
        write_plugin_skill_fixture(
            dir.path(),
            "example",
            Some((
                "reviewer.md",
                "---\nname: reviewer\ndescription: From a plugin\n---\n",
            )),
        );

        let agents = list_agents(dir.path()).unwrap();

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "reviewer");
        assert_eq!(
            agents[0].source,
            ItemSource::Plugin {
                marketplace: "acme".to_string(),
                plugin: "tool".to_string()
            }
        );
    }
}
