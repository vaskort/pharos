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

Fix path:
  body-parser >= 1.20.4
  express >= 5.0.0
  Recommended: Update express to >= 5.0.0
```

## Chain entries

Each chain starts at the vulnerable package version and walks upward toward the top-level dependency.

`requested as` is the version or range requested by the parent package. This is often more useful than the resolved version alone, because it shows whether the parent pinned the vulnerable version or allowed a wider range.

## Owner

The owner is inferred from the `package.json` next to the lockfile.

For direct dependencies, the owner is the target package itself when it is declared in `package.json`.

For transitive dependencies, the owner is the top package in the chain when it is declared in `package.json`.

## Fix path

The fix path lists parent package versions that appear to request a newer target dependency version.

The recommended upgrade is the highest package in the discovered chain with a candidate upgrade. Treat it as a starting point: update the package, refresh the lockfile, and run your test suite.
