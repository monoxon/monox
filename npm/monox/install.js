#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');

// 平台到包名的映射
function getPlatformPackage() {
  const platform = os.platform();
  const arch = os.arch();
  
  const platformKey = `${platform} ${arch}`;
  const packageMap = {
    'darwin arm64': '@monox/darwin-arm64',
    'darwin x64': '@monox/darwin-x64',
    'linux arm64': '@monox/linux-arm64',
    'linux x64': '@monox/linux-x64'
  };

  const packageName = packageMap[platformKey];
  if (!packageName) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }
  
  return { packageName, subpath: 'monox' };
}

function verifyInstallation() {
  const { packageName } = getPlatformPackage();
  
  try {
    // 验证平台特定包是否已安装
    require.resolve(`${packageName}/monox`);
    console.log(`[monox] Installation verified: ${packageName}`);
  } catch (error) {
    console.warn(`[monox] Warning: Platform package "${packageName}" not found.`);
    console.warn('This may be due to:');
    console.warn('1. Optional dependencies were skipped (--no-optional flag)');
    console.warn(`2. Package "${packageName}" is not available`);
    console.warn('\nmonox will attempt to find the binary at runtime.');
  }
}

// 只在直接运行时执行验证
if (require.main === module) {
  verifyInstallation();
}

module.exports = verifyInstallation;
