#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

// 检测包管理器
function detectPackageManager() {
  const { npm_config_user_agent } = process.env;
  
  if (npm_config_user_agent) {
    if (npm_config_user_agent.includes('pnpm')) return 'pnpm';
    if (npm_config_user_agent.includes('yarn')) return 'yarn';
    if (npm_config_user_agent.includes('npm')) return 'npm';
  }
  
  // 回退检测
  try {
    execSync('pnpm --version', { stdio: 'ignore' });
    return 'pnpm';
  } catch {}
  
  try {
    execSync('yarn --version', { stdio: 'ignore' });
    return 'yarn';
  } catch {}
  
  return 'npm'; // 默认
}

// 平台到包名的映射
function getPlatformPackage() {
  const platform = os.platform();
  const arch = os.arch();
  
  const platformKey = `${platform} ${arch}`;
  const packageMap = {
    'darwin arm64': '@monoxon/darwin-arm64',
    'darwin x64': '@monoxon/darwin-x64',
    'linux arm64': '@monoxon/linux-arm64',
    'linux x64': '@monoxon/linux-x64'
  };

  const packageName = packageMap[platformKey];
  if (!packageName) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }
  
  return { packageName, subpath: 'monox' };
}

function installBinary() {
  const { packageName } = getPlatformPackage();
  
  try {
    // 首先尝试从已安装的平台特定包中找到二进制文件
    const binaryPath = require.resolve(`${packageName}/monox`);
    console.log(`[monox] Found platform package: ${packageName}`);
    return;
  } catch (error) {
    console.warn(`[monox] Platform package "${packageName}" not found.`);
    console.warn('This can happen if you use the "--no-optional" flag.');
    console.warn('Attempting to install the platform package...');
    
    // 回退：尝试手动安装平台包
    try {
      const { execSync } = require('child_process');
      const packageVersion = require('./package.json').optionalDependencies[packageName] || 'latest';
      
      console.log(`[monox] Installing ${packageName}@${packageVersion}...`);
      execSync(`npm install ${packageName}@${packageVersion}`, { 
        stdio: 'inherit',
        cwd: __dirname 
      });
      
      // 验证安装是否成功
      require.resolve(`${packageName}/monox`);
      console.log(`[monox] Successfully installed ${packageName}`);
    } catch (installError) {
      console.error(`[monox] Failed to install ${packageName}: ${installError.message}`);
      console.error('\nPlease try:');
      console.error(`1. npm install ${packageName}`);
      console.error('2. Check if your platform is supported');
      console.error('3. Install without --no-optional flag');
    }
  }
}

// 只在直接运行时执行安装
if (require.main === module) {
  installBinary();
}

module.exports = installBinary;
