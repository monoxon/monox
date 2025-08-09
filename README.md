# MonoX

> 🚀 Lightweight monorepo build tool written in Rust

MonoX is an intelligent build tool designed specifically for monorepo projects, helping you efficiently manage multi-package project builds through dependency analysis and task scheduling optimization.

## ✨ Core Features

- 🔍 **Smart Dependency Analysis** - Automatically parse package dependencies and build directed acyclic graphs
- 📦 **Single Package Analysis** - Support analyzing specific packages and their dependency chains for precise build scope
- 📦 **Multi-Package Operations** - Support simultaneous operations on multiple specified packages
- ⚡ **Concurrent Task Execution** - Concurrent builds within the same stage to maximize CPU utilization
- 🛡️ **Safety Checks** - Circular dependency detection, version conflict checking, outdated dependency scanning
- 📊 **Real-time Progress Display** - Beautiful progress bars and task status visualization
- 🌍 **Complete Internationalization** - Chinese/English bilingual interface with dynamic language switching
- 🎯 **Flexible Configuration** - Customize tasks and execution strategies through `monox.toml`
- 🔧 **Multi-Package Manager Support** - Support for pnpm, npm, yarn
- 🎨 **Smart User Interface** - Real-time refresh UI in non-verbose mode, detailed logs in verbose mode
- ⚙️ **Advanced Execution Control** - Timeout control, error handling, concurrency limits

## 🚀 Quick Start

### Installation

```bash
# Install from npm (recommended)
npm install -g monox
# or
pnpm add -g monox
# or
yarn global add monox

# Build from source (requires Rust environment)
git clone https://github.com/monoxon/monox.git
cd monox
cargo build --release

# Add executable to PATH
cp target/release/monox /usr/local/bin/

# Install failed, use install.sh
pnpm monox-install
```

### Initialize Configuration

Run in your monorepo project root directory:

```bash
monox init
```

This will create a `monox.toml` configuration file.

### Basic Usage

#### analyze - Dependency Analysis

```bash
# Analyze project dependencies and build stages
monox analyze

# Analyze specific package and its dependency chain (single package analysis)
monox analyze --package @your-org/package-name

# View detailed dependency information for single package
monox analyze --package @your-org/package-name --detail --verbose

# JSON format output
monox analyze --format json
```

#### run - Execute Tasks

```bash
# Build all packages (in dependency order)
monox run --all --command build

# Run specific package and its dependencies
monox run @your-org/package-name --command build

# Run multiple specific packages (multi-package execution)
monox run --packages "@your-org/pkg1,@your-org/pkg2,@your-org/pkg3" --command build

# Verbose mode to show execution process
monox run --all --command build --verbose
```

#### exec - Execute Predefined Tasks

```bash
# Execute predefined tasks
monox exec build-all

# Execute multi-package task defined in configuration
monox exec build-frontend

# Verbose mode
monox exec test-all --verbose
```

#### check - Health Check

```bash
# Check project health status
monox check --circular --versions --outdated

# Detailed circular dependency path information
monox check --circular --detail --verbose

# JSON format output for check results
monox check --versions --format json
```

#### fix - Problem Resolution

```bash
# Sync project dependencies to the highest version used in the project
monox fix

# Dry-run mode (no actual modifications)
monox fix --dry-run
```

## 📋 Command Reference

### Global Options

```bash
-v, --verbose           Show detailed execution process
--no-color              Disable colored output
--no-progress           Disable progress display
-j, --max-concurrency   Set maximum concurrency
--timeout               Set task timeout (seconds)
--retry                 Set retry count
--continue-on-failure   Continue execution on failure
-C, --workspace-root    Specify workspace root directory
-l, --language          Set interface language (en_us, zh_cn)
```

### Main Commands

#### `analyze` - Dependency Analysis

```bash
monox analyze                              # Analyze and display build stages
monox analyze --format json               # Output in JSON format
monox analyze --verbose                    # Show detailed dependency relationships
monox analyze --package <package-name>    # Analyze specific single package and its dependency chain
monox analyze --package <package-name> --detail  # Single package analysis with detailed information
monox analyze --packages "pkg1,pkg2,pkg3" # Analyze multiple specified packages and their dependencies
```

#### `run` - Execute Commands

```bash
monox run <package> --command <cmd>         # Run command for specific package
monox run --all --command <cmd>             # Run command for all packages
monox run --packages "pkg1,pkg2" --command <cmd>  # Run command for multiple specified packages
monox run --all --command build -v          # Verbose mode execution
```

#### `exec` - Execute Predefined Tasks

```bash
monox exec <task-name>           # Execute task defined in monox.toml
monox exec build-all --verbose   # Execute task in verbose mode
```

#### `check` - Health Check

```bash
monox check --circular           # Check circular dependencies
monox check --versions           # Check version conflicts
monox check --outdated           # Check outdated dependencies
monox check --circular --detail  # Show detailed circular paths
```

#### `fix` - Problem Resolution

```bash
monox fix --versions             # Fix version inconsistencies
monox fix --dry-run             # Dry-run mode, no actual modifications
```

#### `init` - Initialize

```bash
monox init                      # Initialize configuration file
```

## ⚙️ Configuration File

### monox.toml Configuration Example

```toml
[workspace]
root = "."
package_manager = "pnpm"  # pnpm | npm | yarn
ignore = [                # Directories or file patterns to exclude from scanning
    "dist",
    "build",
    ".git",
    "*.tmp"
]

# Predefined tasks
[[tasks]]
name = "build-all"
pkg_name = "*"
desc = "Build all packages"
command = "build"

[[tasks]]
name = "test-system"
pkg_name = "@your-org/system"
desc = "Test system core package"
command = "test"

# Multi-package task example
[[tasks]]
name = "build-frontend"
desc = "Build frontend related packages"
command = "build"
packages = ["@your-org/web-ui", "@your-org/mobile-app", "@your-org/shared-components"]

# Execution configuration
[execution]
max_concurrency = 4        # Maximum concurrency
task_timeout = 300         # Task timeout (seconds)
retry_count = 0            # Retry count
continue_on_failure = false # Continue on failure

# Output configuration
[output]
show_progress = true       # Show progress bar
verbose = false           # Verbose output
colored = true            # Colored output

# Internationalization configuration
[i18n]
language = "zh_cn"        # Interface language (en_us, zh_cn)
```

### Configuration Parameters

#### [workspace] - Workspace

- `root`: Working directory root path, default "."
- `package_manager`: Package manager type, supports "pnpm", "npm", "yarn"
- `ignore`: Directories or file patterns to exclude from scanning, supports glob patterns. Note: `node_modules` directory is always excluded by default

#### [[tasks]] - Task Definition

- `name`: Task name, used for `monox exec <name>`
- `pkg_name`: Package name, "*" means all packages (optional, can use `packages` instead)
- `packages`: Array of package names for multi-package operations (optional, alternative to `pkg_name`)
- `desc`: Task description (optional)
- `command`: Command to execute

Note: Each task must specify either `pkg_name` or `packages` field.

#### [execution] - Execution Control

- `max_concurrency`: Maximum concurrent tasks, defaults to CPU core count
- `task_timeout`: Single task timeout (seconds), default 300
- `retry_count`: Retry count on failure, default 0
- `continue_on_failure`: Whether to continue on failure, default false

#### [output] - Output Control

- `show_progress`: Whether to show progress bar, default true
- `verbose`: Whether to show verbose output, default false
- `colored`: Whether to use colored output, default true

#### [i18n] - Internationalization

- `language`: Interface language, supports "en_us" (English) and "zh_cn" (Simplified Chinese)

## 🌍 Internationalization Support

MonoX provides complete bilingual support with all user interface texts internationalized:

### Language Selection Priority

1. Command line argument `--language` or `-l`
2. Settings in `monox.toml` configuration file
3. System default (English)

### Usage Examples

```bash
# Use Chinese interface
monox analyze -l zh_cn

# Use English interface
monox run --all --command build --language en_us
```

### Supported Languages
- **zh_cn**: Simplified Chinese - Complete localization support
- **en_us**: American English - Standard English interface

## 📦 Single Package Analysis Feature

MonoX supports precise dependency analysis for specific packages, which is particularly useful in large monorepo projects:

### Features

- **Precise Scope**: Only analyze the target package and its direct dependency chain, excluding unrelated packages
- **Build Optimization**: Show the minimal dependency set required to build the target package
- **Quick Diagnosis**: Quickly understand the dependency status of specific packages
- **Multiple Output Formats**: Support both table and JSON format output

### Usage Examples

```bash
# Basic single package analysis
monox analyze --package @your-org/components

# Output example:
# ◇ Analysis Results
# ● Total packages: 1
# ▪ Build stages: 3
# ◦ Packages with workspace dependencies: 1
#
# ▪ Build Stages
# ─────────────────────────
# Stage 1 (1 package):
#   ● @your-org/utils
#
# Stage 2 (1 package):
#   ● @your-org/core
#
# Stage 3 (1 package):
#   ● @your-org/components

# Detailed information mode
monox analyze --package @your-org/components --detail

# JSON format output (convenient for script processing)
monox analyze --package @your-org/components --format json
```

## 🎨 User Interface Features

### Two Output Modes

#### Refresh Mode (Default)
- Real-time updated progress bars and status display
- Dynamic Spinner animations
- Multi-package parallel execution status tracking
- Retain full progress bar display after completion

```
[MONOX] ⠧ ████████████░░░░░░░░ Stage 3/5
[MONOX] Processing packages: (2/5)
[MONOX]   ○ package-a
[MONOX]   ▸ package-b    ← Running  
[MONOX]   ○ package-c
[MONOX]   ● package-d    ← Completed
[MONOX]   ○ package-e
```

#### Verbose Mode (--verbose)
- Complete execution log output
- Start/completion time for each task
- Detailed error information and stack traces
- Performance statistics

```
[MONOX] ▪ Starting task: build in @your-org/utils
[MONOX] ● Task build completed in @your-org/utils, took 1250ms
[MONOX] ▪ Starting task: build in @your-org/core
```

### Internationalized Interface
- All prompts support Chinese and English
- Localized number and time formats
- Complete error message translation

## 📊 Use Cases

### Typical Workflow

1. **Project Initialization**

   ```bash
   monox init
   # Edit monox.toml configuration file
   ```

2. **Dependency Analysis**

   ```bash
   # Analyze entire workspace
   monox analyze --verbose

   # Analyze specific package and its dependency chain
   monox analyze --package @your-org/core --detail
   ```

3. **Health Check**

   ```bash
   monox check --circular --versions --outdated
   # Ensure project is in good state
   ```

4. **Build Execution**

   ```bash
   monox run --all --command build --verbose
   # Build all packages in dependency order
   ```

5. **Test Execution**
   ```bash
   monox exec test-all
   # Execute predefined test tasks
   ```

### Single Package Analysis and Debugging

```bash
# Analyze dependency relationships of specific package
monox analyze --package @your-org/core

# View detailed dependency information for single package
monox analyze --package @your-org/core --detail --verbose

# Output single package analysis results in JSON format
monox analyze --package @your-org/core --format json

# Analyze multiple packages (execute separately)
monox analyze --package @your-org/utils
monox analyze --package @your-org/components
```

### Debugging and Diagnostics

```bash
# Verbose mode: view build process and progress
monox run --all --command build --verbose

# Combined usage: most complete information output
monox analyze --verbose --detail
```

## 🔧 Technical Features

### Core Engine
- **Dependency Analyzer**: Graph algorithms based on petgraph, supporting cycle detection and topological sorting
- **Task Executor**: Asynchronous concurrent execution, intelligent scheduling and resource management
- **Cache System**: Smart caching to improve repeated operation performance

### User Experience
- **Smart UI**: Dynamic refresh interface in non-verbose mode, complete log output in verbose mode
- **Progress Tracking**: Real-time progress bars, task status, execution time statistics
- **Error Handling**: Friendly error messages, internationalized error messages, failure retry mechanism

### Architecture Design
- **Modular**: Clear module boundaries and separation of responsibilities
- **Type Safety**: Full utilization of Rust type system for safety guarantees
- **Async First**: High-performance async runtime based on tokio

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Related Links

- [中文文档](README.zh.md)
- [Design Document](DESIGN.md)
- [Development Task List](TODOLIST.md)
- [Change Log](CHANGELOG.md)
- [Issue Tracking](https://github.com/your-org/monox/issues)

## 📦 Multi-Package Operations

MonoX supports efficient operations on multiple specified packages:

### Features

- **Flexible Package Selection**: Specify multiple packages via command line or configuration
- **Intelligent Dependency Analysis**: Automatically analyze dependencies between specified packages
- **Optimized Execution Order**: Execute packages in proper dependency order
- **Shared Infrastructure**: Reuses single-package analysis and execution infrastructure

### Usage Examples

#### Command Line Multi-Package Operations

```bash
# Analyze multiple specific packages
monox analyze --packages "@your-org/ui-lib,@your-org/web-app,@your-org/mobile-app" --detail

# Build multiple specific packages
monox run --packages "@your-org/ui-lib,@your-org/web-app,@your-org/mobile-app" --command build

# Test multiple packages in verbose mode
monox run --packages "pkg1,pkg2,pkg3" --command test --verbose

# Parameter priority: --all > --packages > --package
monox run --all --command build  # Highest priority, builds all packages
```

#### Configuration File Multi-Package Tasks

```toml
# Define multi-package task in monox.toml
[[tasks]]
name = "build-frontend"
desc = "Build all frontend related packages"
command = "build"
packages = ["@your-org/web-ui", "@your-org/mobile-app", "@your-org/shared-components"]

[[tasks]]
name = "test-backend"
desc = "Test backend services"
command = "test"
packages = ["@your-org/api-server", "@your-org/auth-service", "@your-org/database-lib"]
```

```bash
# Execute multi-package tasks
monox exec build-frontend
monox exec test-backend --verbose
```

### Technical Implementation

- **Multi-Package Analysis**: `analyze_packages()` method supports simultaneous analysis of multiple packages
- **Smart Deduplication**: Automatically removes duplicate packages and optimizes dependency resolution
- **Stage-based Execution**: Uses dependency analysis results for intelligent scheduling
- **Complete Backward Compatibility**: Existing single-package functionality remains unchanged 