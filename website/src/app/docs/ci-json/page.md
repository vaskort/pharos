---
title: JSON reports
---

Use `--json` when another tool needs to consume Pharos output. {% .lead %}

```bash
pharos qs@6.13.0 --path ./my-app --json
```

The top-level report includes the target package and a list of lockfile results:

```json
{
  "package": {
    "name": "qs",
    "version": "6.13.0"
  },
  "lockfiles": [
    {
      "path": "./package-lock.json",
      "lockfile_type": "npm",
      "status": "found",
      "chains": []
    }
  ]
}
```

## Status values

`status` is one of:

- `found` when the exact package version exists in the lockfile.
- `not_found` when the lockfile was parsed but does not contain that version.
- `error` when the lockfile could not be parsed or analyzed.

Parse errors include an `error` string on the lockfile object.

## CI usage

Pharos is most useful in CI when another step has already identified a vulnerable version. Pass that exact version to Pharos and attach the JSON report to the security finding, pull request, or incident note.
