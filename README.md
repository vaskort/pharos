# Pharos 🏛️

**Pharos** (lighthouse in Greek) is a command-line tool that helps you upgrade vulnerable JavaScript packages by analyzing dependency chains and finding the minimum parent package versions needed to resolve security issues.

## What is this / Motivation

Existing security scanners tell you *what* is vulnerable, but not *why* it's there or *how* to fix it. When you find a vulnerable transitive dependency, you're left wondering:

- How did this package end up in my lockfile?
- Which of my dependencies pulled it in?
- Should I add a resolution override, or can I just update a parent?
- If I update a parent, which version actually fixes the vulnerability?

**Pharos answers these questions** by:
1. Visualizing complete dependency chains showing how a vulnerable package ended up in your project
2. Analyzing the npm registry to find which parent package versions would resolve the vulnerability
3. Providing concrete, actionable recommendations for fixing issues

## Features

- 🔍 **Dependency Chain Analysis**: Traces all paths from a vulnerable package back to your direct dependencies
- 🔄 **Smart Version Resolution**: Queries npm registry to find minimum parent versions that fix vulnerabilities
- 📦 **Multi-lockfile Support**: Can analyze multiple lockfiles in a project (with `--recursive` flag)
- 🎨 **Clear Visual Output**: Color-coded, easy-to-read output showing chains and fix paths
- ⚡ **Fast**: Written in Rust for performance and reliability
- 💾 **Registry Caching**: Caches npm registry queries to avoid redundant API calls

## Installation

### Prerequisites
- Rust toolchain (1.70+)

### From Source
```bash
git clone <repository-url>
cd pharos
cargo build --release
```

The compiled binary will be available at `target/release/pharos`.

### Add to PATH (Optional)
```bash
# macOS/Linux
cp target/release/pharos /usr/local/bin/

# Or add to your PATH
export PATH="$PATH:/path/to/pharos/target/release"
```

## Usage

### Basic Usage
```bash
pharos <package>@<version>
```

Example:
```bash
pharos lodash@4.17.19
```

### Options
- `--path <PATH>` or `-p <PATH>`: Specify the directory to search for lockfiles (default: current directory)
- `--recursive` or `-r`: Search for lockfiles recursively in subdirectories

### Examples

**Analyze a vulnerable package in the current directory:**
```bash
pharos minimist@1.2.5
```

**Analyze across multiple projects:**
```bash
pharos semver@7.0.0 --path ~/projects --recursive
```

**Check a specific project:**
```bash
pharos axios@0.21.1 --path ./my-app
```

## How It Works

1. **Discovery**: Pharos finds all relevant lockfiles (currently supports `yarn.lock`)
2. **Chain Tracing**: For each lockfile, it traces all dependency chains leading to the specified package
3. **Registry Analysis**: Fetches version information from npm registry for parent packages
4. **Fix Path Calculation**: Determines the minimum parent package versions that would resolve the vulnerability
5. **Recommendations**: Presents clear, actionable upgrade paths

## Output Example

```
════════════════════════════════════════════════════════════
📁 ./yarn.lock
════════════════════════════════════════════════════════════
  ✓ Found minimist@1.2.5

  ── Chain 1 ──
  minimist@1.2.5 (requested as ^1.2.5) -> mkdirp@1.0.4 -> webpack@5.0.0

 Fix path:
  mkdirp >= 1.0.5
  → Recommended: Update mkdirp to >= 1.0.5
```

## Current Limitations

- **Yarn only**: Currently only supports `yarn.lock` files (npm support planned)
- **Stable versions**: Skips pre-release versions in analysis
- **npm registry**: Only queries the public npm registry

## Roadmap / TODO

- [ ] Add support for `package-lock.json` (npm)
- [ ] Add support for `pnpm-lock.yaml`
- [ ] Add interactive mode for selecting fix strategies
- [ ] Generate automatic PR/MR descriptions
- [ ] Support for private npm registries
- [ ] Export results to JSON/CSV
- [ ] Integration with popular security scanners (Snyk, npm audit, etc.)
- [ ] Add `--fix` flag to automatically update package.json
- [ ] Batch analysis mode for multiple vulnerabilities
- [ ] Web UI for visualization
- [ ] Add configuration file support (.pharosrc)

## Development

### Building
```bash
cargo build
```

### Running in Development
```bash
cargo run -- <package>@<version>
```

### Running Tests
```bash
cargo test
```

### Project Structure
```
pharos/
├── src/
│   ├── main.rs       # CLI entry point and orchestration
│   ├── lockfile.rs   # Lockfile discovery and parsing
│   ├── search.rs     # Dependency chain searching algorithms
│   ├── registry.rs   # npm registry API client
│   └── utils.rs      # Utility functions
├── Cargo.toml        # Rust dependencies
└── README.md
```

### Key Dependencies
- `clap`: Command-line argument parsing
- `yarn-lock-parser`: Parsing yarn.lock files
- `reqwest`: HTTP client for npm registry queries
- `semver`: Semantic version parsing and comparison
- `colored`: Terminal color output
- `serde`: Serialization/deserialization

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

TBC

## Author

Created with ❤️ to make JavaScript dependency management less painful.