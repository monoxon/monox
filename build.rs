use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 告诉 Cargo 当 package.json 改变时重新运行 build script
    println!("cargo:rerun-if-changed=package.json");

    // 读取 package.json
    let package_json_path = "package.json";

    if Path::new(package_json_path).exists() {
        match fs::read_to_string(package_json_path) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(json) => {
                        if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                            // 设置环境变量供编译时使用
                            println!("cargo:rustc-env=PACKAGE_VERSION={}", version);
                            println!("Using package.json version: {}", version);
                        } else {
                            // 如果没有找到版本，回退到 Cargo.toml 版本
                            println!(
                                "cargo:rustc-env=PACKAGE_VERSION={}",
                                env!("CARGO_PKG_VERSION")
                            );
                            println!("Warning: No version found in package.json, using Cargo.toml version");
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse package.json: {}", e);
                        println!(
                            "cargo:rustc-env=PACKAGE_VERSION={}",
                            env!("CARGO_PKG_VERSION")
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read package.json: {}", e);
                println!(
                    "cargo:rustc-env=PACKAGE_VERSION={}",
                    env!("CARGO_PKG_VERSION")
                );
            }
        }
    } else {
        // 如果 package.json 不存在，使用 Cargo.toml 版本
        println!(
            "cargo:rustc-env=PACKAGE_VERSION={}",
            env!("CARGO_PKG_VERSION")
        );
        println!("Warning: package.json not found, using Cargo.toml version");
    }
}
