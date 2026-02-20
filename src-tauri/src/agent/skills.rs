//! Skills system — SKILL.md parser and registry.
//!
//! Ghost skills use the Agent Skills standard (Markdown + YAML frontmatter).
//! Skills define specialized knowledge, instructions, and tool schemas
//! that enhance the agent's capabilities.
//!
//! Skills are loaded from:
//! 1. Built-in skills (bundled with Ghost)
//! 2. User skills (~/.ghost/skills/)
//! 3. Project skills (.ghost/skills/ in workspace)
//!
//! Three-tier loading:
//! - Metadata (~100 tokens): name, description, triggers
//! - Instructions (<5000 tokens): full system prompt
//! - Resources: additional context files

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// A loaded skill definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique skill name (from filename or frontmatter).
    pub name: String,
    /// Short description.
    pub description: String,
    /// Trigger patterns — keywords/phrases that activate this skill.
    pub triggers: Vec<String>,
    /// The full instruction text (system prompt addition).
    pub instructions: String,
    /// Source path of the SKILL.md file.
    pub source: String,
    /// Whether this skill is currently enabled.
    pub enabled: bool,
    /// Tool schemas defined by this skill (optional).
    pub tools: Vec<SkillTool>,
}

/// A tool defined within a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// YAML frontmatter parsed from a SKILL.md file.
#[derive(Debug, Clone, Deserialize)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    triggers: Vec<String>,
    #[serde(default)]
    tools: Vec<SkillToolDef>,
    #[serde(default = "default_true")]
    enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct SkillToolDef {
    name: String,
    description: Option<String>,
    parameters: Option<serde_json::Value>,
}

fn default_true() -> bool {
    true
}

/// The skill registry — holds all loaded skills.
#[derive(Debug, Default)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Load skills from a directory (scans for SKILL.md files).
    pub fn load_from_directory(&mut self, dir: &Path) -> Vec<String> {
        let mut loaded = Vec::new();

        if !dir.exists() {
            tracing::debug!("Skills directory does not exist: {}", dir.display());
            return loaded;
        }

        // Scan for SKILL.md files (direct children and one level deep)

        // Direct SKILL.md files in dir
        if let Ok(skill) = parse_skill_file(&dir.join("SKILL.md")) {
            self.skills.insert(skill.name.clone(), skill.clone());
            loaded.push(skill.name);
        }

        // Subdirectory SKILL.md files
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_file = path.join("SKILL.md");
                    if skill_file.exists() {
                        match parse_skill_file(&skill_file) {
                            Ok(skill) => {
                                loaded.push(skill.name.clone());
                                self.skills.insert(skill.name.clone(), skill);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to parse skill {}: {}",
                                    skill_file.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        loaded
    }

    /// Get all enabled skills.
    #[allow(dead_code)]
    pub fn enabled_skills(&self) -> Vec<&Skill> {
        self.skills.values().filter(|s| s.enabled).collect()
    }

    /// Find skills matching a user query (by triggers).
    pub fn match_skills(&self, query: &str) -> Vec<&Skill> {
        let lower = query.to_lowercase();
        self.skills
            .values()
            .filter(|s| s.enabled && s.triggers.iter().any(|t| lower.contains(&t.to_lowercase())))
            .collect()
    }

    /// Get a skill by name.
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// Get all skills (including disabled).
    pub fn all_skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// Count of loaded skills.
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.skills.len()
    }

    /// Build system prompt additions from matched skills.
    pub fn build_prompt_for_query(&self, query: &str) -> String {
        let matched = self.match_skills(query);
        if matched.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("\n\n## Active Skills\n\n");
        for skill in matched {
            prompt.push_str(&format!("### {}\n{}\n\n", skill.name, skill.instructions));
        }
        prompt
    }
}

/// Parse a SKILL.md file into a Skill struct.
fn parse_skill_file(path: &Path) -> Result<Skill, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let (frontmatter, body) = parse_frontmatter(&content)?;

    let name = frontmatter.name.unwrap_or_else(|| {
        // Derive name from parent directory
        path.parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".into())
    });

    let tools = frontmatter
        .tools
        .into_iter()
        .map(|t| SkillTool {
            name: t.name,
            description: t.description.unwrap_or_default(),
            parameters: t
                .parameters
                .unwrap_or_else(|| serde_json::json!({"type": "object", "properties": {}})),
        })
        .collect();

    Ok(Skill {
        name,
        description: frontmatter.description.unwrap_or_default(),
        triggers: frontmatter.triggers,
        instructions: body,
        source: path.to_string_lossy().to_string(),
        enabled: frontmatter.enabled,
        tools,
    })
}

/// Parse YAML frontmatter from a Markdown file.
///
/// Expects the format:
/// ```text
/// ---
/// name: my-skill
/// description: Does something
/// triggers: [keyword1, keyword2]
/// ---
/// # Instructions
/// ...
/// ```
fn parse_frontmatter(content: &str) -> Result<(SkillFrontmatter, String), String> {
    let trimmed = content.trim();

    if !trimmed.starts_with("---") {
        // No frontmatter — treat entire content as instructions
        return Ok((
            SkillFrontmatter {
                name: None,
                description: None,
                triggers: Vec::new(),
                tools: Vec::new(),
                enabled: true,
            },
            trimmed.to_string(),
        ));
    }

    // Find the closing ---
    let after_first = &trimmed[3..];
    let closing = after_first
        .find("\n---")
        .ok_or("Missing closing --- in frontmatter")?;

    let yaml_str = &after_first[..closing].trim();
    let body = after_first[closing + 4..].trim().to_string();

    let frontmatter: SkillFrontmatter =
        serde_yaml::from_str(yaml_str).map_err(|e| format!("Invalid YAML frontmatter: {}", e))?;

    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = r#"---
name: test-skill
description: A test skill
triggers: [test, testing]
---
# Instructions

Do the thing.
"#;
        let (fm, body) = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name, Some("test-skill".into()));
        assert_eq!(fm.description, Some("A test skill".into()));
        assert_eq!(fm.triggers, vec!["test", "testing"]);
        assert!(body.contains("Do the thing"));
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "# Just Instructions\n\nDo stuff.";
        let (fm, body) = parse_frontmatter(content).unwrap();
        assert!(fm.name.is_none());
        assert!(body.contains("Do stuff"));
    }

    #[test]
    fn test_parse_frontmatter_with_tools() {
        let content = r#"---
name: git-skill
description: Git operations
triggers: [git, commit, branch]
tools:
  - name: git_status
    description: Get git status
  - name: git_commit
    description: Create a commit
    parameters:
      type: object
      properties:
        message:
          type: string
      required: [message]
---
# Git Skill

Help with git operations.
"#;
        let (fm, _body) = parse_frontmatter(content).unwrap();
        assert_eq!(fm.tools.len(), 2);
        assert_eq!(fm.tools[0].name, "git_status");
        assert!(fm.tools[1].parameters.is_some());
    }

    #[test]
    fn test_skill_registry() {
        let registry = SkillRegistry::new();
        assert_eq!(registry.count(), 0);
        assert!(registry.enabled_skills().is_empty());
    }

    #[test]
    fn test_match_skills() {
        let mut registry = SkillRegistry::new();
        registry.skills.insert(
            "git".into(),
            Skill {
                name: "git".into(),
                description: "Git helper".into(),
                triggers: vec!["git".into(), "commit".into()],
                instructions: "Help with git".into(),
                source: "test".into(),
                enabled: true,
                tools: vec![],
            },
        );

        let matched = registry.match_skills("How do I git commit?");
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "git");

        let matched = registry.match_skills("What is the weather?");
        assert!(matched.is_empty());
    }

    #[test]
    fn test_build_prompt() {
        let mut registry = SkillRegistry::new();
        registry.skills.insert(
            "rust".into(),
            Skill {
                name: "rust".into(),
                description: "Rust programming".into(),
                triggers: vec!["rust".into(), "cargo".into()],
                instructions: "Use idiomatic Rust patterns.".into(),
                source: "test".into(),
                enabled: true,
                tools: vec![],
            },
        );

        let prompt = registry.build_prompt_for_query("help me with rust code");
        assert!(prompt.contains("rust"));
        assert!(prompt.contains("idiomatic Rust"));

        let prompt = registry.build_prompt_for_query("make me a sandwich");
        assert!(prompt.is_empty());
    }
}
