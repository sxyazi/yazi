# Contributing to Yazi

Thank you for your interest in contributing to Yazi! We welcome contributions in the form of bug reports, feature requests, documentation improvements, and code changes. This guide will help you understand how to contribute to the project.
## Table of Contents
1. [Getting Started](#getting-started)
2. [Project Structure](#project-structure)
3. [Development Setup](#development-setup)
4. [How to Contribute](#how-to-contribute)
5. [Pull Request Process](#pull-request-process)
6. [Communication](#communication)

## Getting Started
### Prerequisites
Before you begin, ensure you have met the following requirements:
- Rust installed on your machine. You can download it from [rustup.rs](https://rustup.rs).
- Familiarity with Git and GitHub.

### Fork the Repository
1. Fork the [yazi repository](https://github.com/sxyazi/yazi) to your GitHub account.
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
```bash
yazi/
├── assets/             # Assets like images and fonts
├── nix/                # Nix-related configurations
├── scripts/            # Helper scripts
├── snap/               # Snapcraft configuration
├── yazi-adaptor/       # Adaptor module
├── yazi-boot/          # Boot module
├── yazi-cli/           # CLI module
├── yazi-config/        # Configuration module
├── yazi-core/          # Core functionalities
├── yazi-dds/           # Data distribution service
├── yazi-fm/            # File management module
├── yazi-plugin/        # Plugin system
├── yazi-proxy/         # Proxy module
├── yazi-scheduler/     # Task scheduler
├── yazi-shared/        # Shared utilities
├── .github/            # GitHub-specific files and workflows
├── Cargo.toml          # Rust project configuration
└── README.md           # Project overview
```
## Development Setup

1. Ensure Rust is installed:
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
   cargo test
   ```
4. Format the code (requires the nightly version of `rustfmt`):
   ```sh
   rustup component add rustfmt --toolchain nightly
   rustfmt +nightly **/*.rs
   ```

   
## How to Contribute
### Reporting Bugs
If you find a bug, please create an issue
### Suggesting Features
If you have a feature request, please create an issue
### Improving Documentation
Yazi's documentation repository is located at [yazi-rs.github.io](https://github.com/yazi-rs/yazi-rs.github.io). Contributions related to documentation need to be made within this repository.

### Submitting Code Changes
1. Create a new branch for your changes:
   ```sh
   git checkout -b feature/your-feature-name
   ```
2. Make your changes. Ensure that your code follows the project's coding standards and passes all tests.
3. Commit your changes with a descriptive commit message:
   ```sh
   git commit -m "Add feature: description of the feature"
   ```
4. Push your changes to your fork:
   ```sh
   git push origin feature/your-feature-name
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
   git checkout feature/your-feature-name
   git rebase main
   ```
3. Create a pull request to the `main` branch of the upstream repository. Follow the pull request template and ensure that:
   - Your code passes all tests and lints.
   - Your pull request description clearly explains the changes and why they are needed.
4. Address any review comments. Make sure to push updates to the same branch on your fork.

## Communication
- Discord Server (English mainly): https://discord.gg/qfADduSdJu
- Telegram Group (Chinese mainly): https://t.me/yazi_rs
- For general discussion, use the [Discussions](https://github.com/sxyazi/yazi/discussions) section.

Thank you for your interest in contributing to Yazi!






