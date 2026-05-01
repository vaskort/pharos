---
title: CLI reference
---

Command syntax and options for the Pharos CLI. {% .lead %}

## Command

```bash
pharos <package>@<version> [options]
```

The package spec must include an exact version.

## Options

| Option | Description |
| --- | --- |
| `-p, --path <PATH>` | Directory to search. Defaults to the current directory. |
| `-r, --recursive` | Search subdirectories for additional lockfiles. |
| `--json` | Print a machine-readable JSON report instead of human-readable text. |

## Examples

Check the current directory:

```bash
pharos minimist@1.2.5
```

Check a specific project:

```bash
pharos qs@6.13.0 --path ./my-app
```

Search nested projects:

```bash
pharos semver@7.0.0 --path ~/projects --recursive
```

Emit JSON:

```bash
pharos qs@6.13.0 --path ./my-app --json
```
