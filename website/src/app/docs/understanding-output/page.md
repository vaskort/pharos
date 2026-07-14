---
title: Understanding output
---

Pharos prints one report section per lockfile. Each section starts at the vulnerable package and walks upward toward the top-level dependency. {% .lead %}

```text
./package-lock.json
  Found qs@6.13.0

Owner:
  express from dependencies, requested as ^4.18.0

Chain 1
  qs@6.13.0 (requested as 6.13.0)
    -> body-parser@1.20.3 (requested as 1.20.3)
    -> express@4.21.2

Verified remediation:
  body-parser >= 1.20.4
  express >= 5.0.0
  semver verified: express 4.21.2 → 5.0.0
  Change package.json dependencies.express from "^4.18.0" to "^5.0.0"
  Run npm install
```

## Chain entries

Each chain starts at the vulnerable package version and walks upward toward the top-level dependency.

`requested as` is the version or range requested by the parent package. This is often more useful than the resolved version alone, because it shows whether the parent pinned the vulnerable version or allowed a wider range.

## Owner

The owner is inferred from the `package.json` next to the lockfile.

For direct dependencies, the owner is the target package itself when it is declared in `package.json`.

For transitive dependencies, the owner is the top package in the chain when it is declared in `package.json`.

## Remediation

With `--fixed`, Pharos only labels a path `semver verified` when each proposed dependency range is wholly contained in the supplied safe range. This proves the version constraints, but you must still install and test the result.

Without `--fixed`, Pharos labels registry results as candidates. A candidate excludes the exact installed version but may still permit another affected version.

When no owner upgrade can be verified, Pharos prints an npm `overrides` or Yarn `resolutions` fallback. These instructions are advisory; Pharos never edits project files or runs package-manager commands.
