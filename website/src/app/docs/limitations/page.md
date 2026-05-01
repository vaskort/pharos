---
title: Limitations
---

Pharos is intentionally narrow: it explains one known vulnerable package version inside supported JavaScript lockfiles. {% .lead %}

Current limitations:

- npm `package-lock.json` v1 parsing is not supported yet.
- Workspace ownership is not resolved yet.
- Package ownership is inferred only from the sibling `package.json`.
- Fix suggestions rely on public npm registry metadata.
- Private packages in a chain may not have upgrade recommendations.
- Candidate upgrades should be reviewed and tested in the target project.

Planned improvements include npm v1 lockfile support and remediation output that can propose new requested ranges when recommending parent upgrades.
