# ConfKit CLI 发布流程（简版）

## 1. 版本管理

- 更新 `Cargo.toml` 版本号
- 更新 `CHANGELOG`

```sh
cargo release patch --execute --no-confirm --no-publish
cargo release minor --execute --no-confirm --no-publish
cargo release major --execute --no-confirm --no-publish
```

## 2. 编译

- MacOS: `cargo build --release --target x86_64-apple-darwin`
- Linux: `cargo build --release --target x86_64-unknown-linux-gnu`
- 产物路径：
  - MacOS: `target/x86_64-apple-darwin/release/confkit`
  - Linux: `target/x86_64-unknown-linux-gnu/release/confkit`

## 3. 产物分发

- 上传二进制文件至 GitHub Releases 或其他分发平台
- 附加校验和（SHA256）
