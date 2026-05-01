---
title: Supported lockfiles
---

Lockfile formats Pharos can parse today. {% .lead %}

Pharos currently supports:

- `yarn.lock`
- `package-lock.json` v2
- `package-lock.json` v3

When a directory contains more than one supported lockfile, Pharos checks each one and prints a separate result section.

## Recursive scans

By default, Pharos only checks the target directory.

Use `--recursive` to discover lockfiles in nested projects:

```bash
pharos request@2.88.2 --path ~/projects --recursive
```

Lockfile discovery respects `.gitignore`.

## Package ownership

Ownership is inferred from the `package.json` next to each lockfile. Workspace ownership is not resolved yet.
