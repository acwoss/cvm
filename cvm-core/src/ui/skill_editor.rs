use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillContent {
    pub name: String,
    pub description: String,
    pub body: String,
}

struct ParsedFile {
    frontmatter: Mapping,
    body: String,
}

fn parse_file(contents: &str) -> ParsedFile {
    let Some(rest) = contents.strip_prefix("---\n") else {
        return ParsedFile {
            frontmatter: Mapping::new(),
            body: contents.to_string(),
        };
    };
    let Some(end) = rest.find("\n---") else {
        return ParsedFile {
            frontmatter: Mapping::new(),
            body: contents.to_string(),
        };
    };
    let frontmatter: Mapping = serde_yaml::from_str(&rest[..end]).unwrap_or_default();
    let after = &rest[end + 4..];
    // `trim_start_matches` (não `strip_prefix`) é essencial aqui: remove
    // TODAS as quebras de linha entre o fechamento do frontmatter e o
    // corpo, não só uma. Com `strip_prefix('\n')` (remove só 1), cada
    // ciclo salvar->reler->salvar acumularia uma linha em branco extra no
    // início do corpo, já que `render_file` sempre escreve exatamente uma
    // linha em branco de separação (`---\n{yaml}---\n\n{body}`).
    let body = after.trim_start_matches('\n').to_string();
    ParsedFile { frontmatter, body }
}

fn render_file(frontmatter: &Mapping, body: &str) -> Result<String> {
    let yaml = serde_yaml::to_string(&Value::Mapping(frontmatter.clone()))
        .context("failed to serialize frontmatter")?;
    Ok(format!("---\n{yaml}---\n\n{body}"))
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() || id == "." || id == ".." || id.contains(['/', '\\']) {
        bail!("invalid name: '{id}'");
    }
    Ok(())
}

fn read_content(path: &Path, id_fallback: &str) -> Result<SkillContent> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let parsed = parse_file(&raw);
    let name = parsed
        .frontmatter
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(id_fallback)
        .to_string();
    let description = parsed
        .frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(SkillContent {
        name,
        description,
        body: parsed.body,
    })
}

fn write_content(path: &Path, content: &SkillContent) -> Result<()> {
    let raw = fs::read_to_string(path).unwrap_or_default();
    let mut parsed = parse_file(&raw);
    parsed.frontmatter.insert(
        Value::String("name".to_string()),
        Value::String(content.name.clone()),
    );
    parsed.frontmatter.insert(
        Value::String("description".to_string()),
        Value::String(content.description.clone()),
    );
    let rendered = render_file(&parsed.frontmatter, &content.body)?;
    fs::write(path, rendered).with_context(|| format!("failed to write {}", path.display()))
}

fn skill_path(env_dir: &Path, id: &str) -> Result<PathBuf> {
    validate_id(id)?;
    Ok(env_dir.join("skills").join(id).join("SKILL.md"))
}

pub fn read_skill_content(env_dir: &Path, id: &str) -> Result<SkillContent> {
    read_content(&skill_path(env_dir, id)?, id)
}

pub fn write_skill_content(env_dir: &Path, id: &str, content: &SkillContent) -> Result<()> {
    write_content(&skill_path(env_dir, id)?, content)
}

pub fn create_skill(env_dir: &Path, id: &str, name: &str, description: &str) -> Result<()> {
    let path = skill_path(env_dir, id)?;
    let dir = path
        .parent()
        .context("skill path must have a parent directory")?;
    if dir.exists() {
        bail!("skill '{id}' already exists");
    }
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let body = format!(
        "# {name}\n\nDescreva o que essa skill faz e quando usá-la.\n\n## Instruções\n\n1. Primeiro passo\n2. Segundo passo\n"
    );
    write_content(
        &path,
        &SkillContent {
            name: name.to_string(),
            description: description.to_string(),
            body,
        },
    )
}

pub fn delete_skill(env_dir: &Path, id: &str) -> Result<()> {
    let path = skill_path(env_dir, id)?;
    let dir = path
        .parent()
        .context("skill path must have a parent directory")?;
    fs::remove_dir_all(dir).with_context(|| format!("failed to remove {}", dir.display()))
}

fn agent_path(env_dir: &Path, id: &str) -> Result<PathBuf> {
    validate_id(id)?;
    Ok(env_dir.join("agents").join(format!("{id}.md")))
}

pub fn read_agent_content(env_dir: &Path, id: &str) -> Result<SkillContent> {
    read_content(&agent_path(env_dir, id)?, id)
}

pub fn write_agent_content(env_dir: &Path, id: &str, content: &SkillContent) -> Result<()> {
    write_content(&agent_path(env_dir, id)?, content)
}

pub fn create_agent(env_dir: &Path, id: &str, name: &str, description: &str) -> Result<()> {
    let path = agent_path(env_dir, id)?;
    if path.exists() {
        bail!("agent '{id}' already exists");
    }
    let dir = path
        .parent()
        .context("agent path must have a parent directory")?;
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let body = format!(
        "# {name}\n\nVocê é um agente especialista em {name}.\n\n## Capacidades\n\n- Capacidade um\n- Capacidade dois\n"
    );
    write_content(
        &path,
        &SkillContent {
            name: name.to_string(),
            description: description.to_string(),
            body,
        },
    )
}

pub fn delete_agent(env_dir: &Path, id: &str) -> Result<()> {
    let path = agent_path(env_dir, id)?;
    fs::remove_file(&path).with_context(|| format!("failed to remove {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_skill_writes_frontmatter_and_body_template() {
        let dir = tempfile::tempdir().unwrap();

        create_skill(dir.path(), "my-skill", "My Skill", "Does a thing").unwrap();

        let content = read_skill_content(dir.path(), "my-skill").unwrap();
        assert_eq!(content.name, "My Skill");
        assert_eq!(content.description, "Does a thing");
        assert!(content.body.contains("# My Skill"));
    }

    #[test]
    fn create_skill_fails_when_already_exists() {
        let dir = tempfile::tempdir().unwrap();
        create_skill(dir.path(), "my-skill", "My Skill", "x").unwrap();

        assert!(create_skill(dir.path(), "my-skill", "My Skill", "x").is_err());
    }

    #[test]
    fn write_skill_content_preserves_unknown_frontmatter_keys() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("skills/my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: old-name\ndescription: old desc\ncustomField: keep-me\n---\n\nold body\n",
        )
        .unwrap();

        write_skill_content(
            dir.path(),
            "my-skill",
            &SkillContent {
                name: "new-name".to_string(),
                description: "new desc".to_string(),
                body: "new body\n".to_string(),
            },
        )
        .unwrap();

        let raw = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(raw.contains("customField: keep-me"));
        assert!(raw.contains("name: new-name"));
        assert!(raw.contains("description: new desc"));
        assert!(raw.contains("new body"));
        assert!(!raw.contains("old body"));
    }

    #[test]
    fn write_skill_content_round_trip_does_not_accumulate_blank_lines() {
        let dir = tempfile::tempdir().unwrap();
        create_skill(dir.path(), "my-skill", "My Skill", "x").unwrap();

        // Simula três ciclos de "abrir o editor, salvar sem mudar nada" -
        // se `parse_file` só removesse uma quebra de linha por leitura, o
        // corpo cresceria uma linha em branco a cada volta.
        for _ in 0..3 {
            let content = read_skill_content(dir.path(), "my-skill").unwrap();
            write_skill_content(dir.path(), "my-skill", &content).unwrap();
        }

        let final_content = read_skill_content(dir.path(), "my-skill").unwrap();
        assert!(
            !final_content.body.starts_with('\n'),
            "body não deve acumular linhas em branco no início: {:?}",
            final_content.body
        );
    }

    #[test]
    fn delete_skill_removes_the_directory() {
        let dir = tempfile::tempdir().unwrap();
        create_skill(dir.path(), "my-skill", "My Skill", "x").unwrap();

        delete_skill(dir.path(), "my-skill").unwrap();

        assert!(!dir.path().join("skills/my-skill").exists());
    }

    #[cfg(unix)]
    #[test]
    fn delete_skill_on_a_symlink_never_touches_the_real_target() {
        let dir = tempfile::tempdir().unwrap();
        let real_skill = dir.path().join("global/real-skill");
        fs::create_dir_all(&real_skill).unwrap();
        fs::write(
            real_skill.join("SKILL.md"),
            "---\nname: real\n---\n\nreal content\n",
        )
        .unwrap();

        let skills_dir = dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        std::os::unix::fs::symlink(&real_skill, skills_dir.join("linked-skill")).unwrap();

        delete_skill(dir.path(), "linked-skill").unwrap();

        assert!(!skills_dir.join("linked-skill").exists());
        assert!(real_skill.join("SKILL.md").is_file());
        assert_eq!(
            fs::read_to_string(real_skill.join("SKILL.md")).unwrap(),
            "---\nname: real\n---\n\nreal content\n"
        );
    }

    #[test]
    fn skill_operations_reject_path_traversal_ids() {
        let dir = tempfile::tempdir().unwrap();
        assert!(create_skill(dir.path(), "../escape", "x", "x").is_err());
        assert!(read_skill_content(dir.path(), "..").is_err());
        assert!(delete_skill(dir.path(), "/etc").is_err());
    }

    #[test]
    fn create_agent_writes_frontmatter_and_body_template() {
        let dir = tempfile::tempdir().unwrap();

        create_agent(dir.path(), "my-agent", "My Agent", "Handles a thing").unwrap();

        let content = read_agent_content(dir.path(), "my-agent").unwrap();
        assert_eq!(content.name, "My Agent");
        assert_eq!(content.description, "Handles a thing");
        assert!(content.body.contains("# My Agent"));
    }

    #[test]
    fn write_agent_content_preserves_unknown_frontmatter_keys() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("agents")).unwrap();
        fs::write(
            dir.path().join("agents/my-agent.md"),
            "---\nname: old\ndescription: old\ntools: Read, Grep\nmodel: inherit\n---\n\nold body\n",
        )
        .unwrap();

        write_agent_content(
            dir.path(),
            "my-agent",
            &SkillContent {
                name: "new".to_string(),
                description: "new".to_string(),
                body: "new body\n".to_string(),
            },
        )
        .unwrap();

        let raw = fs::read_to_string(dir.path().join("agents/my-agent.md")).unwrap();
        assert!(raw.contains("tools: Read, Grep"));
        assert!(raw.contains("model: inherit"));
        assert!(raw.contains("name: new"));
    }

    #[test]
    fn delete_agent_removes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        create_agent(dir.path(), "my-agent", "My Agent", "x").unwrap();

        delete_agent(dir.path(), "my-agent").unwrap();

        assert!(!dir.path().join("agents/my-agent.md").exists());
    }
}
