# Pharos 🏛️

[![CI](https://github.com/vaskort/pharos/actions/workflows/ci.yml/badge.svg)](https://github.com/vaskort/pharos/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/pharos-cli.svg)](https://www.npmjs.com/package/pharos-cli)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Pharos explains why a known vulnerable JavaScript package version is present in your lockfile.

Give Pharos an exact `package@version` from `yarn audit`, `npm audit`, Dependabot, Snyk, a CVE report, or the vulnerability scanner you already use. It scans your `yarn.lock` or `package-lock.json`, traces the dependency chain that introduced that version, and suggests candidate parent package upgrades.

Pharos is not a vulnerability scanner. It is a lockfile explanation tool for the moment after you already know which package version you need to investigate.

## What It Does

- Finds an exact package version in supported JavaScript lockfiles
- Shows the chain of packages that brought that version into the project
- Prints both resolved versions and the parent-requested version ranges
- Suggests candidate parent upgrades using public npm registry metadata
- Scans one project or multiple nested projects with `--recursive`

## When To Use It

Use Pharos when a security tool tells you a specific package version is vulnerable and you need to understand who owns the fix.

```bash
# Dependabot, yarn audit, or npm audit reports request@2.88.2
npx pharos-cli@latest request@2.88.2 --path ./my-app
```

The output answers:

- Is this exact version in the lockfile?
- Which parent package requested it?
- Was it requested exactly, or did a wider range resolve to it?
- Which top-level dependency chain owns the vulnerable package?
- Is there a candidate parent upgrade that requests a newer version?

## Why Not yarn audit / npm audit?

Use `yarn audit` or `npm audit` to find vulnerabilities. Use Pharos to explain one known vulnerable `package@version`.

| Tool | Primary job |
| --- | --- |
| `yarn audit` / `npm audit` | Find known vulnerabilities, advisories, severities, and CI audit status |
| Pharos | Explain why a specific vulnerable version is in a lockfile and which parent package may need to change |

Pharos does not maintain a vulnerability database, assign severities, or replace audit tooling. It starts from the vulnerable version those tools report and focuses on dependency ownership and remediation paths.

## Install

Run without installing:

```bash
npx pharos-cli@latest <package>@<version>
```

Or install globally:

```bash
npm install -g pharos-cli
pharos <package>@<version>
```

## Usage

```bash
# Check current directory for yarn.lock or package-lock.json
pharos minimist@1.2.5

# Check a specific project
pharos qs@6.13.0 --path ./my-app

# Search nested projects recursively
pharos semver@7.0.0 --path ~/projects --recursive

# Print machine-readable output for CI or downstream tooling
pharos qs@6.13.0 --path ./my-app --json
```

### Options

- `-p, --path <PATH>` - Directory to search (default: current directory)
- `-r, --recursive` - Search subdirectories for additional lockfiles
- `--json` - Print machine-readable JSON instead of human-readable text

## Supported Lockfiles

- `yarn.lock`
- `package-lock.json` v2/v3

When a directory contains more than one supported lockfile, Pharos checks each one and prints a separate result section per file.

## Example Output

```text
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

In each chain:

- `package@version` is the resolved package version in the lockfile
- `requested as` is the version range requested by the parent package
- `Recommended` is the highest parent in the discovered fix path

## JSON Output

Use `--json` when another tool needs to consume Pharos output.

```json
{
  "package": {
    "name": "pkg-a",
    "version": "1.0.0"
  },
  "lockfiles": [
    {
      "path": "./yarn.lock",
      "lockfile_type": "yarn",
      "status": "found",
      "chains": [
        {
          "links": [],
          "fix_path": [],
          "recommended": null,
          "warnings": []
        }
      ]
    }
  ]
}
```

`status` is one of `found`, `not_found`, or `error`. Parse errors include an `error` string on the lockfile object.

## Limitations

- npm `package-lock.json` v1 parsing is not supported yet
- Fix suggestions rely on the public npm registry; private packages in the chain may not have upgrade recommendations
- Candidate upgrades are suggestions based on package metadata; review and test the resulting dependency changes in your project

## License

MIT
