# Pharos 🏛️

**Deterministic dependency remediation for developers and AI coding agents.**

[![CI](https://github.com/vaskort/pharos/actions/workflows/ci.yml/badge.svg)](https://github.com/vaskort/pharos/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/pharos-cli.svg)](https://www.npmjs.com/package/pharos-cli)
[![Docs](https://img.shields.io/badge/docs-pharos-00846a.svg)](https://pharos-cli.io/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

![Pharos tracing a vulnerable qs version back to the packages that own the fix](https://raw.githubusercontent.com/vaskort/pharos/main/.github/pharos-demo.gif)

Pharos explains why a known vulnerable JavaScript package version is present in your lockfile.

Give Pharos an exact `package@version` from `yarn audit`, `npm audit`, Dependabot, Snyk, a CVE report, or the vulnerability scanner you already use. It scans your `yarn.lock` or `package-lock.json`, traces the dependency chain that introduced that version, and suggests candidate parent package upgrades.

Pharos is not a vulnerability scanner. It is a lockfile explanation tool for the moment after you already know which package version you need to investigate.

Documentation: https://pharos-cli.io/

## What It Does

- Finds an exact package version in supported JavaScript lockfiles
- Shows the chain of packages that brought that version into the project
- Prints both resolved versions and the parent-requested version ranges
- Suggests candidate parent upgrades using public npm registry metadata
- Verifies remediation paths against an exact fixed version or safe semver range
- Prints exact manifest or override instructions without modifying project files
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

# Prove that the proposed path only permits fixed releases
pharos qs@6.13.0 --fixed ">=6.14.0 <7"

# Trace chains without contacting the npm registry
pharos qs@6.13.0 --no-registry
```

### Options

- `-p, --path <PATH>` - Directory to search (default: current directory)
- `-r, --recursive` - Search subdirectories for additional lockfiles
- `--json` - Print machine-readable JSON instead of human-readable text
- `--fixed <VERSION_OR_RANGE>` - Verify remediation against a minimum fixed version or complete safe range
- `--no-registry` - Skip registry lookups and print dependency chains only

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

 Owner:
  express from dependencies, requested as ^4.18.0

  ── Chain 1 ──
  qs@6.13.0 (requested as 6.13.0)
    -> body-parser@1.20.3 (requested as 1.20.3)
    -> express@4.21.2

 Verified remediation:
  body-parser >= 1.20.4
  express >= 5.0.0
  → semver verified: express 4.21.2 → 5.0.0
    Change package.json dependencies.express from "^4.18.0" to "^5.0.0"
    Run npm install
```

In each chain:

- `package@version` is the resolved package version in the lockfile
- `requested as` is the version range requested by the parent package
- `Owner` is the top-level package declaration from the sibling `package.json`, when available
- `semver verified` means every dependency range in the proposed path is contained in the supplied safe range
- Without `--fixed`, registry suggestions are labeled as unverified candidates

## JSON Output

Use `--json` when another tool needs to consume Pharos output.

```json
{
  "schema_version": 1,
  "package": {
    "name": "pkg-a",
    "version": "1.0.0",
    "fixed_range": ">=1.0.1"
  },
  "lockfiles": [
    {
      "path": "./yarn.lock",
      "lockfile_type": "yarn",
      "status": "found",
      "chains": [
        {
          "target_locator": "pkg-a@1.0.0",
          "links": [],
          "owner": {
            "name": "pkg-a",
            "dependency_type": "dependencies",
            "requested_as": "1.0.0"
          },
          "fix_path": [{
            "package": "pkg-a",
            "minimum_version": "1.0.1"
          }],
          "recommended": {
            "package": "pkg-a",
            "minimum_version": "1.0.1"
          },
          "remediation": {
            "status": "semver_verified",
            "primary_action": {
              "kind": "direct_update",
              "verification": "semver_verified",
              "package": "pkg-a",
              "current_version": "1.0.0",
              "target_version": "1.0.1",
              "manifest_section": "dependencies",
              "requested_as": "1.0.1",
              "instructions": [
                "Change package.json dependencies.pkg-a from \"1.0.0\" to \"1.0.1\"",
                "Run yarn install",
                "Rerun pharos pkg-a@1.0.0 --fixed '1.0.1'"
              ]
            },
            "alternatives": []
          },
          "warnings": []
        }
      ]
    }
  ]
}
```

`status` is one of `found`, `not_found`, or `error`. Parse errors include an `error` string on the lockfile object.
When a sibling `package.json` exists, `owner` identifies the top-level declaration that owns the chain. If the chain root is not declared there, `owner` is `null`.

## Limitations

- npm `package-lock.json` v1 parsing is not supported yet
- Package ownership is inferred from a `package.json` in the same directory as the lockfile; workspace ownership is not resolved yet
- Fix suggestions rely on the public npm registry; private packages in the chain may not have upgrade recommendations
- Candidate upgrades are suggestions based on package metadata; review and test the resulting dependency changes in your project
- `semver_verified` proves range containment, not application compatibility or a successful install

## License

MIT
