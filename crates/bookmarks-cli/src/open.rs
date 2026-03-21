use anyhow::{Context, Result};

use crate::config::Config;

pub fn resolve_uri<'a>(link: &str, config: &'a Config) -> Result<&'a str> {
    if let Some(alias_target) = config.aliases.get(link) {
        return config
            .links
            .get(alias_target)
            .map(String::as_str)
            .with_context(|| {
                format!("alias '{link}' points to '{alias_target}' which is not in [links]")
            });
    }

    config
        .links
        .get(link)
        .map(String::as_str)
        .with_context(|| format!("'{link}' not found in [aliases] or [links]"))
}

fn open_it(link: &str) -> Result<()> {
    open::that(link).with_context(|| format!("failed to open {link}"))?;
    println!("opening {link}...");
    Ok(())
}

pub fn expand_groups<'a>(links: &'a [String], config: &'a Config) -> Vec<&'a str> {
    let mut expanded = Vec::new();
    for link in links {
        if let Some(group_items) = config.groups.get(link.as_str()) {
            expanded.extend(group_items.iter().map(|s| s.as_str()));
        } else {
            expanded.push(link.as_str());
        }
    }
    expanded
}

pub fn open_links(links: &[String], config: &Config) -> Result<()> {
    let expanded_links = expand_groups(links, config);

    for link in expanded_links {
        match resolve_uri(link, config) {
            Ok(uri) => {
                if let Err(e) = open_it(uri) {
                    eprintln!("[dkdc] failed to open {link}: {e}");
                }
            }
            Err(e) => {
                eprintln!("[dkdc] skipping {link}: {e}");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_config() -> Config {
        let mut aliases = HashMap::new();
        aliases.insert("gh".to_string(), "github".to_string());
        aliases.insert("g".to_string(), "google".to_string());

        let mut links = HashMap::new();
        links.insert("github".to_string(), "https://github.com".to_string());
        links.insert("google".to_string(), "https://google.com".to_string());
        links.insert("rust".to_string(), "https://rust-lang.org".to_string());

        let mut groups = HashMap::new();
        groups.insert(
            "dev".to_string(),
            vec!["gh".to_string(), "rust".to_string()],
        );

        Config {
            aliases,
            links,
            groups,
        }
    }

    #[test]
    fn test_alias_resolves_to_uri() {
        let config = test_config();
        let uri = resolve_uri("gh", &config).unwrap();
        assert_eq!(uri, "https://github.com");
    }

    #[test]
    fn test_link_resolves_to_uri() {
        let config = test_config();
        let uri = resolve_uri("rust", &config).unwrap();
        assert_eq!(uri, "https://rust-lang.org");
    }

    #[test]
    fn test_alias_target_as_link_resolves() {
        let config = test_config();
        // "github" is both an alias target and a link name
        let uri = resolve_uri("github", &config).unwrap();
        assert_eq!(uri, "https://github.com");
    }

    #[test]
    fn test_unknown_link_errors() {
        let config = test_config();
        let result = resolve_uri("unknown", &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_alias_to_missing_link_errors() {
        let mut config = test_config();
        config
            .aliases
            .insert("broken".to_string(), "nonexistent".to_string());

        let result = resolve_uri("broken", &config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("broken"));
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn test_expand_group() {
        let config = test_config();
        let links = vec!["dev".to_string()];
        let expanded = expand_groups(&links, &config);
        assert_eq!(expanded, vec!["gh", "rust"]);
    }

    #[test]
    fn test_mixed_groups_and_links() {
        let config = test_config();
        let links = vec!["dev".to_string(), "google".to_string()];
        let expanded = expand_groups(&links, &config);
        assert_eq!(expanded, vec!["gh", "rust", "google"]);
    }
}
