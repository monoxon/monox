# ConfKit CLI 发布流程（简版）

## 1. 版本管理

- 更新 `Cargo.toml` 版本号
- 更新 `CHANGELOG`

```sh
cargo release patch --execute --no-confirm --no-publish
cargo release minor --execute --no-confirm --no-publish
cargo release major --execute --no-confirm --no-publish

# 发布版本会自动执行
# git cliff -o CHANGELOG.md
```
