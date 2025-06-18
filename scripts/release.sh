#!/bin/bash
set -e

echo "🚀 开始发布流程..."

# 检查是否有未提交的更改
if [[ -n $(git status -s) ]]; then
  echo "⚠️  检测到未提交的更改，请先提交或暂存"
  git status -s
  exit 1
fi

# 代码格式检查
echo "📝 检查代码格式..."
cargo fmt --check

# 代码检查
echo "🔍 运行 clippy 检查..."
cargo clippy -- -D warnings

# 运行测试
echo "🧪 运行测试..."
cargo test

# 构建 release 版本
echo "🔨 构建 release 版本..."
cargo build --release

# 检查二进制文件是否存在
if [[ ! -f "target/release/monox" ]]; then
  echo "❌ 二进制文件不存在: target/release/monox"
  exit 1
fi

echo "✅ 所有检查通过，准备发布"
echo "📋 下一步:"
echo "  1. 运行 'pnpm run changeset' 添加变更记录"
echo "  2. 运行 'pnpm run version' 更新版本号并提交"
echo "  3. 运行 'pnpm run release' 发布到 npm 并打标签" 