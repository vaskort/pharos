# Pharos - Dependency Vulnerability Analyzer

A CLI tool to identify and remediate vulnerable dependencies in JavaScript projects.

## Architecture

### Modules

- `src/main.rs` - CLI entry point using `clap`
- `src/lockfile.rs` - Lockfile discovery and parsing logic
- `src/search.rs` - Package search and dependency chain analysis

### Key Dependencies

- `clap` - CLI argument parsing with derive macros
- `ignore` - Directory walking with `.gitignore` support (replaces `walkdir`)
- `yarn-lock-parser` - Parsing `yarn.lock` files
- TODO: Add npm `package-lock.json` parser

## Implementation Details

### LockFileType enum

Represents the two supported lockfile types:
- `Yarn` → `yarn.lock`
- `Npm` → `package-lock.json`

Methods:
- `file_name()` - returns the filename string
- `from_filename()` - creates enum from filename string

### find_lockfiles()

Uses `ignore::WalkBuilder` to recursively walk the project directory:
- Respects `.gitignore` files automatically
- Returns `Vec<(LockFileType, PathBuf)>` of all discovered lockfiles

### Parsing lockfiles

- `yarn.lock` - Uses `yarn_lock_parser::parse_str()` which returns a `Lockfile` with `entries: Vec<Entry>`
- Each `Entry` has: `name`, `version`, `dependencies`, `peer_dependencies`, etc.

### CLI Arguments

- `package` (required) - Package name to search for
- `--path` / `-p` (optional, default: `.`) - Project root path

## Current Implementation Status

✅ Find all lockfiles in project tree
✅ Parse yarn.lock files
🚧 Search for package in lockfiles
⏳ Determine direct vs transitive dependencies (requires reading `package.json`)
⏳ Display dependency chains
⏳ Suggest remediation options based on the dependency chain:
   - Identify which parent dependencies can be upgraded to pull in a safe version
   - Suggest using package manager resolutions/overrides to force a specific version
   - Show the version ranges needed for each option
⏳ Support for npm `package-lock.json` parsing

## Development Notes

- User is learning Rust - provide hints and guidance, don't write full solutions
- Use pattern matching idiomatically (match, if let)
- Keep code modular and organized
