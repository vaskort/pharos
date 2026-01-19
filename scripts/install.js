const fs = require('fs');
const path = require('path');

const binariesDir = path.join(__dirname, '..', 'binaries');

// Make binaries executable on Unix-like systems
if (process.platform !== 'win32') {
  if (fs.existsSync(binariesDir)) {
    const binaries = fs.readdirSync(binariesDir).filter(f => !f.endsWith('.exe'));

    binaries.forEach(binary => {
      const binaryPath = path.join(binariesDir, binary);
      try {
        fs.chmodSync(binaryPath, 0o755);
        console.log(`✓ Made ${binary} executable`);
      } catch (err) {
        console.error(`Failed to make ${binary} executable:`, err.message);
      }
    });
  }
}

console.log('✓ Pharos CLI installed successfully!');
