// ============================================================================
// MonoX - 中文翻译表
// ============================================================================
//
// 文件: src/i18n/zh_cn.rs
// 职责: 中文翻译内容定义
// 边界:
//   - ✅ 中文翻译字符串定义
//   - ✅ 翻译键值对维护
//   - ❌ 不应包含翻译逻辑
//   - ❌ 不应包含业务逻辑
//   - ❌ 不应包含其他语言翻译
//   - ❌ 不应包含动态翻译生成
//
// ============================================================================

/// 中文翻译表
pub const TRANSLATIONS: &[(&str, &str)] = &[
    // 分析器相关
    ("analyze.start", "开始分析工作区依赖关系..."),
    ("analyze.scanning_workspace", "开始扫描工作区: {}"),
    ("analyze.found_packages", "发现 {} 个包"),
    ("analyze.skip_root_package", "跳过根目录的 package.json: {}"),
    (
        "analyze.skip_invalid_package",
        "跳过无效的 package.json: {}",
    ),
    (
        "analyze.circular_detected",
        "检测到循环依赖，无法计算构建阶段",
    ),
    ("analyze.stage_info", "阶段 {}: {} 个包 ({})"),
    ("analyze.completed", "分析完成，耗时 {}ms，共 {} 个阶段"),
    ("analyze.circular_found", "检测到 {} 个循环依赖"),
    ("analyze.circular_detail", "循环依赖 {}: {}"),
    (
        "analyze.circular_warning",
        "警告: 检测到循环依赖，剩余包: {}",
    ),
    // 单包分析相关
    ("analyze.single_package_start", "开始分析单个包: {}"),
    ("analyze.single_package_found", "找到目标包 '{}': {}"),
    ("analyze.single_package_completed", "单包分析完成: {}，耗时 {}ms"),
    // 错误信息
    (
        "error.no_packages_found",
        "未在工作区中找到任何有效的 package.json 文件",
    ),
    ("error.read_package_json", "读取 package.json 失败: {}"),
    ("error.parse_package_json", "解析 package.json 失败: {}"),
    ("error.get_package_dir", "无法获取包目录"),
    ("error.walk_directory", "遍历目录失败"),
    ("error.workspace_not_exist", "工作区路径不存在: {}"),
    ("error.package_not_found", "未找到指定的包: {}"),
    // CLI 相关
    ("cli.analyze.start", "开始分析工作区依赖关系..."),
    // 配置相关
    ("analyze.config_loaded", "已加载配置文件: {}"),
    ("analyze.config_error", "配置文件加载失败: {}"),
    ("analyze.no_config", "未找到配置文件，使用默认设置"),
    // 输出格式相关
    ("output.analysis_result", "分析结果"),
    ("output.total_packages", "总包数: {}"),
    ("output.total_stages", "构建阶段数: {}"),
    ("output.packages_with_deps", "有工作区依赖的包: {}"),
    ("output.analysis_duration", "分析耗时: {}ms"),
    ("output.circular_dependencies", "循环依赖"),
    ("output.no_circular_dependencies", "未发现循环依赖"),
    ("output.build_stages", "构建阶段"),
    ("output.stage_info", "阶段 {} ({} 个包):"),
    ("output.no_workspace_deps", "无工作区依赖"),
    ("output.depends_on", "依赖: {}"),
    ("output.depends_on_count", "依赖 {} 个包:"),
    ("output.path", "路径: {}"),
    ("output.version", "版本: {}"),
    ("output.scripts", "脚本: {}"),
    ("output.package_details", "包详情"),
    ("output.all_dependencies", "所有依赖 ({})"),
    ("output.scripts_detail", "脚本:"),
    (
        "output.usage_tip",
        "提示: 使用 --detail 显示依赖详情，使用 --verbose 查看更多信息，使用 --format json 输出 JSON 格式",
    ),
    // 初始化相关
    ("init.start", "开始初始化 MonoX 配置..."),
    ("init.config_exists", "配置文件已存在: {}"),
    (
        "init.use_force_hint",
        "使用 --force 参数强制覆盖现有配置文件",
    ),
    ("init.config_created", "配置文件已创建: {}"),
    ("init.create_failed", "创建配置文件失败: {}"),
    ("init.next_steps", "接下来您可以编辑配置文件以满足项目需求"),
];
