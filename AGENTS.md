# Pharos - Dependency Vulnerability Analyzer

A Rust CLI that traces vulnerable JavaScript dependencies and suggests upgrade paths.

## Architecture

### Modules

- `src/main.rs` - CLI entry point (`clap`), package spec parsing, output formatting, and remediation orchestration
- `src/lockfile.rs` - Lockfile discovery and file loading
- `src/search.rs` - Dependency existence checks and chain traversal
- `src/registry.rs` - npm registry fetching and parent package version cache
- `src/utils.rs` - Shared helper utilities (e.g. semver range cleaning)
- `src/*_tests.rs` - Module-focused unit tests

### Key Dependencies

- `clap` - CLI argument parsing with derive macros
- `ignore` - Directory walking with `.gitignore` support
- `yarn-lock-parser` - Parsing `yarn.lock` files
- `reqwest` (blocking + json) - npm registry HTTP calls
- `serde` - Registry response deserialization
- `semver` - Version comparison for upgrade recommendations
- `colored` - CLI output styling
- TODO: Parse npm `package-lock.json` (detection is implemented, parsing is not)

## Implementation Details

### `LockFileType` enum

Represents lockfile types currently detected:
- `Yarn` -> `yarn.lock`
- `Npm` -> `package-lock.json`

Methods:
- `file_name()` - returns canonical filename
- `from_filename()` - maps filename to enum variant

### `find_lockfiles()`

Uses `ignore::WalkBuilder` to walk project paths:
- Respects `.gitignore`
- Controlled by `recursive` flag:
  - `false` -> `max_depth(1)`
  - `true` -> full recursive walk
- Returns `Vec<(LockFileType, PathBuf)>`

### Parsing lockfiles

- `parse_lockfile()` loads lockfile text from disk
- `yarn.lock` is parsed via `yarn_lock_parser::parse_str()`
- `package-lock.json` is currently detected but skipped with a warning

### Dependency chain search

- `package_exists()` validates that the target package/version exists
- `find_dependency_chains()` walks upward from the vulnerable package to all roots
- Chain entries are represented by `ChainLink { name, version, requested_as }`

### Remediation suggestions

- `find_parent_versions()` fetches/caches npm registry metadata for parent packages in chains
- `show_parent_updates()` computes the smallest non-prerelease parent version that pulls a higher target dependency version
- CLI prints:
  - all discovered chains
  - a per-chain "Fix path"
  - a "Recommended" parent upgrade

### CLI Arguments

- `package` (required) - in `name@version` format (exact version expected)
- `--path` / `-p` (optional, default: `.`) - project path to scan
- `--recursive` / `-r` (optional, default: `false`) - recurse into subdirectories

## Current Implementation Status

✅ Find lockfiles with optional recursion  
✅ Parse `yarn.lock` files  
✅ Validate and parse `name@version` CLI input  
✅ Search lockfiles for exact package/version  
✅ Build and print dependency chains  
✅ Query npm registry and suggest parent upgrade paths  
🚧 Determine direct vs transitive dependency ownership from `package.json` metadata  
⏳ Support npm `package-lock.json` parsing  
⏳ Add override/resolution-specific remediation output

## Development Notes

- User is learning Rust - provide hints and guidance, avoid full end-to-end solutions unless explicitly requested
- Prefer idiomatic pattern matching (`match`, `if let`)
- Keep modules focused and composable
- Add/adjust tests with behavior changes (`cargo test`)
