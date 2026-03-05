# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-05

### Added

- `problem-details` core crate (v0.1.0):
  - RFC 7807 `Problem` type with builder API.
  - Extension fields via `serde_json::Map<String, Value>`.
  - `ValidationItem` struct and `Problem::validation()` constructor.
  - `Problem::push_error()` / `Problem::push_error_code()` for field-level errors.
  - Error codes via `.code()` extension key.
  - Trace correlation via `.trace_id()` / `.request_id()` extension keys.
  - `InternalCause` storage (never serialized to JSON).
  - `.with_cause()` for attaching `dyn Error` causes.
  - `IntoProblem` trait for domain error mapping.
  - `APPLICATION_PROBLEM_JSON` content-type constant.
- `problem-details-axum` integration crate (v0.1.0):
  - `IntoResponse` implementation for `Problem`.
  - `ApiError` enum with safe 500 handling.
  - `attach_trace()` helper.
  - Optional `tracing` feature.
- Example Axum application with integration tests.
- GitHub Actions CI and release workflows.
