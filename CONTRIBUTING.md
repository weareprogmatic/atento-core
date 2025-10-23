# Contributing to Atento Core

Thank you for your interest in contributing to Atento Core! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you are expected to uphold this code. Please report unacceptable behavior to atento@weareprogmatic.com.

## Contributor License Agreement (CLA)

**Important**: All contributors must sign our [Contributor License Agreement](CLA.md) before their contributions can be accepted. This is required to ensure clear intellectual property ownership and protect both contributors and the project.

To sign the CLA, add your name to [CLA.md](CLA.md) in your first pull request.

## Governance

This project follows a governance model inspired by SQLite. See [GOVERNANCE.md](GOVERNANCE.md) for details about decision-making and project leadership.

## Getting Started

### Prerequisites

- Rust 1.85.0 or later (for Edition 2024 support)
- Git

### Setting Up Your Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/atento-core.git
   cd atento-core
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/weareprogmatic/atento-core.git
   ```
4. Install development tools:
   ```bash
   make install-tools
   ```

## Development Workflow

### Running Tests

```bash
# Run all tests
make test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Code Quality Checks

```bash
# Format code
make format

# Check formatting
make format-check

# Run clippy
make clippy

# Run all checks including coverage
make pre-commit
```

### Quick Validation

```bash
# Fast checks without coverage
make check
```

## Making Changes

### Creating a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### Commit Guidelines

- Write clear, concise commit messages
- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Reference issues and pull requests liberally after the first line

Example:
```
Add workflow validation for YAML files

- Implement schema validation
- Add error reporting for invalid workflows
- Include tests for edge cases

Fixes #123
```

### Code Style

- Follow Rust standard conventions
- Run `make format` before committing
- Ensure `make clippy` passes without warnings
- Maintain or improve code coverage (target: 80%+)

## Submitting Changes

### Pull Request Process

1. Update your fork with the latest upstream changes:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. Run all checks:
   ```bash
   make pre-commit
   ```

3. Push to your fork:
   ```bash
   git push origin your-branch-name
   ```

4. Open a Pull Request on GitHub with:
   - Clear title describing the change
   - Description of what changed and why
   - Reference to related issues
   - Screenshots/examples if applicable

5. Address review feedback:
   - Make requested changes
   - Push new commits to the same branch
   - Request re-review when ready

### Pull Request Checklist

- [ ] Code follows the project's style guidelines
- [ ] All tests pass (`make test`)
- [ ] New tests added for new functionality
- [ ] Code coverage maintained or improved
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated (for significant changes)
- [ ] Commits are clean and well-described

## Testing

### Unit Tests

Tests are located in `src/tests/`. Each module should have corresponding test coverage.

```bash
cargo test --lib
```

### Integration Tests

Integration tests are in the `tests/` directory.

```bash
cargo test --test integration
```

### QA Smoke Tests

```bash
# Run all QA tests
make qa

# Platform-specific tests
make qa-unix
make qa-windows

# Validate workflow files
make qa-validate
```

## Documentation

- Add doc comments for public APIs
- Update README.md for user-facing changes
- Keep inline comments for complex logic
- Update CHANGELOG.md for notable changes

## Need Help?

- Open an issue for bugs or feature requests
- Join discussions in existing issues
- Email atento@weareprogmatic.com for questions

## License

By contributing, you agree that your contributions will be licensed under both the MIT License and Apache License 2.0, matching the project's dual-license.
