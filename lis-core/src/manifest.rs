//! LIS Project Manifest (lis.toml) Parser
//!
//! Handles parsing and validation of lis.toml project configuration files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// LIS project manifest (lis.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Package metadata
    pub package: Package,

    /// Project dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,

    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,
}

/// Package metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub name: String,

    /// Package version (semver)
    pub version: String,

    /// Package authors
    #[serde(default)]
    pub authors: Vec<String>,

    /// Package description
    #[serde(default)]
    pub description: Option<String>,

    /// License identifier (SPDX)
    #[serde(default)]
    pub license: Option<String>,

    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version string: `"2026.1.16"`
    Version(String),

    /// Detailed dependency specification
    Detailed(DetailedDependency),
}

/// Detailed dependency with path or git source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDependency {
    /// Version requirement (for registry dependencies)
    #[serde(default)]
    pub version: Option<String>,

    /// Local path (relative to manifest)
    #[serde(default)]
    pub path: Option<String>,

    /// Git repository URL
    #[serde(default)]
    pub git: Option<String>,

    /// Git branch name
    #[serde(default)]
    pub branch: Option<String>,

    /// Git tag name
    #[serde(default)]
    pub tag: Option<String>,

    /// Git commit hash
    #[serde(default)]
    pub rev: Option<String>,

    /// Optional dependency flag
    #[serde(default)]
    pub optional: bool,
}

/// Build configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Entry point file (default: src/main.lis)
    #[serde(default = "default_entry")]
    pub entry: String,

    /// Target SIL mode (default: SIL-128)
    #[serde(default = "default_target_mode")]
    pub target_mode: String,

    /// Output directory (default: target)
    #[serde(default = "default_output")]
    pub output: String,

    /// Whether to include debug info
    #[serde(default)]
    pub debug: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            entry: default_entry(),
            target_mode: default_target_mode(),
            output: default_output(),
            debug: false,
        }
    }
}

fn default_entry() -> String {
    "src/main.lis".to_string()
}

fn default_target_mode() -> String {
    "SIL-128".to_string()
}

fn default_output() -> String {
    "target".to_string()
}

impl Manifest {
    /// Parse a manifest from TOML string
    pub fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| Error::Manifest(format!("Failed to parse lis.toml: {}", e)))
    }

    /// Load manifest from a file path
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Manifest(format!("Failed to read {}: {}", path.display(), e)))?;
        Self::from_str(&content)
    }

    /// Find and load manifest by searching up from current directory
    pub fn find_and_load(start_dir: &Path) -> Result<(Self, PathBuf)> {
        let mut current = start_dir.to_path_buf();

        loop {
            let manifest_path = current.join("lis.toml");
            if manifest_path.exists() {
                let manifest = Self::from_file(&manifest_path)?;
                return Ok((manifest, current));
            }

            if !current.pop() {
                return Err(Error::Manifest(
                    "No lis.toml found in current directory or any parent directory".to_string()
                ));
            }
        }
    }

    /// Serialize manifest to TOML string
    pub fn to_string(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| Error::Manifest(format!("Failed to serialize manifest: {}", e)))
    }

    /// Get the resolved entry point path
    pub fn entry_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.build.entry)
    }

    /// Get the resolved output directory
    pub fn output_dir(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.build.output)
    }

    /// Get resolved dependency paths
    pub fn resolve_dependencies(&self, project_root: &Path) -> Result<HashMap<String, PathBuf>> {
        let mut resolved = HashMap::new();

        for (name, dep) in &self.dependencies {
            match dep {
                Dependency::Version(_) => {
                    // Registry dependencies not yet supported
                    return Err(Error::Manifest(format!(
                        "Registry dependencies not yet supported. Use path for dependency '{}'", name
                    )));
                }
                Dependency::Detailed(detailed) => {
                    if let Some(path) = &detailed.path {
                        let dep_path = project_root.join(path);
                        if !dep_path.exists() {
                            return Err(Error::Manifest(format!(
                                "Dependency '{}' path does not exist: {}", name, dep_path.display()
                            )));
                        }
                        resolved.insert(name.clone(), dep_path);
                    } else if detailed.git.is_some() {
                        return Err(Error::Manifest(format!(
                            "Git dependencies not yet supported for '{}'", name
                        )));
                    } else {
                        return Err(Error::Manifest(format!(
                            "Dependency '{}' has no source (path or git)", name
                        )));
                    }
                }
            }
        }

        Ok(resolved)
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            package: Package {
                name: "my-project".to_string(),
                version: "2026.1.16".to_string(),
                authors: Vec::new(),
                description: None,
                license: None,
                repository: None,
            },
            dependencies: HashMap::new(),
            build: BuildConfig::default(),
        }
    }
}

/// Create a new manifest for project scaffolding
pub fn create_manifest(name: &str, authors: Vec<String>) -> Manifest {
    Manifest {
        package: Package {
            name: name.to_string(),
            version: "2026.1.16".to_string(),
            authors,
            description: Some(format!("A LIS project: {}", name)),
            license: Some("MIT".to_string()),
            repository: None,
        },
        dependencies: HashMap::new(),
        build: BuildConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[package]
name = "test-project"
version = "2026.1.16"
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "test-project");
        assert_eq!(manifest.package.version, "2026.1.16");
        assert_eq!(manifest.build.entry, "src/main.lis");
    }

    #[test]
    fn test_parse_full_manifest() {
        let toml = r#"
[package]
name = "neural-network"
version = "1.2.3"
authors = ["Alice <alice@example.com>"]
description = "A neural network library"
license = "MIT"

[dependencies]
utils = { path = "./libs/utils" }
math = { path = "../shared/math" }

[build]
entry = "src/lib.lis"
target_mode = "SIL-64"
output = "dist"
debug = true
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "neural-network");
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.build.entry, "src/lib.lis");
        assert_eq!(manifest.build.target_mode, "SIL-64");
        assert!(manifest.build.debug);
    }

    #[test]
    fn test_create_manifest() {
        let manifest = create_manifest("my-app", vec!["Dev <dev@example.com>".to_string()]);
        assert_eq!(manifest.package.name, "my-app");
        assert_eq!(manifest.package.authors.len(), 1);
    }
}
