#!/bin/bash

# 检查参数
if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

NEW_VERSION="$1"
NPM_DIR="$(dirname "$0")/../npm"

echo "Updating npm packages to version: $NEW_VERSION"

# 检查 npm 目录是否存在
if [ ! -d "$NPM_DIR" ]; then
    echo "Error: NPM directory not found: $NPM_DIR"
    exit 1
fi

# 查找所有 package.json 文件并更新版本号（排除 node_modules 目录）
find "$NPM_DIR" -name "package.json" -type f -not -path "*/node_modules/*" | while read -r package_file; do
    echo "Processing: $package_file"
    
    # 创建临时文件
    temp_file=$(mktemp)
    
    # 使用 sed 更新版本号，保持原始格式
    # 首先更新主版本号
    sed "s/^\([[:space:]]*\"version\":[[:space:]]*\"\)[^\"]*\(\".*\)$/\1$NEW_VERSION\2/" "$package_file" > "$temp_file"
    
    # 然后更新 optionalDependencies 中的版本号（匹配 @monoxon/ 开头的包）
    sed -i '' "s/\(\"@monoxon\/[^\"]*\":[[:space:]]*\"\)[0-9]*\.[0-9]*\.[0-9]*\(\"\)/\1$NEW_VERSION\2/g" "$temp_file"
    
    # 检查是否有变化
    if ! cmp -s "$package_file" "$temp_file"; then
        mv "$temp_file" "$package_file"
        echo "✅ Updated: $package_file"
    else
        rm "$temp_file"
        echo "ℹ️ No changes needed: $package_file"
    fi
done

echo "✅ Successfully updated npm packages to version $NEW_VERSION"
