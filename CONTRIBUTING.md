# Contributing

Thank you for considering contributing to problem-details-rs!

## Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b my-feature`
3. Make your changes
4. Run the test suite: `cargo test --all --all-features`
5. Run lints: `cargo clippy --all-targets --all-features -- -D warnings`
6. Run format check: `cargo fmt --all -- --check`
7. Submit a pull request

## Guidelines

- Write tests for new functionality.
- Keep the core crate dependency-free beyond `serde` and `serde_json`.
- Follow existing code style and conventions.
- Update documentation for public API changes.
- Keep commits focused and write clear commit messages.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
