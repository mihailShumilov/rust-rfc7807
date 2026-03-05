# Contributing

Thank you for considering contributing to rust-rfc7807!

## Getting Started

1. Fork the repository.
2. Create a feature branch: `git checkout -b my-feature`
3. Make your changes.
4. Run checks:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   cargo doc --workspace --no-deps
   ```
5. Submit a pull request.

## Guidelines

- Write tests for new functionality.
- Keep the core crate dependency-free beyond `serde` and `serde_json`.
- Follow existing code style and naming conventions.
- Update documentation for any public API changes.
- Keep commits focused with clear commit messages.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
