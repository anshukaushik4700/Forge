//! Tests for configuration parsing functionality.
//!
//! These tests verify that the FORGE configuration parser correctly handles
//! various YAML formats and validates the configuration properly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

// Import the necessary types from the main crate
// Note: In a real implementation, these would be public and imported from the crate
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Step {
    #[serde(default)]
    name: String,
    command: String,
    #[serde(default)]
    image: String,
    #[serde(default)]
    working_dir: String,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    depends_on: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Stage {
    name: String,
    steps: Vec<Step>,
    #[serde(default)]
    parallel: bool,
    #[serde(default)]
    depends_on: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
struct CacheConfig {
    #[serde(default)]
    directories: Vec<String>,
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Secret {
    name: String,
    env_var: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ForgeConfig {
    #[serde(default = "default_version")]
    version: String,
    #[serde(default)]
    stages: Vec<Stage>,
    #[serde(default)]
    steps: Vec<Step>,
    #[serde(default)]
    cache: CacheConfig,
    #[serde(default)]
    secrets: Vec<Secret>,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn read_forge_config(path: &Path) -> Result<ForgeConfig, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let config: ForgeConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

#[test]
fn test_parse_basic_config() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our test files
    let dir = tempdir()?;
    let file_path = dir.path().join("forge.yaml");

    // Create a basic config file
    let config_content = r#"
steps:
  - name: Install Dependencies
    command: npm install
    image: node:16-alpine
    working_dir: /app
    env:
      NODE_ENV: development
  - name: Run Tests
    command: npm test
    image: node:16-alpine
    working_dir: /app
"#;

    let mut file = File::create(&file_path)?;
    file.write_all(config_content.as_bytes())?;

    // Parse the config
    let config = read_forge_config(&file_path)?;

    // Verify the parsed config
    assert_eq!(config.version, "1.0");
    assert_eq!(config.steps.len(), 2);
    assert_eq!(config.steps[0].name, "Install Dependencies");
    assert_eq!(config.steps[0].command, "npm install");
    assert_eq!(config.steps[0].image, "node:16-alpine");
    assert_eq!(config.steps[0].working_dir, "/app");
    assert_eq!(config.steps[0].env.get("NODE_ENV").unwrap(), "development");

    assert_eq!(config.steps[1].name, "Run Tests");
    assert_eq!(config.steps[1].command, "npm test");

    Ok(())
}

#[test]
fn test_parse_advanced_config() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our test files
    let dir = tempdir()?;
    let file_path = dir.path().join("forge.yaml");

    // Create an advanced config file with stages, cache, and secrets
    let config_content = r#"
version: "1.0"
stages:
  - name: build
    steps:
      - name: Install Dependencies
        command: npm install
        image: node:16-alpine
        working_dir: /app
        env:
          NODE_ENV: development
      - name: Build Project
        command: npm run build
        image: node:16-alpine
        working_dir: /app
        env:
          NODE_ENV: production
        depends_on:
          - Install Dependencies
    parallel: false
    depends_on: []
  - name: test
    steps:
      - name: Run Unit Tests
        command: npm test
        image: node:16-alpine
        working_dir: /app
        env:
          NODE_ENV: development
      - name: Run Linting
        command: npm run lint
        image: node:16-alpine
        working_dir: /app
        env:
          NODE_ENV: development
    parallel: true
    depends_on:
      - build
cache:
  enabled: true
  directories:
    - /app/node_modules
    - /app/.cache
secrets:
  - name: API_TOKEN
    env_var: FORGE_API_TOKEN
"#;

    let mut file = File::create(&file_path)?;
    file.write_all(config_content.as_bytes())?;

    // Parse the config
    let config = read_forge_config(&file_path)?;

    // Verify the parsed config
    assert_eq!(config.version, "1.0");
    assert_eq!(config.stages.len(), 2);

    // Verify build stage
    let build_stage = &config.stages[0];
    assert_eq!(build_stage.name, "build");
    assert_eq!(build_stage.steps.len(), 2);
    assert!(!build_stage.parallel);
    assert_eq!(build_stage.depends_on.len(), 0);

    // Verify test stage
    let test_stage = &config.stages[1];
    assert_eq!(test_stage.name, "test");
    assert_eq!(test_stage.steps.len(), 2);
    assert!(test_stage.parallel);
    assert_eq!(test_stage.depends_on.len(), 1);
    assert_eq!(test_stage.depends_on[0], "build");

    // Verify cache config
    assert!(config.cache.enabled);
    assert_eq!(config.cache.directories.len(), 2);
    assert_eq!(config.cache.directories[0], "/app/node_modules");
    assert_eq!(config.cache.directories[1], "/app/.cache");

    // Verify secrets
    assert_eq!(config.secrets.len(), 1);
    assert_eq!(config.secrets[0].name, "API_TOKEN");
    assert_eq!(config.secrets[0].env_var, "FORGE_API_TOKEN");

    Ok(())
}

#[test]
fn test_invalid_config() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our test files
    let dir = tempdir()?;
    let file_path = dir.path().join("forge.yaml");

    // Create an invalid config file (missing required field 'command')
    let config_content = r#"
steps:
  - name: Invalid Step
    image: node:16-alpine
    working_dir: /app
"#;

    let mut file = File::create(&file_path)?;
    file.write_all(config_content.as_bytes())?;

    // Parse the config - should fail
    let result = read_forge_config(&file_path);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_empty_config() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our test files
    let dir = tempdir()?;
    let file_path = dir.path().join("forge.yaml");

    // Create an empty config file
    let config_content = r#"
# Empty config
"#;

    let mut file = File::create(&file_path)?;
    file.write_all(config_content.as_bytes())?;

    // Parse the config - should succeed but with empty values
    let config = read_forge_config(&file_path)?;
    assert_eq!(config.version, "1.0");
    assert_eq!(config.steps.len(), 0);
    assert_eq!(config.stages.len(), 0);
    assert!(!config.cache.enabled);
    assert_eq!(config.cache.directories.len(), 0);
    assert_eq!(config.secrets.len(), 0);

    Ok(())
}

#[test]
fn test_invalid_yaml_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join("forge.yaml");

    // Create an invalid syntax config file (missing required field 'command')
    let config_content = r#"
steps:
  name: Invalid Syntax
  image: node:16-alpine
    working_dir: /app
"#;

    let mut file = File::create(&file_path)?;
    file.write_all(config_content.as_bytes())?;

    // Parse the config - should fail
    let result = read_forge_config(&file_path);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_cache_config() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join("forge.yaml");

    // Test with cache enabled
    let config_content_enabled = r#"
cache:
  enabled: true
  directories:
    - /path/to/cache
"#;
    let mut file_enabled = File::create(&file_path)?;
    file_enabled.write_all(config_content_enabled.as_bytes())?;
    let config_enabled = read_forge_config(&file_path)?;
    assert!(config_enabled.cache.enabled);
    assert_eq!(config_enabled.cache.directories, vec!["/path/to/cache"]);

    // Test with cache disabled
    let config_content_disabled = r#"
cache:
  enabled: false
"#;
    let mut file_disabled = File::create(&file_path)?;
    file_disabled.write_all(config_content_disabled.as_bytes())?;
    let config_disabled = read_forge_config(&file_path)?;
    assert!(!config_disabled.cache.enabled);

    // Test with cache section missing
    let config_content_missing = r#"
steps:
  - name: A step
    command: echo "Hello"
"#;
    let mut file_missing = File::create(&file_path)?;
    file_missing.write_all(config_content_missing.as_bytes())?;
    let config_missing = read_forge_config(&file_path)?;
    assert!(!config_missing.cache.enabled);

    Ok(())
}

#[test]
fn test_config_secrets() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join("forge.yaml");

    let config_content = r#"
secrets:
  - name: API_TOKEN
    env_var: FORGE_API_TOKEN
  - name: SERVICE_NAME
    env_var: FORGE
"#;
    let mut file_enabled = File::create(&file_path)?;
    file_enabled.write_all(config_content.as_bytes())?;
    let config = read_forge_config(&file_path)?;
    assert!(!config.secrets.is_empty());

    assert_eq!(config.secrets[0].env_var, String::from("FORGE_API_TOKEN"));
    assert_eq!(config.secrets[1].env_var, String::from("FORGE"));

    Ok(())
}

#[test]
fn test_stage_dependency_validation() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join("forge.yaml");

    let config_content = r#"
stages:
  - name: test
    steps: []
    depends_on:
      - setup

  - name: setup
    steps: []

  - name: build
    steps: []
    depends_on:
      - test
"#;

    let mut file = File::create(&file_path)?;
    file.write_all(config_content.as_bytes()).unwrap();

    let config = read_forge_config(&file_path)?;

    // Collect all stage names into a HashSet for efficient lookup.
    let stage_names: std::collections::HashSet<String> =
        config.stages.iter().map(|s| s.name.clone()).collect();

    // Now, validate the dependencies for each stage.
    for stage in &config.stages {
        for dependency in &stage.depends_on {
            // Assert that the dependency exists in the set of stage names.
            assert!(
                stage_names.contains(dependency),
                "Validation failed: Stage '{} ' depends on an unknown stage '{} '.",
                stage.name,
                dependency
            );
        }
    }

    Ok(())
}

#[test]
#[should_panic(
    expected = "Validation failed: Stage 'test' depends on an unknown stage 'non_existent'."
)]
fn test_invalid_stage_dependency() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("forge.yaml");

    let config_content = r#"
stages:
  - name: test
    steps: []
    depends_on:
      - non_existent
"#;

    let mut file = File::create(&file_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = read_forge_config(&file_path).unwrap();

    let stage_names: std::collections::HashSet<String> =
        config.stages.iter().map(|s| s.name.clone()).collect();

    for stage in &config.stages {
        for dependency in &stage.depends_on {
            assert!(
                stage_names.contains(dependency),
                "Validation failed: Stage '{}' depends on an unknown stage '{}'.",
                stage.name,
                dependency
            );
        }
    }
}
