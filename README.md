# Pharos 🏛️

Trace vulnerable JavaScript dependencies through your dependency tree. Like `yarn why`, but shows the full chain and suggests which parent package to update.

## Install

```bash
npx @vaskort/pharos <package>@<version>
```

Or install globally:

```bash
npm install -g @vaskort/pharos
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
  ✓ Found minimist@1.2.5

  ── Chain 1 ──
  minimist@1.2.5 (requested as ^1.2.5) -> mkdirp@1.0.4 -> webpack@5.0.0

  Fix path:
    mkdirp >= 1.0.5
    → Recommended: Update mkdirp to >= 1.0.5
```

## Limitations

- Only supports `yarn.lock` (npm/pnpm coming soon)
- Public npm registry only

## License

MIT