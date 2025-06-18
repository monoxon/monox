# 📦 MonoX 发布指南

## 🚀 发布流程

### 1. 准备发布

```bash
# 运行发布前检查
pnpm run prepare-release
```

这个命令会：
- ✅ 检查 Git 状态
- 📝 检查代码格式 (`cargo fmt --check`)
- 🔍 运行代码检查 (`cargo clippy`)
- 🧪 运行测试 (`cargo test`)
- 🔨 构建 release 版本

### 2. 添加变更记录

```bash
# 交互式添加变更记录
pnpm run changeset
```

选择变更类型：
- **patch**: 修复 bug
- **minor**: 新增功能
- **major**: 重大变更

### 3. 更新版本

```bash
# 根据变更记录更新版本号
pnpm run version
```

这会：
- 更新 `package.json` 中的版本号
- 生成 `CHANGELOG.md`
- 消费掉 `.changeset` 中的变更记录
- 自动提交版本变更

### 4. 发布到 npm

```bash
# 构建、发布并打标签
pnpm run release
```

这会：
- 构建 release 版本
- 发布到 npm
- 创建 Git 标签
- 推送标签到远程仓库

或者分步执行：

```bash
# 1. 构建
pnpm run build

# 2. 发布
pnpx changeset publish

# 3. 打标签并推送
pnpm run tag
```

## 📋 发布清单

- [ ] 运行 `pnpm run prepare-release`
- [ ] 添加变更记录 `pnpm run changeset`
- [ ] 更新版本 `pnpm run version` (自动提交)
- [ ] 发布 `pnpm run release` (自动打标签并推送)

## ⚡ 快速发布

对于小的补丁版本：

```bash
# 一键发布 patch 版本
pnpm run prepare-release && \
pnpm run changeset -- --patch && \
pnpm run version && \
pnpm run release
```

## 🔧 配置

发布配置在以下文件中：
- `package.json` - npm 包配置
- `.changeset/config.json` - changeset 配置
- `scripts/release.sh` - 发布前检查脚本 