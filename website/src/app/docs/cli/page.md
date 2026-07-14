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
| `--fixed <VERSION_OR_RANGE>` | Verify remediation against a minimum fixed version or complete safe range. |
| `--no-registry` | Skip npm registry requests and print dependency chains only. |

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

Verify every proposed dependency range excludes the vulnerable release:

```bash
pharos qs@6.13.0 --fixed ">=6.14.0 <7"
```

An exact value such as `--fixed 6.14.0` is normalized to `>=6.14.0`.

Trace chains without registry remediation:

```bash
pharos qs@6.13.0 --no-registry
```
