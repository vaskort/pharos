# Pharos 🏛️

Trace vulnerable JavaScript dependencies through your dependency tree. Like `yarn why` / `npm explain`, but shows the full chain and suggests which parent package to update.

Pharos scans Yarn `yarn.lock` files and npm `package-lock.json` v2/v3 files.

## Install

```bash
npx pharos-cli@latest <package>@<version>
```

Or install globally:

```bash
npm install -g pharos-cli
```

## Usage

```bash
# Check current directory for yarn.lock or package-lock.json
pharos minimist@1.2.5

# Check specific project
pharos qs@6.13.0 -p ./my-app

# Search recursively
pharos semver@7.0.0 -p ~/projects -r
```

### Options

- `-p, --path <PATH>` — Directory to search (default: current)
- `-r, --recursive` — Search subdirectories

## Supported Lockfiles

- `yarn.lock`
- `package-lock.json` v2/v3

When a directory contains more than one supported lockfile, Pharos checks each one and prints a separate result section per file.

## Example Output

```
════════════════════════════════════════════════════════════
📁 ./package-lock.json
════════════════════════════════════════════════════════════
  ✓ Found qs@6.13.0

  ── Chain 1 ──
  qs@6.13.0 (requested as 6.13.0)
    -> body-parser@1.20.3 (requested as 1.20.3)
    -> express@4.21.2

 Fix path:
  body-parser >= 1.20.4
  express >= 5.0.0
  → Recommended: Update express to >= 5.0.0
```

## Limitations

- npm `package-lock.json` v1 parsing is not supported yet
- Fix suggestions rely on the public npm registry — private packages in the chain may not have upgrade recommendations

## License

MIT
