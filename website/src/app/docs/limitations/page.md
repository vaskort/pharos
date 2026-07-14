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
- Semver verification proves range containment, not application compatibility or installation success.
- Override and resolution fallbacks can expose incompatibilities and must be tested.

Planned improvements include workspace ownership, pnpm and npm v1 lockfiles, and audit-report ingestion.
