use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

const DEFAULT_EDITOR: &str = "vi";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    #[serde(default)]
    pub links: HashMap<String, String>,
    #[serde(default)]
    pub groups: HashMap<String, Vec<String>>,
}

pub const DEFAULT_CONFIG: &str = r#"# bookmarks config file
[aliases]
gh = "github"
li = "linkedin"
[links]
github = "https://github.com"
linkedin = "https://linkedin.com"
[groups]
socials = ["gh", "linkedin"]
"#;

impl Config {
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        for (alias, target) in &self.aliases {
            if !self.links.contains_key(target) {
                warnings.push(format!(
                    "alias '{alias}' points to '{target}' which is not in [links]"
                ));
            }
        }

        for (group, entries) in &self.groups {
            for entry in entries {
                if !self.aliases.contains_key(entry) && !self.links.contains_key(entry) {
                    warnings.push(format!(
                        "group '{group}' contains '{entry}' which is not in [aliases] or [links]"
                    ));
                }
            }
        }

        warnings
    }

    /// Rename a link key and cascade to all aliases that target it.
    pub fn rename_link(&mut self, old: &str, new: &str) -> Result<()> {
        let url = self
            .links
            .remove(old)
            .with_context(|| format!("link '{old}' not found"))?;
        self.links.insert(new.to_string(), url);

        // Update aliases that point to the old name
        for target in self.aliases.values_mut() {
            if target == old {
                *target = new.to_string();
            }
        }

        Ok(())
    }

    /// Rename an alias key and cascade to all groups that reference it.
    pub fn rename_alias(&mut self, old: &str, new: &str) -> Result<()> {
        let target = self
            .aliases
            .remove(old)
            .with_context(|| format!("alias '{old}' not found"))?;
        self.aliases.insert(new.to_string(), target);

        // Update group entries that reference the old name
        for entries in self.groups.values_mut() {
            for entry in entries.iter_mut() {
                if entry == old {
                    *entry = new.to_string();
                }
            }
        }

        Ok(())
    }
}

pub fn edit_config(config_path: &Path) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| DEFAULT_EDITOR.to_string());

    println!("Opening {} with {}...", config_path.display(), editor);

    let status = std::process::Command::new(&editor)
        .arg(config_path)
        .status()
        .with_context(|| format!("Editor {editor} not found in PATH"))?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    Ok(())
}

fn print_section<V>(
    name: &str,
    map: &HashMap<String, V>,
    format_value: impl Fn(&V) -> Cow<'_, str>,
) {
    if map.is_empty() {
        return;
    }

    println!("{name}:");
    println!();

    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_unstable_by_key(|(k, _)| k.as_str());

    let max_key_len = entries.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

    for (key, value) in entries {
        println!("• {key:<max_key_len$} | {}", format_value(value));
    }

    println!();
}

pub fn print_config(config: &Config) {
    print_section("aliases", &config.aliases, |v| Cow::Borrowed(v));
    print_section("links", &config.links, |v| Cow::Borrowed(v));
    print_section("groups", &config.groups, |v| {
        Cow::Owned(format!("[{}]", v.join(", ")))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let toml = r#"
[aliases]
gh = "github"

[links]
github = "https://github.com"

[groups]
dev = ["gh"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.aliases.get("gh"), Some(&"github".to_string()));
        assert_eq!(
            config.links.get("github"),
            Some(&"https://github.com".to_string())
        );
        assert_eq!(config.groups.get("dev"), Some(&vec!["gh".to_string()]));
    }

    #[test]
    fn test_parse_empty_config() {
        let toml = "";
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.aliases.is_empty());
        assert!(config.links.is_empty());
        assert!(config.groups.is_empty());
    }

    #[test]
    fn test_parse_partial_config() {
        let toml = r#"
[links]
rust = "https://rust-lang.org"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.aliases.is_empty());
        assert_eq!(
            config.links.get("rust"),
            Some(&"https://rust-lang.org".to_string())
        );
        assert!(config.groups.is_empty());
    }

    #[test]
    fn test_config_roundtrip() {
        let mut config = Config::default();
        config.aliases.insert("a".to_string(), "b".to_string());
        config
            .links
            .insert("b".to_string(), "https://example.com".to_string());
        config.groups.insert("g".to_string(), vec!["a".to_string()]);

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.aliases, deserialized.aliases);
        assert_eq!(config.links, deserialized.links);
        assert_eq!(config.groups, deserialized.groups);
    }

    #[test]
    fn test_default_config_parses() {
        let config: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert!(!config.aliases.is_empty());
        assert!(!config.links.is_empty());
        assert!(!config.groups.is_empty());
    }

    #[test]
    fn test_valid_config_has_no_warnings() {
        let config: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert!(config.validate().is_empty());
    }

    #[test]
    fn test_broken_alias_target_warns() {
        let toml = r#"
[aliases]
broken = "nonexistent"

[links]
real = "https://example.com"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let warnings = config.validate();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("nonexistent"));
    }

    #[test]
    fn test_rename_link_cascades_aliases() {
        let toml = r#"
[aliases]
gh = "github"
g = "github"

[links]
github = "https://github.com"
"#;
        let mut config: Config = toml::from_str(toml).unwrap();
        config.rename_link("github", "gh-link").unwrap();
        assert!(config.links.contains_key("gh-link"));
        assert!(!config.links.contains_key("github"));
        assert_eq!(config.aliases.get("gh"), Some(&"gh-link".to_string()));
        assert_eq!(config.aliases.get("g"), Some(&"gh-link".to_string()));
    }

    #[test]
    fn test_rename_alias_cascades_groups() {
        let toml = r#"
[aliases]
gh = "github"

[links]
github = "https://github.com"

[groups]
dev = ["gh"]
all = ["gh", "other"]
"#;
        let mut config: Config = toml::from_str(toml).unwrap();
        config.rename_alias("gh", "github-alias").unwrap();
        assert!(config.aliases.contains_key("github-alias"));
        assert!(!config.aliases.contains_key("gh"));
        assert_eq!(
            config.groups.get("dev"),
            Some(&vec!["github-alias".to_string()])
        );
        let all = config.groups.get("all").unwrap();
        assert!(all.contains(&"github-alias".to_string()));
        assert!(all.contains(&"other".to_string()));
    }

    #[test]
    fn test_rename_nonexistent_link_errors() {
        let mut config = Config::default();
        assert!(config.rename_link("nope", "new").is_err());
    }

    #[test]
    fn test_rename_nonexistent_alias_errors() {
        let mut config = Config::default();
        assert!(config.rename_alias("nope", "new").is_err());
    }

    #[test]
    fn test_broken_group_entry_warns() {
        let toml = r#"
[links]
real = "https://example.com"

[groups]
dev = ["real", "ghost"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let warnings = config.validate();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("ghost"));
    }
}
