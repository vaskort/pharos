# Pharos 🏛️

A CLI tool that helps you fix vulnerable JavaScript dependencies by showing you exactly how they got into your project and which parent packages to update.

## The Problem

Security scanners tell you *what* is vulnerable, but not *why* it's there or *how* to fix it. When you find a vulnerable transitive dependency:

- How did this package end up in my lockfile?
- Which of my dependencies pulled it in?
- Should I add a resolution override, or can I just update a parent?
- If I update a parent, which version actually fixes the vulnerability?

**Pharos answers these questions.**

## Installation

Coming soon via npx (temporarily unavailable while setting up npm distribution).

## Usage

```bash
pharos <package>@<version>
```

### Examples

```bash
# Check a vulnerable package in current directory
pharos minimist@1.2.5

# Analyze a specific project
pharos axios@0.21.1 --path ./my-app

# Search recursively through multiple projects
pharos semver@7.0.0 --path ~/projects --recursive
```

### Options
- `--path <PATH>` or `-p <PATH>`: Directory to search for lockfiles (default: current directory)
- `--recursive` or `-r`: Search subdirectories recursively

## Example Output

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

## How It Works

1. Finds all lockfiles in your project
2. Traces dependency chains from the vulnerable package to your direct dependencies
3. Queries npm registry to find which parent versions fix the vulnerability
4. Shows you the minimum version you need to upgrade to

## Limitations

- Currently only supports `yarn.lock` (npm and pnpm support coming soon)
- Only queries public npm registry
- Skips pre-release versions

## Roadmap

- [ ] npm (`package-lock.json`) support
- [ ] pnpm (`pnpm-lock.yaml`) support
- [ ] Interactive mode for choosing fixes
- [ ] JSON output format
- [ ] Private registry support
- [ ] Integration with security scanners

## License

MIT - see [LICENSE](LICENSE)
