# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2025-08-27

### ⚙️ Miscellaneous Tasks

- 整理个平台发布
## [@monoxon/linux-x64@0.2.5] - 2025-08-27

### ⚙️ Miscellaneous Tasks

- Refactory publish
- 添加发布钩子脚本, 发版时更新 npm package.json
- 同步 npm 版本号修改 js 脚本为 sh 脚本, 保证在 rust 容器内正常执行
## [0.2.5] - 2025-08-09

### 🐛 Bug Fixes

- 修复 install.sh

### ⚙️ Miscellaneous Tasks

- Release monox version 0.2.5
## [0.2.4] - 2025-08-09

### ⚙️ Miscellaneous Tasks

- 修复 install.sh
- Release monox version 0.2.4
## [0.2.3] - 2025-08-09

### 🐛 Bug Fixes

- 修复安装执行

### ⚙️ Miscellaneous Tasks

- Release monox version 0.2.3
## [0.2.2] - 2025-08-09

### 🐛 Bug Fixes

- 修复安装执行

### ⚙️ Miscellaneous Tasks

- Release monox version 0.2.2
## [0.2.1] - 2025-08-09

### ⚙️ Miscellaneous Tasks

- 添加 install.sh 文件跟踪
- Release monox version 0.2.1
## [0.2.0] - 2025-08-09

### ⚙️ Miscellaneous Tasks

- 修改下载方案, 根据不同平台从 github assets 下载资源
- Release monox version 0.2.0
## [0.1.1] - 2025-08-09

### ⚙️ Miscellaneous Tasks

- 添加发布相关配置
- Release monox version 0.1.1
## [0.1.0] - 2025-08-09

### 🚀 Features

- *(analyze)* Initialize
- 修改 executor 为多任务并发执行
- Add command exec
- 添加 npm 发包配置和 pnpm 支持
- Optimize npm package keywords for better discoverability
- 添加多包处理机制

### 🐛 Bug Fixes

- *(check|fix|run)* Command check, run and fix
- Perf task executor ui
- 修复 clippy 警告和代码格式问题
- 添加 update, 优化日志打印
- Remove package name prefix from git tags - Add --no-git-tag to changeset publish - Use custom tag script for clean v1.0.0 format - Update release documentation

### 📚 Documentation

- 添加英文文档
- Update todolist

### ⚡ Performance

- 优化 UI 显示
- 优化 ui 和文档

### ⚙️ Miscellaneous Tasks

- Version packages
- Version packages
- 添加 build.rs 保证 Cargo.toml 与 package.json 版本一致, 帮助说明修改为英文
- Version packages
- 优化 tag 命令
- 更新 make 任务, release 文档
- 仓库地址更新
