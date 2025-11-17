# Contributing to AIOS

Thank you for your interest in contributing to AIOS! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Bugs

- Use GitHub Issues to report bugs
- Include detailed information about the bug
- Provide steps to reproduce
- Include system information (OS, hardware, etc.)

### Suggesting Features

- Use GitHub Discussions for feature suggestions
- Clearly describe the feature and its use case
- Explain why it would be valuable to AIOS

### Contributing Code

1. **Fork the repository**
2. **Create a branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes**
4. **Write or update tests** (if applicable)
5. **Ensure code follows style guidelines**
6. **Commit your changes** (`git commit -m 'Add amazing feature'`)
7. **Push to your fork** (`git push origin feature/amazing-feature`)
8. **Open a Pull Request**

## Contributor License Agreement (CLA)

Before we can accept your contributions, you must sign our Contributor License Agreement (CLA). This ensures that:

- You grant CUI Labs (Pte.) Ltd., Singapore the right to use your contributions
- You confirm that you have the right to grant these rights
- You understand that your contributions will be licensed under the MIT License

**To sign the CLA:**
1. Read the [CLA document](CLA.md)
2. Sign it electronically or print and sign
3. Submit it via GitHub or email to contact@cuilabs.io

## Development Setup

See [README.md](README.md) for development setup instructions.

## Coding Standards

### Rust (Kernel)

- Follow Rust style guidelines
- Use `cargo fmt` to format code
- Use `cargo clippy` to check for issues
- Write tests for new features
- Document public APIs

### TypeScript (Runtime/Services)

- Follow TypeScript best practices
- Use ESLint/Prettier
- Write tests using your preferred testing framework
- Document public APIs

## Commit Messages

- Use clear, descriptive commit messages
- Reference issue numbers when applicable
- Follow conventional commit format when possible

Example:
```
feat(kernel): Add agent lifecycle hooks

- Implement spawn, clone, merge, split operations
- Add lifecycle event bus integration
- Add tests for lifecycle operations

Fixes #123
```

## Pull Request Process

1. **Update documentation** if needed
2. **Add tests** for new features
3. **Ensure all tests pass**
4. **Update CHANGELOG.md** if applicable
5. **Request review** from maintainers
6. **Address review feedback**

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

- GitHub Discussions: For questions and discussions
- GitHub Issues: For bug reports and feature requests
- Email: contact@cuilabs.io
- Website: https://cuilabs.io

Thank you for contributing to AIOS!

