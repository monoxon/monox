# MonoX

> 🚀 基于 Rust 的轻量级 monorepo 构建工具

MonoX 是一个专为 monorepo 项目设计的智能构建工具，通过依赖关系分析和任务调度优化，帮助您高效管理多包项目的构建过程。

## ✨ 核心特性

- 🔍 **智能依赖分析** - 自动解析包依赖关系，构建有向无环图
- 📦 **单包分析** - 支持分析指定包及其依赖链，精确定位构建范围
- ⚡ **并发任务执行** - 同阶段包并发构建，最大化 CPU 利用率
- 🛡️ **安全性检查** - 循环依赖检测、版本冲突检查
- 📊 **实时进度显示** - 美观的进度条和任务状态展示
- 🌍 **国际化支持** - 完整的中文/英文双语界面
- 🎯 **灵活配置** - 通过 `monox.toml` 自定义任务和执行策略
- 🔧 **多包管理器支持** - 支持 pnpm、npm、yarn

## 🚀 快速开始

### 安装

```bash
# 从源码构建（需要 Rust 环境）
git clone https://github.com/your-org/monox.git
cd monox
cargo make release

# 将可执行文件添加到 PATH
cp target/release/monox /usr/local/bin/
```

### 初始化配置

在您的 monorepo 项目根目录运行：

```bash
monox init
```

这将创建一个 `monox.toml` 配置文件。

### 基本使用

```bash
# 分析项目依赖关系和构建阶段
monox analyze

# 分析指定包及其依赖链（单包分析）
monox analyze --package @your-org/package-name

# 查看单包的详细依赖信息
monox analyze --package @your-org/package-name --detail --verbose

# 构建所有包（按依赖顺序）
monox run --all --command build

# 运行指定包及其依赖
monox run @your-org/package-name --command build

# 执行预定义任务
monox exec build-all

# 检查项目健康状态
monox check --circular --versions --outdated
```

## 📋 命令参考

### 全局选项

```bash
-v, --verbose           显示详细执行过程
--no-color              禁用彩色输出
--no-progress           禁用进度显示
-j, --max-concurrency   设置最大并发数
--timeout               设置任务超时时间（秒）
--retry                 设置重试次数
--continue-on-failure   失败时继续执行
-C, --workspace-root    指定工作区根目录
-l, --language          设置界面语言 (en_us, zh_cn)
```

### 主要命令

#### `analyze` - 依赖分析
```bash
monox analyze                              # 分析并显示构建阶段
monox analyze --format json               # 输出 JSON 格式
monox analyze --verbose                    # 显示详细依赖关系
monox analyze --package <package-name>    # 分析指定单个包及其依赖链
monox analyze --package <package-name> --detail  # 单包分析显示详细信息
```

#### `run` - 执行命令
```bash
monox run <package> --command <cmd>    # 运行指定包的命令
monox run --all --command <cmd>        # 运行所有包的命令
```

#### `exec` - 执行预定义任务
```bash
monox exec <task-name>           # 执行 monox.toml 中定义的任务
```

#### `check` - 健康检查
```bash
monox check --circular           # 检查循环依赖
monox check --versions           # 检查版本冲突
monox check --outdated           # 检查过期依赖
```

#### `fix` - 问题修复
```bash
monox fix --versions             # 修复版本不一致
```

#### `update` - 依赖更新
```bash
monox update <package>           # 更新指定包的依赖
monox update --all               # 更新所有包的依赖
```

## ⚙️ 配置文件

### monox.toml 配置示例

```toml
[workspace]
root = "."
package_manager = "pnpm"  # pnpm | npm | yarn
ignore = [                # 排除扫描的目录或文件模式
    "dist",
    "build",
    ".git",
    "*.tmp"
]

# 预定义任务
[[tasks]]
name = "build-all"
pkg_name = "*"
desc = "构建所有包"
command = "build"

[[tasks]]
name = "test-system"
pkg_name = "@your-org/system"
desc = "测试系统核心包"
command = "test"

# 执行配置
[execution]
max_concurrency = 4        # 最大并发数
task_timeout = 300         # 任务超时（秒）
retry_count = 0            # 重试次数
continue_on_failure = false # 失败时是否继续

# 输出配置
[output]
show_progress = true       # 显示进度条
verbose = false           # 详细输出
colored = true            # 彩色输出

# 国际化配置
[i18n]
language = "zh_cn"        # 界面语言 (en_us, zh_cn)
```

### 配置参数说明

#### [workspace] - 工作空间
- `root`: 工作目录根路径，默认 "."
- `package_manager`: 包管理器类型，支持 "pnpm"、"npm"、"yarn"
- `ignore`: 排除扫描的目录或文件模式，支持 glob 通配符。注意：`node_modules` 目录始终被排除，无需配置

#### [[tasks]] - 任务定义
- `name`: 任务名称，用于 `monox exec <name>`
- `pkg_name`: 包名，"*" 表示所有包
- `desc`: 任务描述（可选）
- `command`: 执行的命令

#### [execution] - 执行控制
- `max_concurrency`: 最大并发任务数，默认为 CPU 核心数
- `task_timeout`: 单个任务超时时间（秒），默认 300
- `retry_count`: 失败重试次数，默认 0
- `continue_on_failure`: 失败时是否继续，默认 false

#### [output] - 输出控制
- `show_progress`: 是否显示进度条，默认 true
- `verbose`: 是否详细输出，默认 false
- `colored`: 是否彩色输出，默认 true

#### [i18n] - 国际化
- `language`: 界面语言，支持 "en_us"（英语）和 "zh_cn"（简体中文）

## 🌍 国际化支持

MonoX 提供完整的双语支持：

### 语言选择优先级
1. 命令行参数 `--language` 或 `-l`
2. 配置文件 `monox.toml` 中的设置
3. 系统默认（英语）

### 使用示例
```bash
# 使用中文界面
monox analyze -l zh_cn

# 使用英文界面
monox run --all --command build --language en_us
```

## 📦 单包分析功能

MonoX 支持对指定包进行精确的依赖分析，这在大型 monorepo 项目中特别有用：

### 功能特点

- **精确范围**：只分析目标包及其直接依赖链，不包含无关包
- **构建优化**：显示构建目标包所需的最小依赖集合
- **快速诊断**：快速了解特定包的依赖状况
- **多格式输出**：支持表格和 JSON 格式输出

### 使用示例

```bash
# 基本单包分析
monox analyze --package @your-org/components

# 输出示例：
# ◇ 分析结果
# ● 总包数: 1
# ▪ 构建阶段数: 3
# ◦ 有工作区依赖的包: 1
# 
# ▪ 构建阶段
# ─────────────────────────
# 阶段 1 (1 个包):
#   ● @your-org/utils
# 
# 阶段 2 (1 个包):
#   ● @your-org/core
# 
# 阶段 3 (1 个包):
#   ● @your-org/components

# 详细信息模式
monox analyze --package @your-org/components --detail

# JSON 格式输出（便于脚本处理）
monox analyze --package @your-org/components --format json
```

### 适用场景

1. **开发调试**：了解特定包的依赖关系
2. **构建优化**：确定最小构建范围
3. **依赖审查**：检查包的依赖是否合理
4. **CI/CD 优化**：只构建相关的包

## 📊 使用场景

### 典型工作流

1. **项目初始化**
   ```bash
   monox init
   # 编辑 monox.toml 配置文件
   ```

2. **依赖分析**
   ```bash
   # 分析整个工作区
   monox analyze --verbose
   
   # 分析特定包及其依赖链
   monox analyze --package @your-org/core --detail
   ```

3. **健康检查**
   ```bash
   monox check --circular --versions --outdated
   # 确保项目状态良好
   ```

4. **构建执行**
   ```bash
   monox run --all --command build --verbose
   # 按依赖顺序构建所有包
   ```

5. **测试运行**
   ```bash
   monox exec test-all
   # 执行预定义的测试任务
   ```

### 单包分析和调试

```bash
# 分析特定包的依赖关系
monox analyze --package @your-org/core

# 查看单包的详细依赖信息
monox analyze --package @your-org/core --detail --verbose

# 以 JSON 格式输出单包分析结果
monox analyze --package @your-org/core --format json

# 分析多个包（分别执行）
monox analyze --package @your-org/utils
monox analyze --package @your-org/components
```

### 调试和诊断

```bash
# 详细模式：查看构建过程和进度
monox run --all --command build --verbose

# 组合使用：最完整的信息输出
monox analyze --verbose --detail
```

## 🤝 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解开发指南。

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- [设计文档](DESIGN.md)
- [开发贡献指南](CONTRIBUTING.md)
- [更新日志](CHANGELOG.md)
- [问题反馈](https://github.com/your-org/monox/issues) 