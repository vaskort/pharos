---
title: Getting started
---

Pharos explains why a known vulnerable JavaScript package version is in your lockfile, and finds the package that owns the fix. {% .lead %}

{% quick-links %}

{% quick-link title="Understanding output" icon="presets" href="/docs/understanding-output" description="Read chains, owners, fix paths, and recommendations." /%}

{% quick-link title="JSON reports" icon="plugins" href="/docs/ci-json" description="Wire Pharos into CI with the structured `--json` output." /%}

{% quick-link title="CLI reference" icon="theming" href="/docs/cli" description="Every flag and example in one place." /%}

{% quick-link title="Supported lockfiles" icon="installation" href="/docs/lockfiles" description="What Pharos can parse today." /%}

{% /quick-links %}

Pass any exact `package@version` reported by Dependabot, Snyk, `npm audit`, `yarn audit`, or a CVE. Pharos walks the lockfile, traces the chain, and tells you which top-level dependency owns the fix.

---

## Run without installing

```bash
npx pharos-cli@latest request@2.88.2 --path ./my-app
```

The package argument must include an exact version:

```bash
pharos <package>@<version>
```

## Install globally

```bash
npm install -g pharos-cli
pharos request@2.88.2 --path ./my-app
```

## Choose a project

By default, Pharos scans the current directory:

```bash
pharos minimist@1.2.5
```

Pass `--path` to scan another project:

```bash
pharos qs@6.13.0 --path ./apps/web
```

Use `--recursive` when you want to scan multiple nested projects:

```bash
pharos semver@7.0.0 --path ~/projects --recursive
```

## What to do next

After Pharos finds a chain, read the owner and fix path in the output. The owner is the top-level dependency declaration in the sibling `package.json` that likely needs to change.

{% callout title="Not a vulnerability scanner" %}
Pharos doesn't maintain an advisory database. Use `yarn audit`, `npm audit`, Dependabot, or your existing scanner to find vulnerabilities, then hand the exact `package@version` to Pharos to explain and remediate.
{% /callout %}
