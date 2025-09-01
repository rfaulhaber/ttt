# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ttt` is a Rust command-line utility for checking truth tables and optimizing boolean functions. The project uses Rust 2024 edition with Clap for CLI argument parsing, Miette for error handling, and Thiserror for error types.

## Development Commands

### Building and Running
- `cargo build` - Build the project
- `cargo run` - Run the main binary
- `cargo run --bin ttt` - Explicitly run the ttt binary

### Testing and Linting  
- `cargo test` - Run tests
- `cargo clippy` - Run linting with Clippy
- `cargo nextest run` - Run tests with nextest (if available in dev environment)

### Nix Development
- `nix develop` - Enter development shell with Rust toolchain and dependencies
- `nix build` - Build the project using Nix
- `alejandra` - Format Nix files (available in dev shell)

## Architecture

The project follows a modular Rust binary + library structure:

- **main.rs**: CLI interface using clap, handles argument parsing and output formatting
- **lib.rs**: Main library crate that exposes the `source` and `eval` modules
- **source/**: Source code processing modules:
  - **lexer.rs**: Tokenization of boolean expressions with span tracking for error reporting
  - **parser.rs**: Recursive descent parser with miette error diagnostics
  - **mod.rs**: Module declarations and public API
- **eval.rs**: Expression evaluation, truth table generation, and equivalence checking

The CLI supports multiple commands as documented in README:
- `table` - Generate truth tables from boolean expressions
- `eq` - Check equivalence between two boolean expressions  
- `reduce` - Simplify boolean expressions

### Boolean Expression Grammar
The project implements a flexible grammar supporting:
- Operators: `&&`/`∧`/`and`, `||`/`∨`/`or`, `!`/`¬`/`not`, `->`/`→`, `xor`/`⊻`/`⊕`
- Identifiers: Alphabetic characters (non-keywords)
- Expression structure: `(unary operator)? identifier ((binary operator) expr)?`

## Development Environment

The project uses Nix flakes for reproducible development environments with:
- Latest stable Rust toolchain
- Clippy for linting
- rust-analyzer for IDE support
- cargo-nextest for enhanced testing
- Claude Code integration

Note: The source files in `source/` directory are currently empty and need implementation to match the functionality described in the README.