// ============================================================================
// MonoX - English Translation Table
// ============================================================================
//
// 文件: src/i18n/en_us.rs
// 职责: English translation content definition
// 边界:
//   - ✅ English translation strings definition
//   - ✅ Translation key-value pairs maintenance
//   - ❌ Should not contain translation logic
//   - ❌ Should not contain business logic
//   - ❌ Should not contain other language translations
//   - ❌ Should not contain dynamic translation generation
//
// ============================================================================

/// English translation table
pub const TRANSLATIONS: &[(&str, &str)] = &[
    // Analyzer related
    ("analyze.start", "Starting workspace dependency analysis..."),
    ("analyze.scanning_workspace", "Scanning workspace: {}"),
    ("analyze.found_packages", "Found {} packages"),
    (
        "analyze.skip_root_package",
        "Skipping root package.json: {}",
    ),
    (
        "analyze.skip_invalid_package",
        "Skipping invalid package.json: {}",
    ),
    (
        "analyze.circular_detected",
        "Circular dependencies detected, cannot calculate build stages",
    ),
    ("analyze.stage_info", "Stage {}: {} packages ({})"),
    (
        "analyze.completed",
        "Analysis completed in {}ms, {} stages total",
    ),
    (
        "analyze.circular_found",
        "Detected {} circular dependencies",
    ),
    ("analyze.circular_detail", "Circular dependency {}: {}"),
    (
        "analyze.circular_warning",
        "Warning: Circular dependencies detected, remaining packages: {}",
    ),
    // Single package analysis related
    ("analyze.single_package_start", "Starting single package analysis: {}"),
    ("analyze.single_package_found", "Found target package '{}': {}"),
    ("analyze.single_package_completed", "Single package analysis completed: {}, took {}ms"),
    // Error messages
    (
        "error.no_packages_found",
        "No valid package.json files found in workspace",
    ),
    ("error.read_package_json", "Failed to read package.json: {}"),
    (
        "error.parse_package_json",
        "Failed to parse package.json: {}",
    ),
    ("error.get_package_dir", "Cannot get package directory"),
    ("error.walk_directory", "Failed to walk directory"),
    (
        "error.workspace_not_exist",
        "Workspace path does not exist: {}",
    ),
    ("error.package_not_found", "Package not found: {}"),
    // CLI related
    (
        "cli.analyze.start",
        "Starting workspace dependency analysis...",
    ),
    // Config related
    ("analyze.config_loaded", "Loaded config file: {}"),
    ("analyze.config_error", "Failed to load config file: {}"),
    (
        "analyze.no_config",
        "No config file found, using default settings",
    ),
    // Output format related
    ("output.analysis_result", "Analysis Result"),
    ("output.total_packages", "Total packages: {}"),
    ("output.total_stages", "Build stages: {}"),
    (
        "output.packages_with_deps",
        "Packages with workspace deps: {}",
    ),
    ("output.analysis_duration", "Analysis duration: {}ms"),
    ("output.circular_dependencies", "Circular Dependencies"),
    (
        "output.no_circular_dependencies",
        "No circular dependencies found",
    ),
    ("output.build_stages", "Build Stages"),
    ("output.stage_info", "Stage {} ({} packages):"),
    ("output.no_workspace_deps", "no workspace dependencies"),
    ("output.depends_on", "depends on: {}"),
    ("output.depends_on_count", "depends on {} packages:"),
    ("output.path", "Path: {}"),
    ("output.version", "Version: {}"),
    ("output.scripts", "Scripts: {}"),
    ("output.package_details", "Package Details"),
    ("output.all_dependencies", "All dependencies ({})"),
    ("output.scripts_detail", "Scripts:"),
    (
        "output.usage_tip",
        "Tip: Use --detail to show dependencies, --verbose for more details, --format json for JSON output",
    ),
    // Init related
    ("init.start", "Initializing MonoX configuration..."),
    ("init.config_exists", "Config file already exists: {}"),
    (
        "init.use_force_hint",
        "Use --force to overwrite existing config file",
    ),
    ("init.config_created", "Config file created: {}"),
    ("init.create_failed", "Failed to create config file: {}"),
    (
        "init.next_steps",
        "You can now edit the config file to suit your project needs",
    ),
];
