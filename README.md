# pharos

## What is this / Motivation
Pharos (lighthouse in Greek) is a tool that helps you upgrade vulnerable JavaScript packages. (first version supports only yarn, sorry!)

Existing security scanners tell you *what* is vulnerable, but not *why* it's there or *how* to fix it. When you find a vulnerable transitive dependency, you're left wondering:

- How did this package end up in my lockfile?
- Which of my dependencies pulled it in?
- Should I add a resolution override, or can I just update a parent?
- If I update a parent, which version actually fixes the vulnerability?

Pharos answers these questions by visualising dependency chains and (soon) showing how parent package versions affect which version of a dependency gets resolved.

## Installation
TBC

## Usage
TBC

## Features
TBC

## Roadmap / TODO
TBC

## Development
TBC
