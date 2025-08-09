# 从 Cargo.toml 中提取版本号
VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '=' -f 2 | tr -d ' "')

echo "Extracted version: $VERSION"

# 更新 package.json 中的版本号
if [ -f package.json ]; then
  echo "Updating package.json version to: $VERSION"
  jq --arg v "$VERSION" '.version = $v' package.json > package.json.tmp && \
  mv package.json.tmp package.json
  echo "package.json updated successfully"
fi

# 更新 CHANGELOG.md
git cliff -o CHANGELOG.md --tag $VERSION && \
git add CHANGELOG.md package.json