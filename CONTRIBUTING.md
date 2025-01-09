# Contributing to Yazi

Thank you for your interest in contributing to Yazi! We welcome contributions in the form of bug reports, feature requests, documentation improvements, and code changes.

This guide will help you understand how to contribute to the project.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Structure](#project-structure)
3. [Development Setup](#development-setup)
4. [How to Contribute](#how-to-contribute)
5. [Pull Request Process](#pull-request-process)

## Getting Started

### Prerequisites

Before you begin, ensure you have met the following requirements:

- Rust installed on your machine. You can download it from [rustup.rs](https://rustup.rs).
- Familiarity with Git and GitHub.

### Fork the Repository

1. Fork the [Yazi repository](https://github.com/sxyazi/yazi) to your GitHub account.
2. Clone your fork to your local machine:

   ```sh
   git clone https://github.com/<your-username>/yazi.git
   ```

3. Set up the upstream remote:
   ```sh
   git remote add upstream https://github.com/sxyazi/yazi.git
   ```

## Project Structure

A brief overview of the project's structure:

```sh
.
├── assets/             # Assets like images and fonts
├── nix/                # Nix-related configurations
├── scripts/            # Helper scripts used by CI/CD
├── snap/               # Snapcraft configuration
├── yazi-adapter/       # Yazi image adapter
├── yazi-boot/          # Yazi bootstrapper
├── yazi-cli/           # Yazi command-line interface
├── yazi-config/        # Yazi configuration file parser
├── yazi-core/          # Yazi core logic
├── yazi-dds/           # Yazi data distribution service
├── yazi-fm/            # Yazi file manager
├── yazi-plugin/        # Yazi plugin system
├── yazi-proxy/         # Yazi event proxy
├── yazi-scheduler/     # Yazi task scheduler
├── yazi-shared/        # Yazi shared library
├── .github/            # GitHub-specific files and workflows
├── Cargo.toml          # Rust workflow configuration
└── README.md           # Project overview
```

## Development Setup

1. Ensure the latest stable Rust is installed:

   ```sh
   rustc --version
   cargo --version
   ```

2. Build the project:

   ```sh
   cargo build
   ```

3. Run the tests:

   ```sh
   cargo test --workspace
   ```

4. Format the code (requires `rustfmt` nightly):

   ```sh
   rustup component add rustfmt --toolchain nightly
   rustfmt +nightly **/*.rs
   ```

## How to Contribute

### Reporting Bugs

If you find a bug, please file an issue.

### Suggesting Features

If you have a feature request, please file an issue.

### Improving Documentation

Yazi's documentation placed at [yazi-rs/yazi-rs.github.io](https://github.com/yazi-rs/yazi-rs.github.io), contributions related to documentation need to be made within this repository.

### Submitting Code Changes

1. Create a new branch for your changes:

   ```sh
   git checkout -b your-branch-name
   ```

2. Make your changes. Ensure that your code follows the project's [coding style](https://github.com/sxyazi/yazi/blob/main/rustfmt.toml) and passes all tests.
3. Commit your changes with a descriptive commit message:

   ```sh
   git commit -m "feat: an awesome feature"
   ```

4. Push your changes to your fork:
   ```sh
   git push origin your-branch-name
   ```

## Pull Request Process

1. Ensure your fork is up-to-date with the upstream repository:

   ```sh
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. Rebase your feature branch onto the main branch:

   ```sh
   git checkout your-branch-name
   git rebase main
   ```

3. Create a pull request to the `main` branch of the upstream repository. Follow the pull request template and ensure that:
   - Your code passes all tests and lints.
   - Your pull request description clearly explains the changes and why they are needed.
4. Address any review comments. Make sure to push updates to the same branch on your fork.
