#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// 从命令行参数获取新版本号
const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Usage: node update-npm-versions.js <version>');
  process.exit(1);
}

console.log(`Updating npm packages to version: ${newVersion}`);

// 递归查找所有 package.json 文件
function findPackageJsonFiles(dir, files = []) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    
    if (entry.isDirectory()) {
      findPackageJsonFiles(fullPath, files);
    } else if (entry.name === 'package.json') {
      files.push(fullPath);
    }
  }
  
  return files;
}

// 更新 package.json 文件，保持原有格式
function updatePackageJson(filePath) {
  try {
    const originalContent = fs.readFileSync(filePath, 'utf8');
    
    // 解析 JSON，保持原有的格式信息
    const pkg = JSON.parse(originalContent);
    
    // 更新主版本号
    pkg.version = newVersion;
    
    // 更新 optionalDependencies 中的 monox 相关包版本
    if (pkg.optionalDependencies) {
      for (const dep in pkg.optionalDependencies) {
        if (dep.startsWith('@monox/') || dep === 'monox') {
          pkg.optionalDependencies[dep] = newVersion;
        }
      }
    }
    
    // 更新 dependencies 中的 monox 相关包版本
    if (pkg.dependencies) {
      for (const dep in pkg.dependencies) {
        if (dep.startsWith('@monox/') || dep === 'monox') {
          pkg.dependencies[dep] = newVersion;
        }
      }
    }
    
    // 更新 devDependencies 中的 monox 相关包版本
    if (pkg.devDependencies) {
      for (const dep in pkg.devDependencies) {
        if (dep.startsWith('@monox/') || dep === 'monox') {
          pkg.devDependencies[dep] = newVersion;
        }
      }
    }
    
    // 重新序列化，尝试保持原有的缩进格式
    let newContent;
    
    // 检测原始文件的缩进格式
    const indentMatch = originalContent.match(/\n(\s+)/);
    const indent = indentMatch ? indentMatch[1] : '  '; // 默认2个空格
    
    // 使用检测到的缩进格式
    if (indent.includes('\t')) {
      newContent = JSON.stringify(pkg, null, '\t');
    } else {
      newContent = JSON.stringify(pkg, null, indent.length);
    }
    
    // 确保文件以换行符结尾
    if (!newContent.endsWith('\n')) {
      newContent += '\n';
    }
    
    fs.writeFileSync(filePath, newContent);
    console.log(`✅ Updated: ${filePath}`);
    
  } catch (error) {
    console.error(`❌ Error updating ${filePath}:`, error.message);
  }
}

// 主逻辑
const npmDir = path.join(__dirname, '..', 'npm');

if (!fs.existsSync(npmDir)) {
  console.error(`NPM directory not found: ${npmDir}`);
  process.exit(1);
}

// 查找所有 package.json 文件
const packageJsonFiles = findPackageJsonFiles(npmDir);

if (packageJsonFiles.length === 0) {
  console.log('No package.json files found in npm directory');
  process.exit(0);
}

console.log(`Found ${packageJsonFiles.length} package.json files`);

// 更新所有文件
packageJsonFiles.forEach(updatePackageJson);

console.log(`✅ Successfully updated ${packageJsonFiles.length} npm packages to version ${newVersion}`);
