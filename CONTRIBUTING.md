# Contributing to Pharos

Thanks for your interest in contributing! Here's how to get started.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- A JavaScript project with a `yarn.lock` to test against

## Getting Started

```bash
# Clone the repo
git clone https://github.com/VasilisK/pharos.git
cd pharos

# Build
cargo build

# Run locally
cargo run -- <package>@<version> --path /path/to/js/project
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test search::tests
cargo test lockfile::tests
cargo test registry::tests

# Run a specific test
cargo test test_package_exists_found
```

## Project Structure

```
src/
├── main.rs       # CLI entry point, argument parsing, output formatting
├── lockfile.rs   # Lockfile discovery and parsing
├── search.rs     # Package search and dependency chain analysis
├── registry.rs   # npm registry lookups and caching
└── utils.rs      # Shared utilities
testdata/         # Test fixture files (yarn.lock samples)
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common issues
- Write tests for new functionality — test fixtures go in `testdata/`

## Submitting a Pull Request

1. Fork the repo
2. Create a branch (`git checkout -b my-feature`)
3. Make your changes
4. Run `cargo test` and `cargo fmt`
5. Commit with a clear message
6. Open a PR against `main`

## Reporting Bugs

Use the [bug report template](https://github.com/VasilisK/pharos/issues/new?template=bug_report.md) to file an issue. Include:

- The command you ran
- The error or unexpected output
- Your OS and Rust version (`rustc --version`)

## Feature Requests

Open an issue using the [feature request template](https://github.com/VasilisK/pharos/issues/new?template=feature_request.md). Describe the problem you're trying to solve and any ideas for how it could work.