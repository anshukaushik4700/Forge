# FORGE - Local CI/CD Runner

<div align="center">
  <!-- Placeholder for logo, will be added later -->
  <p><strong>Fast, Offline, Reliable, Go-anywhere Execution</strong></p>
  <p>
    <a href="https://github.com/0xReLogic/Forge/actions/workflows/ci.yml">
      <img src="https://github.com/0xReLogic/Forge/actions/workflows/ci.yml/badge.svg" alt="FORGE CI">
    </a>
    <a href="https://opensource.org/licenses/MIT">
      <img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT">
    </a>
  </p>
  <p>
    <a href="#installation"><strong>Installation</strong></a> ·
    <a href="#usage"><strong>Usage</strong></a> ·
    <a href="#documentation"><strong>Documentation</strong></a> ·
    <a href="#contributing"><strong>Contributing</strong></a>
  </p>
</div>

## About FORGE

FORGE is a lightweight local CI/CD tool built with Rust that allows you to run automation pipelines on your local machine. It's extremely useful for developing and testing pipelines before pushing them to larger CI/CD systems.

### Why FORGE?

- **Local & Fast**: Run pipelines on your local machine without waiting for CI/CD servers
- **Offline-First**: Works without an internet connection (as long as Docker images are available)
- **Compatible**: Syntax similar to GitHub Actions and GitLab CI
- **Lightweight**: Minimal resource consumption
- **Portable**: Runs on Windows, macOS, and Linux

## Features

- ✅ Run CI/CD pipelines from simple YAML files
- ✅ Isolation using Docker containers
- ✅ Support for various Docker images
- ✅ Real-time log streaming with colors
- ✅ Environment variables management
- ✅ Intuitive command-line interface
- ✅ Multi-stage pipelines with parallel execution
- ✅ Caching to speed up builds
- ✅ Secure secrets management
- ✅ Dependencies between steps and stages

## Installation

### Prerequisites

#### Recommended: VSCode Docker Extension
1. Install the [Docker Extension](https://marketplace.visualstudio.com/items?itemName=ms-azuretools.vscode-docker) in Visual Studio Code
2. Restart VSCode to activate the extension
3. The extension will handle Docker installation and configuration automatically

#### Alternative: Manual Installation
- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or newer)
- [Docker Desktop](https://docs.docker.com/get-docker/) (version 20.10.0 or newer)

#### Package Manager Installation
```bash
# macOS
brew install --cask docker

# Windows with Chocolatey
choco install docker-desktop

# Linux (Ubuntu/Debian)
sudo apt install docker.io
```

### From Source

```bash
git clone https://github.com/0xReLogic/Forge.git
cd forge
cargo build --release
```

The executable will be available at `target/release/forge-cli`.

### With Cargo

```bash
cargo install forge-cli
```

### Binary Releases

You can also download pre-compiled binaries from the [releases page](https://github.com/0xReLogic/Forge/releases).

## Quick Usage

```bash
# Initialize a project with an example configuration file
forge-cli init

# Validate the configuration
forge-cli validate

# Run the pipeline
forge-cli run
```

## VSCode Integration

### Quick Setup
For VSCode users, FORGE integrates seamlessly with the Docker extension:

1. Open your project in VSCode
2. Ensure the Docker extension is installed and active
3. Initialize FORGE configuration: `forge-cli init`
4. Run pipeline: `forge-cli run`

### Recommended Extensions
For optimal experience, consider installing:
- Docker (ms-azuretools.vscode-docker)
- YAML (redhat.vscode-yaml)
- JSON (ms-vscode.vscode-json)

## Documentation

### Project Initialization

Create a new forge.yaml configuration file:

```bash
forge-cli init
```

Or with a different filename:

```bash
forge-cli init --file custom-forge.yaml
```

Use the `--force` flag to overwrite an existing file:

```bash
forge-cli init --force
```

### Configuration Validation

Validate a configuration file:

```bash
forge-cli validate
```

Or with a different configuration file:

```bash
forge-cli validate --file custom-forge.yaml
```

### Run Pipeline

Run the pipeline:

```bash
forge-cli run
```

Or with a different configuration file:

```bash
forge-cli run --file custom-forge.yaml
```

Run with verbose output:

```bash
forge-cli run --verbose
```

Run a specific stage:

```bash
forge-cli run --stage build
```

Enable or disable caching:

```bash
forge-cli run --cache
forge-cli run --no-cache
```

### Using Secrets

Secrets are defined in the configuration file and their values are taken from environment variables:

```bash
# Set the secret value in the environment
export FORGE_API_TOKEN=your_secret_token

# Run the pipeline with the secret
forge-cli run
```

## Configuration Format

FORGE uses YAML format for pipeline configuration. Two formats are supported:

### Basic Format

The basic format supports a list of steps that are executed sequentially.

```yaml
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
```

### Advanced Format with Multi-Stage

The advanced format supports stages, dependencies, parallel execution, caching, and secrets.

```yaml
version: "1.0"
stages:
  - name: build
    steps:
      - name: Install Dependencies
        command: npm install
        image: node:16-alpine
        working_dir: /app
    parallel: false
  - name: test
    steps:
      - name: Run Tests
        command: npm test
        image: node:16-alpine
        working_dir: /app
    depends_on:
      - build
cache:
  enabled: true
  directories:
    - /app/node_modules
secrets:
  - name: API_TOKEN
    env_var: FORGE_API_TOKEN
```

### Configuration Reference

#### Step Properties

| Property | Description | Required | Default |
|----------|-------------|---------|---------|
| `name` | Step name | No | `""` |
| `command` | Command to run | Yes | - |
| `image` | Docker image to use | No | `alpine:latest` |
| `working_dir` | Working directory inside the container | No | `""` |
| `env` | Environment variables | No | `{}` |
| `depends_on` | Dependencies on other steps | No | `[]` |

#### Stage Properties

| Property | Description | Required | Default |
|----------|-------------|---------|---------|
| `name` | Stage name | Yes | - |
| `steps` | Steps in the stage | Yes | - |
| `parallel` | Whether steps are executed in parallel | No | `false` |
| `depends_on` | Dependencies on other stages | No | `[]` |

#### Cache Properties

| Property | Description | Required | Default |
|----------|-------------|---------|---------|
| `enabled` | Whether caching is enabled | No | `false` |
| `directories` | Directories to cache | No | `[]` |

#### Secret Properties

| Property | Description | Required | Default |
|----------|-------------|---------|---------|
| `name` | Secret name | Yes | - |
| `env_var` | Environment variable name on the host | Yes | - |

## Advanced Features

### Multi-Stage Execution

FORGE supports multi-stage pipelines where stages can depend on each other. This allows you to create complex workflows with dependencies:

1. **Sequential Execution**: Stages run in order based on their dependencies
2. **Parallel Steps**: Steps within a stage can run in parallel if `parallel: true` is set
3. **Dependency Management**: Use `depends_on` to specify which stages must complete before a stage can start

Example of a multi-stage pipeline:

```yaml
stages:
  - name: setup
    steps:
      - name: Install Dependencies
        command: npm install
    parallel: false
  
  - name: test
    steps:
      - name: Unit Tests
        command: npm test
      - name: Integration Tests
        command: npm run test:integration
    parallel: true
    depends_on:
      - setup
  
  - name: build
    steps:
      - name: Build Application
        command: npm run build
    depends_on:
      - test
```

### Caching

FORGE provides a caching mechanism to speed up your pipelines by preserving files between runs:

1. **Directory Caching**: Specify directories to cache between steps and stages
2. **Automatic Management**: FORGE automatically handles copying files to and from the cache
3. **Cache Control**: Enable or disable caching via configuration or command line flags

Example of cache configuration:

```yaml
cache:
  enabled: true
  directories:
    - /app/node_modules
    - /app/.cache
    - /app/dist
```

You can also control caching from the command line:

```bash
# Force enable caching
forge-cli run --cache

# Force disable caching
forge-cli run --no-cache
```

## Usage Examples

### Node.js Project

```yaml
version: "1.0"
stages:
  - name: setup
    steps:
      - name: Install Dependencies
        command: npm install
        image: node:16-alpine
        working_dir: /app
    parallel: false
  - name: test
    steps:
      - name: Lint
        command: npm run lint
        image: node:16-alpine
        working_dir: /app
      - name: Unit Tests
        command: npm test
        image: node:16-alpine
        working_dir: /app
    parallel: true
    depends_on:
      - setup
  - name: build
    steps:
      - name: Build
        command: npm run build
        image: node:16-alpine
        working_dir: /app
        env:
          NODE_ENV: production
    depends_on:
      - test
cache:
  enabled: true
  directories:
    - /app/node_modules
```

### Rust Project

```yaml
version: "1.0"
stages:
  - name: check
    steps:
      - name: Cargo Check
        command: cargo check
        image: rust:1.70-slim
        working_dir: /app
  - name: test
    steps:
      - name: Cargo Test
        command: cargo test
        image: rust:1.70-slim
        working_dir: /app
    depends_on:
      - check
  - name: build
    steps:
      - name: Cargo Build
        command: cargo build --release
        image: rust:1.70-slim
        working_dir: /app
    depends_on:
      - test
cache:
  enabled: true
  directories:
    - /app/target
    - /usr/local/cargo/registry
```

## Architecture

FORGE consists of several logical components currently implemented in a monolithic file:

1. **YAML Parser**: Reads and validates configuration files
2. **Orchestrator**: Manages pipeline execution, including dependencies and parallelism
3. **Docker Client**: Interacts with Docker API to run containers
4. **Logger**: Handles log streaming from containers to the terminal
5. **Cache Manager**: Manages directory caching to speed up builds
6. **Secret Manager**: Securely manages secrets

In future development, these components will be refactored into separate modules to improve maintainability and testability.

## Comparison with Other Tools

| Feature | FORGE | GitHub Actions | GitLab CI | Jenkins |
|-------|-------|---------------|-----------|---------|
| Local Execution | ✅ | ❌ | ❌ | ⚠️ (complex) |
| Offline | ✅ | ❌ | ❌ | ✅ |
| Lightweight | ✅ | ❌ | ❌ | ❌ |
| Multi-Stage | ✅ | ✅ | ✅ | ✅ |
| Parallelism | ✅ | ✅ | ✅ | ✅ |
| Caching | ✅ | ✅ | ✅ | ✅ |
| Secrets | ✅ | ✅ | ✅ | ✅ |

## Roadmap

### Completed Milestones
- [x] Phase 1: Core Engine
  - [x] Project Setup & Blueprint Parsing
  - [x] Docker Integration
  - [x] Basic Pipeline Execution
  - [x] Command-line Interface

- [x] Phase 2: Enhanced Features
  - [x] Real-time Log Streaming
  - [x] Environment Variables Support
  - [x] Working Directory Configuration
  - [x] Step Dependencies

- [x] Phase 3: Advanced Features
  - [x] Multi-Stage Pipeline
  - [x] Dependency Caching
  - [x] Secret Management

- [x] Phase 4: Open Source Readiness
  - [x] Comprehensive Documentation
  - [x] Thorough Testing
  - [x] CI/CD for FORGE Itself

### Future Development

#### Phase 5: Ecosystem Growth (Next 3-6 months)
- [ ] Plugin System for Extensibility
- [ ] Pipeline Templates Library
- [ ] Improved Error Handling and Recovery
- [ ] Pipeline Visualization
- [ ] Enhanced Testing Framework

#### Phase 6: Enterprise Features (6-12 months)
- [ ] Web Dashboard for Monitoring
- [ ] Remote Execution Support
- [ ] Notifications (Email, Slack, etc.)
- [ ] Performance Optimizations
- [ ] Advanced Caching Strategies

#### Phase 7: Advanced Capabilities (12+ months)
- [ ] Distributed Execution
- [ ] Advanced Scheduling and Resource Management
- [ ] Integration with Popular CI/CD Platforms
- [ ] Custom Workflow Languages
- [ ] Enterprise Security Features

## Contributing

Contributions are always welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on the process for submitting pull requests.

### How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the [MIT License](LICENSE).

## Credits

- Inspired by [GitHub Actions](https://github.com/features/actions), [GitLab CI](https://docs.gitlab.com/ee/ci/), and [Jenkins](https://www.jenkins.io/)

## FAQ

### Can FORGE replace GitHub Actions/GitLab CI?

FORGE is not designed to replace cloud-based CI/CD systems, but to complement them. FORGE is extremely useful for developing and testing pipelines before pushing them to larger CI/CD systems.

### Does FORGE support Windows?

Yes, FORGE runs on Windows, macOS, and Linux.

### How do I add secrets?

You can add secrets by defining them in the configuration file and providing their values through environment variables on the host machine.

### Does FORGE support parallel execution?

Yes, FORGE supports parallel execution for steps in the same stage by setting `parallel: true` on the stage.

## Contact

Your Name - email@example.com

Project Link: [https://github.com/0xReLogic/Forge](https://github.com/0xReLogic/Forge)