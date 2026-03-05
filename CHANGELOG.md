# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-05

### Added

- Initial release of `problem-details` core crate.
  - RFC 7807 `Problem` type with builder API.
  - Extension fields support via `BTreeMap<String, serde_json::Value>`.
  - Validation errors (`errors` extension key).
  - Error codes (`code` extension key).
  - Trace correlation (`trace_id`, `request_id` extension keys).
  - Internal cause storage (never serialized).
  - `IntoProblem` trait for domain error mapping.
- Initial release of `problem-details-axum` integration crate.
  - `IntoResponse` implementation for `Problem`.
  - `ApiError` wrapper with safe 500 handling.
  - Trace ID helpers.
  - Optional `tracing` feature.
- Example Axum application with integration tests.
