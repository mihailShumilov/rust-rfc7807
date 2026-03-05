# rust-rfc7807

**RFC 7807 Problem Details for HTTP APIs in Rust.**

[![Crates.io](https://img.shields.io/crates/v/rust-rfc7807.svg)](https://crates.io/crates/rust-rfc7807)
[![Documentation](https://docs.rs/rust-rfc7807/badge.svg)](https://docs.rs/rust-rfc7807)
[![CI](https://github.com/mihailShumilov/rust-rfc7807/actions/workflows/ci.yml/badge.svg)](https://github.com/mihailShumilov/rust-rfc7807/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/crates/l/rust-rfc7807.svg)](LICENSE)

## Overview

[RFC 7807](https://www.rfc-editor.org/rfc/rfc7807) defines `application/problem+json`, a standard JSON
format for HTTP API error responses. Instead of every service inventing its own error shape, clients
get a consistent, predictable structure they can parse and act on.

`rust-rfc7807` provides a lightweight Rust implementation with an ergonomic builder API,
safe defaults that prevent leaking internal details in 500 responses, and first-class Axum integration.

## Features

- RFC 7807 compliant `application/problem+json` responses
- Builder API with constructors for common HTTP status codes
- Structured validation errors with field-level detail
- Stable error codes for frontend/client consumption
- Internal cause storage that is **never serialized** to JSON
- Safe 500 defaults -- generic public message, no secret leaks
- Trace and request ID correlation
- Axum `IntoResponse` integration (optional feature)
- Minimal dependencies: core crate requires only `serde` and `serde_json`

## Stability Guarantees

The following are considered **stable** and will not change in 0.x without a major notice:

- **RFC 7807 fields**: `type`, `title`, `status`, `detail`, `instance`
- **Extension keys reserved by this crate**: `code`, `trace_id`, `request_id`, `errors`
- **Validation errors shape**: each item in the `"errors"` array contains `field` (String), `message` (String), and optional `code` (String)
- **Safe 500 behavior**: `Problem::internal_server_error()` always defaults to a generic public message; internal causes are never serialized

## Docs.rs

The canonical API documentation with copy-paste examples lives on docs.rs:

- **Core**: [docs.rs/rust-rfc7807](https://docs.rs/rust-rfc7807)
- **Axum**: [docs.rs/rust-rfc7807-axum](https://docs.rs/rust-rfc7807-axum)

## Workspace

| Crate | Description |
|---|---|
| [`rust-rfc7807`](crates/rust-rfc7807) | Core `Problem` type, builder, traits, serialization |
| [`rust-rfc7807-axum`](crates/rust-rfc7807-axum) | Axum `IntoResponse`, `ApiError` wrapper, trace helpers |

## Example Responses

**404 Not Found**

```json
{
  "type": "https://api.example.com/problems/user-not-found",
  "title": "User not found",
  "status": 404,
  "detail": "No user with ID 42 exists",
  "instance": "/users/42",
  "code": "USER_NOT_FOUND",
  "trace_id": "a3f8e2b1-1c4d-4a5f-9b6e-7d8c9e0f1a2b"
}
```

**422 Validation Error**

```json
{
  "type": "validation_error",
  "title": "Validation failed",
  "status": 422,
  "code": "VALIDATION_ERROR",
  "errors": [
    { "field": "email", "message": "must be a valid email address", "code": "INVALID_EMAIL" },
    { "field": "password", "message": "must be at least 8 characters", "code": "TOO_SHORT" }
  ]
}
```

**500 Internal Server Error**

```json
{
  "title": "Internal Server Error",
  "status": 500,
  "detail": "An unexpected error occurred."
}
```

Internal details (database errors, stack traces, credentials) are **never** included in 500 responses.

## Quick Start (Axum)

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-rfc7807-axum = "0.1"
```

Define a handler that returns `Result<T, ApiError>`:

```rust,ignore
use axum::{routing::get, Router};
use rust_rfc7807::Problem;
use rust_rfc7807_axum::ApiError;

async fn get_user() -> Result<String, ApiError> {
    Err(Problem::not_found()
        .title("User not found")
        .detail("No user with ID 42")
        .code("USER_NOT_FOUND")
        .into())
}

#[tokio::main]
async fn main() {
    let app: Router = Router::new().route("/users/:id", get(get_user));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

The response will have status `404`, content type `application/problem+json`, and a structured JSON body.

## Runnable Example

The `examples/axum-api` directory contains a complete Axum server with 4 routes:

```bash
cargo run -p axum-api
```

Then test with curl:

```bash
# 200 OK
curl -s http://localhost:3000/ok
# {"message":"Hello, world!"}

# 404 Not Found (application/problem+json)
curl -s -i http://localhost:3000/not-found
# HTTP/1.1 404 Not Found
# content-type: application/problem+json
# {"title":"Resource not found","status":404,"detail":"User 42 does not exist","code":"RESOURCE_NOT_FOUND","trace_id":"example-trace-id"}

# 422 Validation Error
curl -s -X POST -H 'Content-Type: application/json' -d '{}' http://localhost:3000/validate
# {"type":"validation_error","title":"Validation failed","status":422,"code":"VALIDATION_ERROR","errors":[...]}

# 500 Internal Server Error (no secrets leaked)
curl -s http://localhost:3000/internal
# {"title":"Internal Server Error","status":500,"detail":"An unexpected error occurred."}
```

## Creating Problems

Use status constructors and the builder API:

```rust
use rust_rfc7807::Problem;

// 404 with all fields
let problem = Problem::not_found()
    .type_("https://api.example.com/problems/user-not-found")
    .title("User not found")
    .detail("No user with ID 42 exists")
    .instance("/users/42")
    .code("USER_NOT_FOUND")
    .trace_id("abc-123");

// Available constructors
let _ = Problem::new(418);              // Any status
let _ = Problem::bad_request();         // 400
let _ = Problem::unauthorized();        // 401
let _ = Problem::forbidden();           // 403
let _ = Problem::not_found();           // 404
let _ = Problem::conflict();            // 409
let _ = Problem::unprocessable_entity();// 422
let _ = Problem::validation();          // 422 with defaults
let _ = Problem::too_many_requests();   // 429
let _ = Problem::internal_server_error(); // 500 with safe defaults
```

## Mapping Domain Errors

Implement the `IntoProblem` trait on your application's error types:

```rust
use rust_rfc7807::{IntoProblem, Problem};

enum AppError {
    UserNotFound(u64),
    EmailTaken(String),
}

impl IntoProblem for AppError {
    fn into_problem(self) -> Problem {
        match self {
            AppError::UserNotFound(id) => Problem::not_found()
                .title("User not found")
                .detail(format!("No user with ID {id}"))
                .code("USER_NOT_FOUND"),
            AppError::EmailTaken(email) => Problem::conflict()
                .title("Email already registered")
                .detail(format!("{email} is already in use"))
                .code("EMAIL_TAKEN"),
        }
    }
}
```

## Validation Errors

Use `Problem::validation()` with `push_error` and `push_error_code`:

```rust
use rust_rfc7807::Problem;

let problem = Problem::validation()
    .push_error_code("email", "must be a valid email address", "INVALID_EMAIL")
    .push_error_code("password", "must be at least 8 characters", "TOO_SHORT")
    .push_error("name", "is required")
    .code("VALIDATION_ERROR");
```

Each error in the `"errors"` array includes:

| Field | Type | Description |
|---|---|---|
| `field` | `String` | Field path (e.g. `"email"`, `"address.zip"`) |
| `message` | `String` | Human-readable description |
| `code` | `Option<String>` | Machine-readable code for client logic |

Frontends can map `field` directly to form inputs and use `code` for i18n or conditional UI.

## Security Notes

500 errors are the most dangerous for information leakage. Database connection strings, stack
traces, and internal identifiers can end up in API responses if error handling is careless.

This crate prevents that by design:

- `Problem::internal_server_error()` defaults to a generic public message.
- `.with_cause()` stores the diagnostic error in a field marked `#[serde(skip)]` -- it never
  appears in JSON output.
- `ApiError::internal()` wraps any error into a safe 500 response automatically.

```rust
use rust_rfc7807::Problem;

let problem = Problem::internal_server_error()
    .with_cause(std::io::Error::other("connection to db:5432 refused"));

// Safe for clients
let json = serde_json::to_string(&problem).unwrap();
assert!(!json.contains("db:5432"));
assert!(json.contains("An unexpected error occurred."));

// Available for server-side logging
let cause = problem.internal_cause().unwrap();
assert!(cause.to_string().contains("db:5432"));
```

## Integrations

### Axum (available now)

`rust-rfc7807-axum` provides:

- `IntoResponse` for `Problem` (sets status code + content type)
- `ApiError` enum for use in `Result<T, ApiError>` handler return types
- `attach_trace()` helper for trace IDs
- Optional `tracing` feature for span-based trace ID extraction

### Planned

- Actix Web integration
- `utoipa` / OpenAPI schema generation
- `validator` crate bridge for automatic field error extraction

## Design Philosophy

- **Standards first.** Follow RFC 7807 faithfully. Extensions are additive, not breaking.
- **Minimal dependencies.** Core crate depends only on `serde` and `serde_json`. Framework integrations are separate crates.
- **Explicit over magic.** No global error registries. No derive macros that hide behavior. You implement `IntoProblem`, you see exactly what maps to what.
- **Secure by default.** 500 errors produce generic messages. Internal causes must be opted into and are never serialized.
- **Predictable output.** `None` fields are omitted. Empty extensions produce no extra keys. What you build is what gets serialized.

## Publishing

Both crates publish automatically when commits land on `main`. To control the version bump type, include a keyword in your commit message:

- **Patch** (default): `fix: handle empty detail field`
- **Minor**: `feat: add actix integration [minor]`
- **Major**: `breaking: change Problem fields [major]`
- **Skip release**: `docs: fix typo [skip release]`

To publish manually:

```bash
# 1. Bump versions in both crates/rust-rfc7807/Cargo.toml and crates/rust-rfc7807-axum/Cargo.toml
# 2. Commit, tag, and push:
git tag v0.2.0
git push origin main --tags
```

Required GitHub secret: `CARGO_REGISTRY_TOKEN` in the `crates-io` environment.

## Roadmap

- [ ] Actix Web integration crate
- [ ] `#[derive(IntoProblem)]` macro for enum error types
- [ ] `utoipa` / OpenAPI schema generation for `Problem`
- [ ] `validator` crate bridge for automatic field error extraction

## MSRV

Minimum supported Rust version: **1.75**.

## Author

[Mykhailo Shumilov](https://www.linkedin.com/in/michael-shumilov-46453737/)

## License

Licensed under the [MIT License](LICENSE).
