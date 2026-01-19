#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

// Determine which binary to use based on platform
function getBinaryPath() {
  const platform = process.platform;
  const arch = process.arch;

  let binaryName;

  if (platform === 'win32') {
    binaryName = 'pharos-win.exe';
  } else if (platform === 'darwin') {
    binaryName = arch === 'arm64' ? 'pharos-macos-arm64' : 'pharos-macos';
  } else if (platform === 'linux') {
    binaryName = 'pharos-linux';
  } else {
    console.error(`Unsupported platform: ${platform}`);
    process.exit(1);
  }

  return path.join(__dirname, '..', 'binaries', binaryName);
}

const binaryPath = getBinaryPath();

// Execute the binary with all arguments
const child = spawn(binaryPath, process.argv.slice(2), { stdio: 'inherit' });

child.on('exit', (code) => process.exit(code));
