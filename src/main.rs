//! # FORGE CLI
//!
//! FORGE is a lightweight local CI/CD system built with Rust that allows you
//! to run automation pipelines on your local machine. It's extremely useful for
//! developing and testing pipelines before pushing them to larger CI/CD systems.
//!
//! ## Key Features
//!
//! - Run CI/CD pipelines from simple YAML files
//! - Isolation using Docker containers
//! - Support for various Docker images
//! - Real-time log streaming with colors
//! - Multi-stage pipelines with parallel execution
//! - Caching to speed up builds
//! - Secure secrets management
//!
//! ## Usage
//!
//! ```bash
//! # Initialize a project with an example configuration file
//! forge-cli init
//!
//! # Validate the configuration
//! forge-cli validate
//!
//! # Run the pipeline
//! forge-cli run
//! ```

use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::models::{HostConfig, Mount, MountTypeEnum};
use std::env;
use futures_util::stream::StreamExt;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

/// Representation of a step in a CI/CD pipeline.
///
/// Each step runs in a separate Docker container and can have
/// configurations like image, working directory, and environment variables.
///
/// # Example
///
/// ```yaml
/// steps:
///   - name: Build
///     command: npm run build
///     image: node:16-alpine
///     working_dir: /app
///     env:
///       NODE_ENV: production
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Step {
    /// Step name (optional)
    /// If not provided, a numeric index will be used.
    #[serde(default)]
    name: String,
    
    /// Command to run inside the container
    /// This is the only required field.
    command: String,
    
    /// Docker image to use
    /// If not provided, `alpine:latest` will be used.
    #[serde(default)]
    image: String,
    
    /// Working directory inside the container
    /// If not provided, the container's default directory will be used.
    #[serde(default)]
    working_dir: String,
    
    /// Environment variables to set inside the container
    /// Format: key-value pairs
    #[serde(default)]
    env: std::collections::HashMap<String, String>,
    
    /// Dependencies on other steps (optional)
    /// This step will only run after all the mentioned steps have completed.
    #[serde(default)]
    depends_on: Vec<String>,
}

/// Representation of a stage in a CI/CD pipeline.
///
/// A stage is a collection of steps that can be executed
/// sequentially or in parallel. Stages can also have dependencies
/// on other stages.
///
/// # Example
///
/// ```yaml
/// stages:
///   - name: build
///     steps:
///       - name: Build
///         command: npm run build
///     parallel: false
///     depends_on: []
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Stage {
    /// Stage name
    /// Used for reference in dependencies.
    name: String,
    
    /// Steps in this stage
    /// There must be at least one step.
    steps: Vec<Step>,
    
    /// Whether steps in this stage are executed in parallel
    /// If true, all steps will be run simultaneously.
    /// If false, steps will be run sequentially based on dependencies.
    #[serde(default)]
    parallel: bool,
    
    /// Dependencies on other stages (optional)
    /// This stage will only run after all the mentioned stages have completed.
    #[serde(default)]
    depends_on: Vec<String>,
}

/// Main configuration for FORGE pipeline.
///
/// This is the root structure parsed from the YAML file.
/// It supports two formats: the old format with just `steps` and
/// the new format with `stages`, `cache`, and `secrets`.
///
/// # Old Format Example
///
/// ```yaml
/// steps:
///   - name: Build
///     command: npm run build
/// ```
///
/// # New Format Example
///
/// ```yaml
/// version: "1.0"
/// stages:
///   - name: build
///     steps:
///       - name: Build
///         command: npm run build
/// cache:
///   enabled: true
///   directories:
///     - /app/node_modules
/// secrets:
///   - name: API_TOKEN
///     env_var: FORGE_API_TOKEN
/// ```
#[derive(Debug, Serialize, Deserialize)]
struct ForgeConfig {
    /// Configuration format version
    /// Default: "1.0"
    #[serde(default = "default_version")]
    version: String,
    
    /// Stages in the pipeline
    /// The new format uses stages for better organization.
    #[serde(default)]
    stages: Vec<Stage>,
    
    /// Steps in the pipeline (for compatibility with the old format)
    /// If stages is empty, these steps will be run sequentially.
    #[serde(default)]
    steps: Vec<Step>,
    
    /// Cache configuration
    /// Used to speed up builds by caching certain directories.
    #[serde(default)]
    cache: CacheConfig,
    
    /// Secrets to use in the pipeline
    /// Used to provide sensitive values to containers.
    #[serde(default)]
    secrets: Vec<Secret>,
}

/// Helper function to provide a default value for the configuration version.
///
/// Always returns "1.0" as the default version.
fn default_version() -> String {
    "1.0".to_string()
}

/// Configuration for directory caching.
///
/// Caching can speed up builds by preserving certain directories
/// (like node_modules) between pipeline executions.
///
/// # Example
///
/// ```yaml
/// cache:
///   enabled: true
///   directories:
///     - /app/node_modules
///     - /app/.cache
/// ```
#[derive(Debug, Serialize, Deserialize, Default)]
struct CacheConfig {
    /// Directories to cache
    /// Paths relative to the working directory inside the container.
    #[serde(default)]
    directories: Vec<String>,
    
    /// Whether caching is enabled
    /// If false, caching won't be performed even if directories are specified.
    #[serde(default)]
    enabled: bool,
}

/// Representation of a secret in the pipeline.
///
/// Secrets are used to provide sensitive values (like API tokens)
/// to containers without storing them in the configuration file.
///
/// # Example
///
/// ```yaml
/// secrets:
///   - name: API_TOKEN
///     env_var: FORGE_API_TOKEN
/// ```
///
/// The secret value is taken from the `FORGE_API_TOKEN` environment variable on the host
/// and provided as the `API_TOKEN` environment variable inside the container.
#[derive(Debug, Serialize, Deserialize)]
struct Secret {
    /// Secret name
    /// Will be the name of the environment variable inside the container.
    name: String,
    
    /// Name of the environment variable on the host containing the secret value
    /// The value of this environment variable will be used as the secret value.
    env_var: String,
}

/// Main CLI structure for FORGE.
///
/// Defines all subcommands and options available in the CLI.
/// Uses the `clap` library for command-line argument parsing.
#[derive(Parser)]
#[command(
    name = "forge",
    author = "FORGE Team",
    version = "0.1.0",
    about = "Local CI/CD Runner",
    long_about = "FORGE is a CLI tool designed for developers frustrated with the slow feedback cycle of cloud-based CI/CD. By emulating CI/CD pipelines locally using Docker, FORGE aims to drastically improve developer productivity."
)]
struct Cli {
    /// Subcommand to run (run, init, or validate)
    #[command(subcommand)]
    command: Commands,
}

/// Enum defining all subcommands available in the FORGE CLI.
///
/// Each variant represents one subcommand with its own options.
#[derive(Subcommand)]
enum Commands {
    /// Run the FORGE pipeline
    ///
    /// This subcommand reads the configuration file, validates it, and runs
    /// the pipeline according to that configuration. The pipeline can be in the old format
    /// with just steps, or the new format with stages, cache, and secrets.
    ///
    /// # Examples
    ///
    /// ```bash
    /// forge-cli run
    /// forge-cli run --file custom-forge.yaml --verbose
    /// forge-cli run --stage build
    /// forge-cli run --cache
    /// ```
    Run {
        /// Path to the forge.yaml file
        /// Default: "forge.yaml" in the current directory
        #[arg(short, long, default_value = "forge.yaml")]
        file: String,
        
        /// Show more detailed output
        /// Includes information about containers, images, and full logs
        #[arg(short, long)]
        verbose: bool,
        
        /// Enable caching (overrides configuration)
        /// If set, will enable caching even if disabled in the configuration
        #[arg(long)]
        cache: bool,
        
        /// Disable caching (overrides configuration)
        /// If set, will disable caching even if enabled in the configuration
        #[arg(long)]
        no_cache: bool,
        
        /// Run only a specific stage
        /// If set, only the mentioned stage will be run
        #[arg(short, long)]
        stage: Option<String>,
    },
    
    /// Initialize a new forge.yaml file
    ///
    /// This subcommand creates an example configuration file that can be modified
    /// as needed. The file contains a simple pipeline with some steps, stages, cache, and secrets.
    ///
    /// # Examples
    ///
    /// ```bash
    /// forge-cli init
    /// forge-cli init --file custom-forge.yaml
    /// forge-cli init --force
    /// ```
    Init {
        /// Path to create the forge.yaml file
        /// Default: "forge.yaml" in the current directory
        #[arg(short, long, default_value = "forge.yaml")]
        file: String,
        
        /// Force overwrite if file exists
        /// If not set and the file already exists, an error will be shown
        #[arg(short = 'F', long)]
        force: bool,
    },
    
    /// Validate a forge.yaml file
    ///
    /// This subcommand reads the configuration file and validates its format
    /// without running the pipeline. Useful for checking if the configuration
    /// is valid before running it.
    ///
    /// # Examples
    ///
    /// ```bash
    /// forge-cli validate
    /// forge-cli validate --file custom-forge.yaml
    /// ```
    Validate {
        /// Path to the forge.yaml file to validate
        /// Default: "forge.yaml" in the current directory
        #[arg(short, long, default_value = "forge.yaml")]
        file: String,
    },
}

/// Read and parse the FORGE configuration file.
///
/// This function reads a YAML file from the given path and parses it
/// into a `ForgeConfig` structure. If the file doesn't exist or its format is invalid,
/// an error will be returned.
///
/// # Arguments
///
/// * `path` - Path to the FORGE configuration file (usually forge.yaml)
///
/// # Returns
///
/// * `Result<ForgeConfig, Box<dyn std::error::Error + Send + Sync>>` - Parsing result or error
///
/// # Errors
///
/// This function will return an error if:
/// - The file is not found
/// - The file cannot be read
/// - The YAML format is invalid
/// - The YAML format doesn't match the `ForgeConfig` structure
///
/// # Example
///
/// ```rust
/// let config = read_forge_config(Path::new("forge.yaml"))?;
/// println!("Loaded config with {} stages", config.stages.len());
/// ```
fn read_forge_config(path: &Path) -> Result<ForgeConfig, Box<dyn std::error::Error + Send + Sync>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let config: ForgeConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

/// Pull a Docker image if not already available.
///
/// This function checks if the required Docker image is already available
/// on the local machine. If not, it will pull it from the Docker registry.
/// Displays a spinner during the pulling process.
///
/// # Arguments
///
/// * `docker` - Reference to the Docker client
/// * `image` - Name of the Docker image to pull (e.g., "node:16-alpine")
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
///
/// # Errors
///
/// This function will return an error if:
/// - Connection to the Docker daemon fails
/// - The image is not found in the registry
/// - There's no internet connection (if the image is not already available locally)
///
/// # Example
///
/// ```rust
/// let docker = Docker::connect_with_local_defaults()?;
/// pull_image(&docker, "node:16-alpine").await?;
/// ```
async fn pull_image(docker: &Docker, image: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("{}", format!("Pulling image: {}", image).cyan().bold());
    
    let options = Some(CreateImageOptions {
        from_image: image.to_string(),
        ..Default::default()
    });
    
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    spinner.set_message(format!("Pulling {}", image));
    
    let mut stream = docker.create_image(options, None, None);
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                if let Some(status) = info.status {
                    spinner.set_message(format!("{}: {}", image, status));
                }
            },
            Err(e) => {
                spinner.finish_with_message(format!("Failed to pull image: {}", image));
                return Err(Box::new(e));
            },
        }
    }
    
    spinner.finish_with_message(format!("{}", format!("Image pulled successfully: {}", image).green()));
    Ok(())
}

/// Run a command in a Docker container.
///
/// This function creates and runs a Docker container based on the step configuration,
/// runs the specified command, and displays the output in real-time.
/// The container will be removed after the command finishes.
///
/// # Arguments
///
/// * `docker` - Reference to the Docker client
/// * `step` - Step configuration to run
/// * `verbose` - Whether verbose output is enabled
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
///
/// # Errors
///
/// This function will return an error if:
/// - Connection to the Docker daemon fails
/// - The image is not found
/// - The container fails to be created or run
/// - The command returns a non-zero exit code
///
/// # Example
///
/// ```rust
/// let docker = Docker::connect_with_local_defaults()?;
/// let step = Step {
///     name: "Build".to_string(),
///     command: "npm run build".to_string(),
///     image: "node:16-alpine".to_string(),
///     working_dir: "/app".to_string(),
///     env: HashMap::new(),
///     depends_on: vec![],
/// };
/// run_command_in_container(&docker, &step, true).await?;
/// ```
async fn run_command_in_container(
    docker: &Docker,
    step: &Step,
    verbose: bool,
    cache_config: &CacheConfig,
    temp_dir: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let image = if step.image.is_empty() {
        "alpine:latest"
    } else {
        &step.image
    };
    
    // Pull the image if needed
    pull_image(docker, image).await?;
    
    // Create a unique container name
    let container_name = format!("forge-{}", uuid::Uuid::new_v4());
    
    // Prepare environment variables
    let env: Vec<String> = step.env.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    
    // Create container
    let step_name = if step.name.is_empty() { "unnamed step" } else { &step.name };
    println!("{}", format!("Running step: {}", step_name).yellow().bold());
    if verbose {
        println!("  Command: {}", step.command);
        println!("  Image: {}", image);
        if !step.working_dir.is_empty() {
            println!("  Working directory: {}", step.working_dir);
        }
        if !step.env.is_empty() {
            println!("  Environment variables:");
            for (k, v) in &step.env {
                println!("    {}={}", k, v);
            }
        }
    }
    
    let options = Some(CreateContainerOptions {
        name: container_name.clone(),
        ..Default::default()
    });
    
    // Setup volume mounts for caching
    let mut mounts = vec![];
    
    // Add bind mount for shared data between steps
    let shared_mount = Mount {
        target: Some("/forge-shared".to_string()),
        source: Some(temp_dir.to_string_lossy().to_string()),
        typ: Some(MountTypeEnum::BIND),
        ..Default::default()
    };
    mounts.push(shared_mount);
    
    // Setup host config with mounts
    let host_config = HostConfig {
        auto_remove: Some(false), // Change to false to prevent automatic removal
        mounts: Some(mounts),
        ..Default::default()
    };
    
    // If caching is enabled, add cache directories to the command
    let mut command = step.command.clone();
    if cache_config.enabled && !cache_config.directories.is_empty() {
        // Create a script for cache setup
        let mut cache_setup = String::new();
        for dir in &cache_config.directories {
            // Create the directory in the shared volume if it doesn't exist
            cache_setup.push_str(&format!("mkdir -p /forge-shared{}\n", dir));
            // Create the target directory if it doesn't exist
            cache_setup.push_str(&format!("mkdir -p {}\n", dir));
            // Copy from shared volume to the target directory if it exists
            cache_setup.push_str(&format!("if [ -d /forge-shared{} ] && [ \"$(ls -A /forge-shared{})\" ]; then cp -r /forge-shared{}/* {}/ 2>/dev/null || true; fi\n", dir, dir, dir, dir));
        }
        
        // Create a script for cache teardown
        let mut cache_teardown = String::new();
        for dir in &cache_config.directories {
            // Create the directory in the shared volume if it doesn't exist
            cache_teardown.push_str(&format!("mkdir -p /forge-shared{}\n", dir));
            // Copy from the target directory to the shared volume if it exists
            cache_teardown.push_str(&format!("if [ -d {} ] && [ \"$(ls -A {})\" ]; then cp -r {}/* /forge-shared{}/ 2>/dev/null || true; fi\n", dir, dir, dir, dir));
        }
        
        // Create a combined script
        let script = format!(
            "#!/bin/sh\n\n# Cache setup\n{}\n# Main command\n{}\n\n# Cache teardown\n{}\n\n# Exit with the status of the main command\nexit $?",
            cache_setup, command, cache_teardown
        );
        
        // Use the script as the command
        command = script;
        
        if verbose {
            println!("  Cache enabled for directories: {:?}", cache_config.directories);
        }
    }
    
    let config = Config {
        image: Some(image.to_string()),
        cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), command]),
        env: Some(env),
        working_dir: if step.working_dir.is_empty() { None } else { Some(step.working_dir.clone()) },
        host_config: Some(host_config),
        ..Default::default()
    };
    
    let container = docker.create_container(options, config).await?;
    
    // Start container
    docker.start_container::<String>(&container.id, None).await?;
    
    // Wait for container to finish first
    let mut wait_stream = docker.wait_container::<String>(&container.id, None);
    let wait_future = wait_stream.next();
    
    // Stream logs while waiting
    let log_options = bollard::container::LogsOptions {
        follow: true,
        stdout: true,
        stderr: true,
        ..Default::default()
    };
    
    let mut log_stream = docker.logs::<String>(&container.id, Some(log_options));
    
    while let Some(result) = log_stream.next().await {
        match result {
            Ok(output) => {
                match output {
                    bollard::container::LogOutput::StdOut { message } => {
                        println!("{}", String::from_utf8_lossy(&message));
                    },
                    bollard::container::LogOutput::StdErr { message } => {
                        eprintln!("{}", String::from_utf8_lossy(&message).red());
                    },
                    _ => {}
                }
            },
            Err(e) => {
                eprintln!("Error streaming logs: {}", e);
                break;
            }
        }
    }
    
    // Get the wait result
    let wait_result = wait_future.await;
    
    // Process the wait result
    let exit_status = match wait_result {
        Some(Ok(exit)) => {
            if exit.status_code == 0 {
                println!("{}", format!("Step completed successfully: {}", step_name).green().bold());
                true
            } else {
                let error_msg = format!("Step failed with exit code {}: {}", exit.status_code, step_name);
                println!("{}", error_msg.red().bold());
                false
            }
        },
        Some(Err(e)) => {
            let error_msg = format!("Error waiting for container: {}", e);
            println!("{}", error_msg.red().bold());
            false
        },
        None => {
            let error_msg = "Container exited without providing a status code";
            println!("{}", error_msg.red().bold());
            false
        }
    };
    
    // Clean up the container manually
    match docker.remove_container(&container.id, None).await {
        Ok(_) => println!("Container removed: {}", container.id),
        Err(e) => eprintln!("Failed to remove container: {}", e),
    }
    
    if exit_status {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Step failed: {}", step_name)
        )))
    }
}

/// Create an example forge.yaml file.
///
/// This function creates a new forge.yaml file with an example configuration.
/// If the file already exists and `force` is false, an error will be returned.
///
/// # Arguments
///
/// * `path` - Path where the file should be created
/// * `force` - Whether to overwrite an existing file
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
///
/// # Errors
///
/// This function will return an error if:
/// - The file already exists and `force` is false
/// - The file cannot be created or written to
///
/// # Example
///
/// ```rust
/// create_example_config("forge.yaml", false)?;
/// ```
fn create_example_config(path: &str, force: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if Path::new(path).exists() && !force {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("File {} already exists. Use --force to overwrite.", path)
        )));
    }
    
    let example_config = r#"# FORGE Configuration File
version: "1.0"

# Define stages in your pipeline
stages:
  - name: setup
    steps:
      - name: Install Dependencies
        command: echo "Installing dependencies..."
        image: alpine:latest
    parallel: false
  
  - name: test
    steps:
      - name: Run Tests
        command: echo "Running tests..."
        image: alpine:latest
    depends_on:
      - setup
  
  - name: build
    steps:
      - name: Build Application
        command: echo "Building application..."
        image: alpine:latest
    depends_on:
      - test

# Cache configuration
cache:
  enabled: true
  directories:
    - /app/node_modules
    - /app/.cache

# Secrets configuration
secrets:
  - name: API_TOKEN
    env_var: FORGE_API_TOKEN
"#;
    
    let mut file = File::create(path)?;
    std::io::Write::write_all(&mut file, example_config.as_bytes())?;
    
    println!("{}", format!("Created example configuration file: {}", path).green().bold());
    println!("Edit this file to configure your pipeline.");
    
    Ok(())
}

/// Main function for the FORGE CLI.
///
/// This function parses command-line arguments and executes the appropriate
/// subcommand (run, init, or validate).
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
///
/// # Errors
///
/// This function will return an error if:
/// - Command-line arguments are invalid
/// - The subcommand fails to execute
///
/// # Example
///
/// ```rust
/// fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     tokio::runtime::Runtime::new()?.block_on(async {
///         forge_main().await
///     })
/// }
/// ```
async fn forge_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { file, verbose, cache, no_cache, stage } => {
            println!("{}", "FORGE Pipeline Runner".cyan().bold());
            
            // Read and parse the configuration file
            let config_path = Path::new(&file);
            if !config_path.exists() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Configuration file not found: {}", file)
                )));
            }
            
            let mut config = read_forge_config(config_path)?;
            
            // Override cache settings if specified
            if cache {
                config.cache.enabled = true;
            }
            if no_cache {
                config.cache.enabled = false;
            }
            
            // Connect to Docker
            let docker = Docker::connect_with_local_defaults()?;
            
            // Check if Docker is running
            docker.ping().await?;
            
            // If using the old format (just steps), convert to the new format
            if config.stages.is_empty() && !config.steps.is_empty() {
                config.stages.push(Stage {
                    name: "default".to_string(),
                    steps: config.steps.clone(),
                    parallel: false,
                    depends_on: vec![],
                });
            }
            
            // Filter stages if a specific stage is requested
            if let Some(stage_name) = stage {
                config.stages.retain(|s| s.name == stage_name);
                if config.stages.is_empty() {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Stage not found: {}", stage_name)
                    )));
                }
            }
            
            // Create a temporary directory for sharing data between containers
            let temp_dir = env::temp_dir().join(format!("forge-{}", uuid::Uuid::new_v4()));
            
            // Create the directory if it doesn't exist
            if !temp_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                    eprintln!("Failed to create temporary directory: {}", e);
                    // Continue anyway, as this is not critical
                } else if verbose {
                    println!("Created temporary directory: {}", temp_dir.display());
                }
            }
            
            // Run the pipeline
            for stage in &config.stages {
                println!("{}", format!("Stage: {}", stage.name).cyan().bold());
                
                // Run steps in parallel or sequentially
                if stage.parallel {
                    // TODO: Implement parallel execution
                    // For now, just run sequentially
                    for step in &stage.steps {
                        run_command_in_container(&docker, step, verbose, &config.cache, &temp_dir).await?;
                    }
                } else {
                    for step in &stage.steps {
                        run_command_in_container(&docker, step, verbose, &config.cache, &temp_dir).await?;
                    }
                }
                
                // No cleanup here, we'll do it after all stages are done
            }
            
            // Clean up the temporary directory after the pipeline is done
            if verbose {
                println!("Removing temporary directory: {}", temp_dir.display());
            }
            
            if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
                eprintln!("Failed to remove temporary directory: {}", e);
                // Continue anyway, as this is not critical
            } else if verbose {
                println!("Temporary directory removed successfully");
            }
            
            println!("{}", "Pipeline completed successfully!".green().bold());
            Ok(())
        },
        Commands::Init { file, force } => {
            create_example_config(&file, force)
        },
        Commands::Validate { file } => {
            println!("{}", "Validating configuration file...".cyan().bold());
            
            let config_path = Path::new(&file);
            if !config_path.exists() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Configuration file not found: {}", file)
                )));
            }
            
            let config = read_forge_config(config_path)?;
            
            // Validate the configuration
            if config.stages.is_empty() && config.steps.is_empty() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Configuration must have at least one stage or step"
                )));
            }
            
            // Check for circular dependencies in stages
            // TODO: Implement circular dependency check
            
            println!("{}", "Configuration is valid!".green().bold());
            
            // Print summary
            if !config.stages.is_empty() {
                println!("Stages:");
                for stage in &config.stages {
                    println!("  - {} ({} steps)", stage.name, stage.steps.len());
                }
            } else {
                println!("Steps: {}", config.steps.len());
            }
            
            if config.cache.enabled {
                println!("Cache: Enabled");
                println!("Cached directories:");
                for dir in &config.cache.directories {
                    println!("  - {}", dir);
                }
            } else {
                println!("Cache: Disabled");
            }
            
            if !config.secrets.is_empty() {
                println!("Secrets:");
                for secret in &config.secrets {
                    println!("  - {} (from {})", secret.name, secret.env_var);
                }
            }
            
            Ok(())
        },
    }
}

/// Main function that sets up the async runtime and runs the FORGE CLI.
///
/// This function creates a new Tokio runtime and runs the `forge_main` function
/// inside it. It handles any errors that occur and prints them to stderr.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Success or error
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        if let Err(e) = forge_main().await {
            eprintln!("{}", format!("Error: {}", e).red().bold());
            std::process::exit(1);
        }
        Ok(())
    })
}