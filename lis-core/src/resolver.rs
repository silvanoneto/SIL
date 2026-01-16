//! Module Resolver for LIS
//!
//! Resolves module paths, loads source files, and builds the dependency graph.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{Item, Program};
use crate::error::{Error, Result};
use crate::lexer::Lexer;
use crate::manifest::Manifest;
use crate::parser::Parser;

/// A resolved module with its source and AST
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// Module path (e.g., ["utils", "math"])
    pub path: Vec<String>,

    /// Canonical module name (e.g., "utils::math")
    pub canonical_name: String,

    /// Source file path
    pub file_path: PathBuf,

    /// Parsed AST
    pub ast: Program,

    /// Public symbols exported by this module
    pub exports: HashSet<String>,

    /// Modules this module depends on
    pub dependencies: Vec<Vec<String>>,
}

/// Module resolver that handles loading and dependency resolution
pub struct ModuleResolver {
    /// Project root directory (where lis.toml is)
    project_root: PathBuf,

    /// Project manifest
    manifest: Manifest,

    /// Resolved modules by canonical name
    modules: HashMap<String, ResolvedModule>,

    /// Dependency paths from manifest
    dependency_paths: HashMap<String, PathBuf>,

    /// Modules currently being resolved (for cycle detection)
    resolving: HashSet<String>,
}

impl ModuleResolver {
    /// Create a new resolver from a manifest and project root
    pub fn new(manifest: Manifest, project_root: PathBuf) -> Result<Self> {
        let dependency_paths = manifest.resolve_dependencies(&project_root)?;

        Ok(Self {
            project_root,
            manifest,
            modules: HashMap::new(),
            dependency_paths,
            resolving: HashSet::new(),
        })
    }

    /// Create a resolver by finding the manifest from current directory
    pub fn from_current_dir() -> Result<Self> {
        let current_dir = std::env::current_dir()
            .map_err(|e| Error::IoError { message: e.to_string() })?;
        let (manifest, project_root) = Manifest::find_and_load(&current_dir)?;
        Self::new(manifest, project_root)
    }

    /// Resolve all modules starting from the entry point
    pub fn resolve_all(&mut self) -> Result<Vec<ResolvedModule>> {
        // Start from entry point
        let entry_path = self.manifest.entry_path(&self.project_root);

        if !entry_path.exists() {
            return Err(Error::ModuleError {
                message: format!("Entry point not found: {}", entry_path.display()),
                path: Some(entry_path.to_string_lossy().to_string()),
            });
        }

        // Resolve entry module
        self.resolve_file(&entry_path, vec!["main".to_string()])?;

        // Return all modules in dependency order
        self.topological_sort()
    }

    /// Resolve a single file as a module
    fn resolve_file(&mut self, file_path: &Path, module_path: Vec<String>) -> Result<String> {
        let canonical_name = module_path.join("::");

        // Check if already resolved
        if self.modules.contains_key(&canonical_name) {
            return Ok(canonical_name);
        }

        // Check for cycles
        if self.resolving.contains(&canonical_name) {
            return Err(Error::ModuleError {
                message: format!("Cyclic dependency detected: {}", canonical_name),
                path: Some(file_path.to_string_lossy().to_string()),
            });
        }

        self.resolving.insert(canonical_name.clone());

        // Read and parse source
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| Error::IoError { message: format!("{}: {}", file_path.display(), e) })?;

        let tokens = Lexer::new(&source).tokenize_with_spans()?;
        let ast = Parser::new(tokens).parse()?;

        // Collect exports (public items)
        let exports = self.collect_exports(&ast);

        // Collect dependencies (use statements)
        let dependencies = self.collect_dependencies(&ast);

        // Resolve each dependency
        for dep_path in &dependencies {
            self.resolve_module_path(dep_path, file_path)?;
        }

        self.resolving.remove(&canonical_name);

        // Store resolved module
        let module = ResolvedModule {
            path: module_path,
            canonical_name: canonical_name.clone(),
            file_path: file_path.to_path_buf(),
            ast,
            exports,
            dependencies,
        };

        self.modules.insert(canonical_name.clone(), module);

        Ok(canonical_name)
    }

    /// Resolve a module path from a use statement
    fn resolve_module_path(&mut self, path: &[String], from_file: &Path) -> Result<String> {
        if path.is_empty() {
            return Err(Error::ModuleError {
                message: "Empty module path".to_string(),
                path: Some(from_file.to_string_lossy().to_string()),
            });
        }

        let first_segment = &path[0];

        // Check if it's a dependency from manifest
        if let Some(dep_path) = self.dependency_paths.get(first_segment) {
            // Dependency module
            let mut module_file = dep_path.clone();

            if path.len() > 1 {
                // Submodule: dependency_path/src/sub/module.lis
                module_file = module_file.join("src");
                for segment in &path[1..] {
                    module_file = module_file.join(segment);
                }
            } else {
                // Root: dependency_path/src/lib.lis
                module_file = module_file.join("src").join("lib.lis");
            }

            // Try .lis extension
            if !module_file.exists() {
                module_file.set_extension("lis");
            }

            // Try mod.lis pattern
            if !module_file.exists() {
                module_file = dep_path.join("src");
                for segment in &path[1..] {
                    module_file = module_file.join(segment);
                }
                module_file = module_file.join("mod.lis");
            }

            if !module_file.exists() {
                return Err(Error::ModuleError {
                    message: format!("Module not found: {}", path.join("::")),
                    path: Some(module_file.to_string_lossy().to_string()),
                });
            }

            return self.resolve_file(&module_file, path.to_vec());
        }

        // Local module relative to src/
        let from_dir = from_file.parent().unwrap_or(Path::new("."));
        let mut module_file = self.project_root.join("src");

        for segment in path {
            module_file = module_file.join(segment);
        }

        // Try direct file: src/module.lis
        let direct_file = module_file.with_extension("lis");
        if direct_file.exists() {
            return self.resolve_file(&direct_file, path.to_vec());
        }

        // Try directory with mod.lis: src/module/mod.lis
        let mod_file = module_file.join("mod.lis");
        if mod_file.exists() {
            return self.resolve_file(&mod_file, path.to_vec());
        }

        // Try relative to current file
        let mut relative_file = from_dir.to_path_buf();
        for segment in path {
            relative_file = relative_file.join(segment);
        }

        let relative_direct = relative_file.with_extension("lis");
        if relative_direct.exists() {
            return self.resolve_file(&relative_direct, path.to_vec());
        }

        let relative_mod = relative_file.join("mod.lis");
        if relative_mod.exists() {
            return self.resolve_file(&relative_mod, path.to_vec());
        }

        Err(Error::ModuleError {
            message: format!("Module not found: {}", path.join("::")),
            path: Some(direct_file.to_string_lossy().to_string()),
        })
    }

    /// Collect public exports from a program
    fn collect_exports(&self, program: &Program) -> HashSet<String> {
        let mut exports = HashSet::new();

        for item in &program.items {
            match item {
                Item::Function { name, is_pub, .. } if *is_pub => {
                    exports.insert(name.clone());
                }
                Item::Transform { name, is_pub, .. } if *is_pub => {
                    exports.insert(name.clone());
                }
                Item::TypeAlias { name, is_pub, .. } if *is_pub => {
                    exports.insert(name.clone());
                }
                Item::Use(use_stmt) if use_stmt.is_pub => {
                    // Re-exports
                    if let Some(alias) = &use_stmt.alias {
                        exports.insert(alias.clone());
                    } else if let Some(last) = use_stmt.path.last() {
                        exports.insert(last.clone());
                    }
                }
                Item::Module(mod_decl) if mod_decl.is_pub => {
                    exports.insert(mod_decl.name.clone());
                }
                Item::ExternFunction(ext) => {
                    // Extern functions are always available (for FFI)
                    exports.insert(ext.name.clone());
                }
                _ => {}
            }
        }

        exports
    }

    /// Collect dependencies from use statements
    fn collect_dependencies(&self, program: &Program) -> Vec<Vec<String>> {
        let mut deps = Vec::new();

        for item in &program.items {
            if let Item::Use(use_stmt) = item {
                if !use_stmt.path.is_empty() {
                    deps.push(use_stmt.path.clone());
                }
            }
        }

        deps
    }

    /// Topological sort of modules by dependency order
    fn topological_sort(&self) -> Result<Vec<ResolvedModule>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        fn visit(
            name: &str,
            modules: &HashMap<String, ResolvedModule>,
            visited: &mut HashSet<String>,
            temp_visited: &mut HashSet<String>,
            result: &mut Vec<ResolvedModule>,
        ) -> Result<()> {
            if visited.contains(name) {
                return Ok(());
            }
            if temp_visited.contains(name) {
                return Err(Error::ModuleError {
                    message: format!("Cyclic dependency detected: {}", name),
                    path: None,
                });
            }

            temp_visited.insert(name.to_string());

            if let Some(module) = modules.get(name) {
                for dep in &module.dependencies {
                    let dep_name = dep.join("::");
                    visit(&dep_name, modules, visited, temp_visited, result)?;
                }

                visited.insert(name.to_string());
                result.push(module.clone());
            }

            Ok(())
        }

        for name in self.modules.keys() {
            visit(name, &self.modules, &mut visited, &mut temp_visited, &mut result)?;
        }

        Ok(result)
    }

    /// Get a resolved module by canonical name
    pub fn get_module(&self, name: &str) -> Option<&ResolvedModule> {
        self.modules.get(name)
    }

    /// Get all resolved modules
    pub fn modules(&self) -> &HashMap<String, ResolvedModule> {
        &self.modules
    }

    /// Get the project manifest
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get the project root
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

/// Resolve a single LIS file without a project context
/// This is for standalone file compilation (backward compatibility)
pub fn resolve_single_file(file_path: &Path) -> Result<ResolvedModule> {
    let source = std::fs::read_to_string(file_path)
        .map_err(|e| Error::IoError { message: format!("{}: {}", file_path.display(), e) })?;

    let tokens = Lexer::new(&source).tokenize_with_spans()?;
    let ast = Parser::new(tokens).parse()?;

    // For standalone files, we don't resolve dependencies
    // Just check that there are no use statements
    for item in &ast.items {
        if let Item::Use(use_stmt) = item {
            return Err(Error::ModuleError {
                message: format!(
                    "Use statement '{}' found in standalone file. Create a lis.toml for multi-file projects.",
                    use_stmt.path.join("::")
                ),
                path: Some(file_path.to_string_lossy().to_string()),
            });
        }
    }

    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    Ok(ResolvedModule {
        path: vec![file_stem.to_string()],
        canonical_name: file_stem.to_string(),
        file_path: file_path.to_path_buf(),
        ast,
        exports: HashSet::new(), // No exports for standalone
        dependencies: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_project() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();

        // Create lis.toml
        let manifest = r#"
[package]
name = "test-project"
version = "2026.1.16"

[build]
entry = "src/main.lis"
"#;
        std::fs::write(root.join("lis.toml"), manifest).unwrap();

        // Create src directory
        std::fs::create_dir_all(root.join("src")).unwrap();

        (dir, root)
    }

    #[test]
    fn test_resolve_single_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.lis");

        let source = r#"
fn main() {
    let x = 42;
}
"#;
        std::fs::write(&file_path, source).unwrap();

        let module = resolve_single_file(&file_path).unwrap();
        assert_eq!(module.canonical_name, "test");
        assert_eq!(module.ast.items.len(), 1);
    }

    #[test]
    fn test_resolve_project() {
        let (_dir, root) = create_test_project();

        let main_source = r#"
fn main() {
    let x = 42;
}
"#;
        std::fs::write(root.join("src/main.lis"), main_source).unwrap();

        let manifest = Manifest::from_file(&root.join("lis.toml")).unwrap();
        let mut resolver = ModuleResolver::new(manifest, root).unwrap();

        let modules = resolver.resolve_all().unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].canonical_name, "main");
    }
}
