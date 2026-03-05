# problem-details-rs

Rust implementation of [RFC 7807 Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc7807) with pragmatic extensions and Axum integration.

## What is RFC 7807?

RFC 7807 defines a standard JSON format for HTTP API error responses using the `application/problem+json` content type. Instead of ad-hoc error formats, every error response follows a predictable structure:

```json
{
  "type": "https://example.com/problems/not-found",
  "title": "Not Found",
  "status": 404,
  "detail": "User 42 does not exist",
  "instance": "/users/42"
}
```

## Why this crate?

- Minimal dependencies (core crate only needs `serde` + `serde_json`).
- Ergonomic builder API with common status constructors.
- First-class support for validation errors, error codes, and trace correlation.
- Security-first: internal error causes are never serialized (no 500 leaks).
- Clean Axum integration via optional feature.

## Crates

| Crate | Description |
|---|---|
| `problem-details` | Core `Problem` type, traits, and serialization |
| `problem-details-axum` | Axum `IntoResponse` + `ApiError` wrapper |

## Quick Start (Axum)

```rust
use axum::{routing::get, Router};
use problem_details::Problem;
use problem_details_axum::ApiError;

async fn not_found_handler() -> Result<String, ApiError> {
    Err(ApiError::from(
        Problem::not_found()
            .with_title("User not found")
            .with_detail("No user with ID 42")
            .with_code("USER_NOT_FOUND"),
    ))
}

let app: Router = Router::new().route("/users/:id", get(not_found_handler));
```

## Mapping Domain Errors

```rust
use problem_details::{IntoProblem, Problem};

enum AppError {
    NotFound(String),
    Forbidden,
}

impl IntoProblem for AppError {
    fn to_problem(&self) -> Problem {
        match self {
            AppError::NotFound(msg) => Problem::not_found()
                .with_detail(msg.clone())
                .with_code("NOT_FOUND"),
            AppError::Forbidden => Problem::forbidden()
                .with_title("Forbidden")
                .with_code("FORBIDDEN"),
        }
    }
}
```

## Validation Errors

```rust
use problem_details::{Problem, ValidationError};

let problem = Problem::unprocessable_entity()
    .with_title("Validation Failed")
    .with_code("VALIDATION_ERROR")
    .with_errors(vec![
        ValidationError::new("email", "must be a valid email")
            .with_code("INVALID_EMAIL"),
        ValidationError::new("name", "is required")
            .with_code("REQUIRED"),
    ]);
```

Produces:

```json
{
  "status": 422,
  "title": "Validation Failed",
  "code": "VALIDATION_ERROR",
  "errors": [
    { "field": "email", "message": "must be a valid email", "code": "INVALID_EMAIL" },
    { "field": "name", "message": "is required", "code": "REQUIRED" }
  ]
}
```

## Security: No Leak on 500

Internal error details are stored separately and **never serialized**:

```rust
use problem_details::Problem;

let problem = Problem::internal_server_error()
    .with_internal_cause("db connection failed: password=secret123");

let json = serde_json::to_string(&problem).unwrap();
assert!(!json.contains("secret123"));  // Safe!
assert!(json.contains("An internal error occurred"));  // Generic public message
```

The `internal_cause()` method provides access for server-side logging:

```rust
# use problem_details::Problem;
# let problem = Problem::internal_server_error()
#     .with_internal_cause("db connection failed");
if let Some(cause) = problem.internal_cause() {
    eprintln!("Internal error: {cause}");
}
```

## Trace Correlation

```rust
use problem_details::Problem;
use problem_details_axum::with_trace;

let problem = with_trace(
    Problem::bad_request().with_title("Bad Request"),
    "trace-abc-123"
);
// JSON will include: "trace_id": "trace-abc-123"
```

## Features

| Feature | Crate | Description |
|---|---|---|
| `axum` | `problem-details` | Enables `IntoResponse` impl (auto-enabled by `problem-details-axum`) |
| `tracing` | `problem-details-axum` | Best-effort trace ID extraction from `tracing` spans |

## MSRV

Minimum supported Rust version: **1.75**.

## License

MIT License. See [LICENSE](LICENSE) for details.
