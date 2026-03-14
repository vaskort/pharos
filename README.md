# Pharos 🏛️

Trace vulnerable JavaScript dependencies through your dependency tree. Like `yarn why`, but shows the full chain and suggests which parent package to update.

## Install

```bash
npx pharos-cli <package>@<version>
```

Or install globally:

```bash
npm install -g pharos-cli
```

## Usage

```bash
# Check current directory
pharos minimist@1.2.5

# Check specific project
pharos qs@6.13.0 -p ./my-app

# Search recursively
pharos semver@7.0.0 -p ~/projects -r
```

### Options

- `-p, --path <PATH>` — Directory to search (default: current)
- `-r, --recursive` — Search subdirectories

## Example Output

```
════════════════════════════════════════════════════════════
📁 ./yarn.lock
════════════════════════════════════════════════════════════
  ✓ Found qs@6.13.0

  ── Chain 1 ──
  qs@6.13.0 (requested as 6.13.0)
    → body-parser@1.20.3 (requested as 1.20.3)
    → express@4.21.2 (requested as ^4.18.2)
    → my-app@19.0.2

 Fix path:
  body-parser >= 1.20.4
  express >= 5.0.0
  → Recommended: Update express to >= 5.0.0
```

## Limitations

- Only parses `yarn.lock` files (`package-lock.json` detection is in place, parsing coming soon)
- Fix suggestions rely on the public npm registry — private packages in the chain may not have upgrade recommendations

## License

MIT
