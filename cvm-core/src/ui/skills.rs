use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillOrAgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub built_in: bool,
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

pub fn list_skills(env_dir: &Path) -> Result<Vec<SkillOrAgentInfo>> {
    let dir = env_dir.join("skills");
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(&dir)? {
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
            id,
        });
    }
    skills.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(skills)
}

pub fn list_agents(env_dir: &Path) -> Result<Vec<SkillOrAgentInfo>> {
    let dir = env_dir.join("agents");
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut agents = Vec::new();
    for entry in fs::read_dir(&dir)? {
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
            id,
        });
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
            }]
        );
    }

    #[test]
    fn returns_empty_list_when_agents_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(list_agents(dir.path()).unwrap(), Vec::new());
    }
}
