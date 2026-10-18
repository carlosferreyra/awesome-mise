//! Shared catalog loading and validation for the standalone maintenance scripts.
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub categories: Vec<Category>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub name: String,
    pub slug: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub description: String,
    pub url: String,
    pub notes: String,
    #[serde(default, deserialize_with = "optional_string")]
    pub config: Option<String>,
    #[serde(default, deserialize_with = "optional_string")]
    pub usage: Option<String>,
}

fn optional_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

pub fn repo_root() -> Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()?
        .parent()
        .ok_or("Missing repository parent")?
        .to_owned())
}

pub fn load(root: &Path) -> Result<Catalog> {
    let catalog: Catalog = serde_json::from_str(&fs::read_to_string(root.join("catalog.json"))?)?;
    validate(&catalog, root)?;
    Ok(catalog)
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn slug(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

pub fn validate(catalog: &Catalog, root: &Path) -> Result<()> {
    require(
        !catalog.categories.is_empty(),
        "Categories must be a non-empty list",
    )?;
    let mut slugs = BTreeSet::new();
    let mut names = BTreeSet::new();
    for category in &catalog.categories {
        require(!category.name.trim().is_empty(), "Missing category name")?;
        require(slug(&category.slug), "Invalid slug")?;
        require(slugs.insert(&category.slug), "Duplicate category slug")?;
        require(!category.entries.is_empty(), "Category needs entries")?;
        for entry in &category.entries {
            require(
                [
                    &entry.name,
                    &entry.kind,
                    &entry.description,
                    &entry.url,
                    &entry.notes,
                ]
                .iter()
                .all(|s| !s.trim().is_empty()),
                "Entry values must be non-empty strings",
            )?;
            require(
                entry
                    .config
                    .iter()
                    .chain(entry.usage.iter())
                    .all(|s| !s.trim().is_empty()),
                "Entry values must be non-empty strings",
            )?;
            require(
                ["tool", "integration", "recipe", "learning"].contains(&entry.kind.as_str()),
                "Unknown resource type",
            )?;
            require(
                names.insert(entry.name.to_lowercase()),
                "Duplicate entry name",
            )?;
            let url = url::Url::parse(&entry.url)?;
            require(
                url.scheme() == "https"
                    && url.host_str().is_some()
                    && !entry.url.chars().any(char::is_whitespace),
                "Expected HTTPS source URL",
            )?;
            if entry.kind == "recipe" {
                let config = entry
                    .config
                    .as_deref()
                    .ok_or("Recipe needs config and usage")?;
                require(entry.usage.is_some(), "Recipe needs config and usage")?;
                let parts: Vec<_> = config.split('/').collect();
                require(
                    parts.len() == 3
                        && parts[0] == "examples"
                        && !parts[1].is_empty()
                        && parts[1]
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                        && parts[2] == "mise.toml",
                    "Invalid recipe path",
                )?;
                let path = root.join(config).canonicalize()?;
                require(
                    path.starts_with(root.canonicalize()?),
                    "Recipe must stay inside repository",
                )?;
                fs::read_to_string(path)?.parse::<toml::Table>()?;
            } else {
                require(
                    entry.config.is_none() && entry.usage.is_none(),
                    "Only recipes have config and usage",
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn fixture() -> (tempfile::TempDir, Value) {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("examples/sample")).unwrap();
        fs::write(
            root.path().join("examples/sample/mise.toml"),
            "[tasks.hello]\nrun = 'echo hello'\n",
        )
        .unwrap();
        let data = json!({"categories": [{"name": "Recipes", "slug": "recipes", "entries": [{
            "name": "Sample", "type": "recipe", "description": "Example.",
            "url": "https://example.com", "notes": "Notes.",
            "config": "examples/sample/mise.toml", "usage": "mise run hello"
        }]}]});
        (root, data)
    }

    fn check(data: Value, root: &Path) -> Result<()> {
        validate(&serde_json::from_value(data)?, root)
    }

    #[test]
    fn duplicate_names_rejected() {
        let (root, mut data) = fixture();
        let entry = data["categories"][0]["entries"][0].clone();
        data["categories"][0]["entries"]
            .as_array_mut()
            .unwrap()
            .push(entry);
        assert!(
            check(data, root.path())
                .unwrap_err()
                .to_string()
                .contains("Duplicate entry")
        );
    }

    #[test]
    fn invalid_url_rejected() {
        let (root, mut data) = fixture();
        data["categories"][0]["entries"][0]["url"] = json!("javascript:alert(1)");
        assert!(
            check(data, root.path())
                .unwrap_err()
                .to_string()
                .contains("HTTPS")
        );
    }

    #[test]
    fn recipe_requires_config() {
        let (root, mut data) = fixture();
        data["categories"][0]["entries"][0]
            .as_object_mut()
            .unwrap()
            .remove("config");
        assert!(
            check(data, root.path())
                .unwrap_err()
                .to_string()
                .contains("Recipe needs")
        );
    }

    #[test]
    fn recipe_path_cannot_escape() {
        let (root, mut data) = fixture();
        data["categories"][0]["entries"][0]["config"] = json!("../mise.toml");
        assert!(
            check(data, root.path())
                .unwrap_err()
                .to_string()
                .contains("recipe path")
        );
    }

    #[test]
    fn explicit_null_optional_field_rejected() {
        let (root, mut data) = fixture();
        data["categories"][0]["entries"][0]["config"] = Value::Null;
        assert!(check(data, root.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_config_cannot_escape() {
        let (root, data) = fixture();
        let outside = tempfile::tempdir().unwrap();
        let source = outside.path().join("mise.toml");
        fs::write(&source, "[tasks.hello]\nrun = 'echo hello'\n").unwrap();
        let config = root.path().join("examples/sample/mise.toml");
        fs::remove_file(&config).unwrap();
        std::os::unix::fs::symlink(source, config).unwrap();
        assert!(
            check(data, root.path())
                .unwrap_err()
                .to_string()
                .contains("inside repository")
        );
    }

    #[test]
    fn malformed_toml_rejected() {
        let (root, data) = fixture();
        fs::write(root.path().join("examples/sample/mise.toml"), "[broken").unwrap();
        assert!(check(data, root.path()).is_err());
    }

    #[test]
    fn unknown_fields_and_null_strings_rejected() {
        let (root, mut data) = fixture();
        data["extra"] = json!(true);
        assert!(check(data, root.path()).is_err());
        let (_, mut data) = fixture();
        data["categories"][0]["entries"][0]["description"] = Value::Null;
        assert!(check(data, root.path()).is_err());
    }
}
