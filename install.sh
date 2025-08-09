#!/bin/bash

set -e

# 获取最新版本号（无需认证，使用HTML解析方法）
echo "正在获取最新版本号..."
# 使用GitHub的releases/latest页面的重定向获取最新版本
LATEST_VERSION_URL=$(curl -s -L -o /dev/null -w %{url_effective} https://github.com/monoxon/monox/releases/latest)
LATEST_VERSION=$(basename "$LATEST_VERSION_URL")

if [ -z "$LATEST_VERSION" ]; then
    echo "错误: 无法获取最新版本号"
    echo "请检查仓库是否存在: https://github.com/monoxon/monox"
    exit 1
fi

# 移除可能的"v"前缀（如果存在）
CLEAN_VERSION=${LATEST_VERSION#v}

echo "找到最新版本: $LATEST_VERSION"

# 确定系统架构和平台
case $(uname -s) in
    Darwin*)
        OS="apple-darwin"
        if [ "$(uname -m)" = "arm64" ]; then
            ARCH="aarch64"
        else
            ARCH="x86_4"
        fi
        ;;
    Linux*)
        OS="unknown-linux-gnu"
        if [ "$(uname -m)" = "aarch64" ] || [ "$(uname -m)" = "arm64" ]; then
            ARCH="aarch64"
        else
            ARCH="x86_64"
        fi
        ;;
    *)
        echo "错误: 不支持的操作系统: $(uname -s)"
        exit 1
        ;;
esac

BINARY_NAME="monox-$ARCH-$OS"
DOWNLOAD_URL="https://github.com/monoxon/monox/releases/download/$LATEST_VERSION/${BINARY_NAME}"
TARGET_DIR="$PWD"
TARGET_BINARY="$TARGET_DIR/monox"

# 创建目标目录（如果不存在）
mkdir -p "$TARGET_DIR"

# 下载二进制文件
echo "正在下载 monox $LATEST_VERSION (${ARCH}-${OS})..."

# 检查是否支持断点续传
if command -v curl > /dev/null; then
    DOWNLOAD_CMD="curl -L -o \"$TARGET_BINARY\" -C - \"$DOWNLOAD_URL\""
    if ! eval "$DOWNLOAD_CMD"; then
        # 如果断点续传失败，尝试普通下载
        echo "断点续传失败，尝试普通下载..."
        curl -L -o "$TARGET_BINARY" "$DOWNLOAD_URL"
    fi
elif command -v wget > /dev/null; then
    wget -O "$TARGET_BINARY" "$DOWNLOAD_URL"
else
    echo "错误: 需要 curl 或 wget 来下载文件"
    exit 1
fi

# 检查下载是否成功
if [ ! -f "$TARGET_BINARY" ]; then
    echo "错误: 下载失败"
    echo "请手动下载: $DOWNLOAD_URL"
    exit 1
fi

# 设置可执行权限
chmod +x "$TARGET_BINARY"

echo ""
echo "monox $LATEST_VERSION 已成功安装到: $TARGET_BINARY"
echo "请确保将以下目录添加到您的 PATH 环境变量中:"
echo "  export PATH=\"$TARGET_DIR:\$PATH\""
echo ""
echo "或者直接运行:"
echo "  $TARGET_BINARY --help"