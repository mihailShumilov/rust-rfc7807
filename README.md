# problem-details

**RFC 7807 Problem Details for HTTP APIs in Rust.**

[![Crates.io](https://img.shields.io/crates/v/problem-details.svg)](https://crates.io/crates/problem-details)
[![Documentation](https://docs.rs/problem-details/badge.svg)](https://docs.rs/problem-details)
[![License: MIT](https://img.shields.io/crates/l/problem-details.svg)](LICENSE)
[![CI](https://github.com/example/problem-details-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/example/problem-details-rs/actions/workflows/ci.yml)

## Overview

[RFC 7807](https://www.rfc-editor.org/rfc/rfc7807) defines a standard JSON format (`application/problem+json`) for describing errors in HTTP APIs. Instead of every service inventing its own error shape, clients get a consistent, predictable structure they can parse and act on.

`problem-details` provides a lightweight Rust implementation with an ergonomic builder API, safe defaults that prevent leaking internal details in 500 responses, and first-class Axum integration.

## Features

- RFC 7807 compliant `application/problem+json` responses
- Builder API with constructors for common HTTP status codes
- Structured validation errors with field-level detail
- Stable error codes for frontend/client consumption
- Internal cause storage that is **never serialized** to JSON
- Safe 500 defaults — generic public message, no secret leaks
- Trace and request ID correlation
- Axum `IntoResponse` integration (optional feature)
- Minimal dependencies: core crate requires only `serde` and `serde_json`

## Workspace

| Crate | Description |
|---|---|
| [`problem-details`](crates/problem-details) | Core `Problem` type, builder, traits, serialization |
| [`problem-details-axum`](crates/problem-details-axum) | Axum `IntoResponse` impl, `ApiError` wrapper, trace helpers |

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
  "title": "Validation Failed",
  "status": 422,
  "code": "VALIDATION_ERROR",
  "errors": [
    { "field": "email", "message": "must be a valid email address", "code": "INVALID_EMAIL" },
    { "field": "password", "message": "must be at least 8 characters", "code": "TOO_SHORT" }
  ],
  "trace_id": "b4a9c1d2-5e6f-7a8b-9c0d-1e2f3a4b5c6d"
}
```

**500 Internal Server Error**

```json
{
  "status": 500,
  "detail": "An internal error occurred",
  "trace_id": "c5b0d2e3-6f7a-8b9c-0d1e-2f3a4b5c6d7e"
}
```

Note: internal details (database errors, stack traces, credentials) are **never** included in 500 responses.

## Quick Start (Axum)

Add to your `Cargo.toml`:

```toml
[dependencies]
problem-details-axum = "0.1"
```

Define a handler that returns `Result<T, ApiError>`:

```rust,no_run
use axum::{routing::get, Router};
use problem_details::Problem;
use problem_details_axum::ApiError;

async fn get_user() -> Result<String, ApiError> {
    // Return a structured 404 error
    Err(ApiError::from(
        Problem::not_found()
            .with_title("User not found")
            .with_detail("No user with ID 42")
            .with_code("USER_NOT_FOUND"),
    ))
}

#[tokio::main]
async fn main() {
    let app: Router = Router::new().route("/users/:id", get(get_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

The response will have status `404`, content type `application/problem+json`, and a structured JSON body.

## Creating Problems

Use status constructors and the builder API:

```rust
use problem_details::Problem;

// 404 with all fields
let problem = Problem::not_found()
    .with_type("https://api.example.com/problems/user-not-found")
    .with_title("User not found")
    .with_detail("No user with ID 42 exists")
    .with_instance("/users/42")
    .with_code("USER_NOT_FOUND")
    .with_trace_id("abc-123");

// Available constructors
let _ = Problem::bad_request();         // 400
let _ = Problem::unauthorized();        // 401
let _ = Problem::forbidden();           // 403
let _ = Problem::not_found();           // 404
let _ = Problem::conflict();            // 409
let _ = Problem::unprocessable_entity();// 422
let _ = Problem::too_many_requests();   // 429
let _ = Problem::internal_server_error(); // 500
let _ = Problem::with_status(418);      // Any status
```

## Mapping Domain Errors

Implement the `IntoProblem` trait on your application's error types:

```rust
use problem_details::{IntoProblem, Problem};
use problem_details_axum::ApiError;

enum AppError {
    UserNotFound(u64),
    EmailTaken(String),
}

impl IntoProblem for AppError {
    fn to_problem(&self) -> Problem {
        match self {
            AppError::UserNotFound(id) => Problem::not_found()
                .with_title("User not found")
                .with_detail(format!("No user with ID {id}"))
                .with_code("USER_NOT_FOUND"),
            AppError::EmailTaken(email) => Problem::conflict()
                .with_title("Email already registered")
                .with_detail(format!("{email} is already in use"))
                .with_code("EMAIL_TAKEN"),
        }
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError::from_domain(&err)
    }
}
```

Then use it in handlers:

```rust,ignore
async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, ApiError> {
    let user = find_user(id).ok_or(AppError::UserNotFound(id))?;
    Ok(Json(user))
}
```

## Validation Errors

Validation failures use the `errors` extension key with a structured array of field-level errors. This format is designed for frontend form binding:

```rust
use problem_details::{Problem, ValidationError};

let problem = Problem::unprocessable_entity()
    .with_title("Validation Failed")
    .with_code("VALIDATION_ERROR")
    .with_errors(vec![
        ValidationError::new("email", "must be a valid email address")
            .with_code("INVALID_EMAIL"),
        ValidationError::new("password", "must be at least 8 characters")
            .with_code("TOO_SHORT"),
        ValidationError::new("name", "is required")
            .with_code("REQUIRED"),
    ]);
```

Each error includes:

| Field | Type | Description |
|---|---|---|
| `field` | `String` | The field path (e.g. `"email"`, `"address.zip"`) |
| `message` | `String` | Human-readable description |
| `code` | `Option<String>` | Machine-readable code for client logic |

Frontends can map `field` directly to form inputs and use `code` for i18n or conditional UI logic.

## Security

500 errors are the most dangerous for information leakage. Database connection strings, stack traces, and internal identifiers can end up in API responses if error handling is careless.

This crate prevents that by design:

- `Problem::internal_server_error()` defaults to a generic public message: `"An internal error occurred"`.
- `with_internal_cause()` stores diagnostic details in a field that is **marked `#[serde(skip)]`** and never appears in JSON output.
- `ApiError::internal()` wraps any error into a safe 500 response automatically.

```rust
use problem_details::Problem;

let problem = Problem::internal_server_error()
    .with_internal_cause("connection to db.prod:5432 refused — auth failed");

// Safe for clients
let json = serde_json::to_string(&problem).unwrap();
assert!(!json.contains("db.prod"));
assert!(!json.contains("auth failed"));

// Available for server-side logging
assert!(problem.internal_cause().is_some());
```

## Integration

### Axum

`problem-details-axum` provides:

- `IntoResponse` for `Problem` (sets status code + `application/problem+json` content type)
- `ApiError` wrapper for use in `Result<T, ApiError>` handler return types
- `with_trace()` helper for attaching trace IDs
- Optional `tracing` feature for span-based trace ID extraction

### Future

The following integrations are planned or under consideration:

- Actix Web
- OpenAPI / `utoipa` schema generation
- `validator` crate integration for automatic `ValidationError` mapping

## Features

| Feature | Crate | Description |
|---|---|---|
| `axum` | `problem-details` | Enables `IntoResponse` impl for `Problem` (auto-enabled by `problem-details-axum`) |
| `tracing` | `problem-details-axum` | Best-effort trace ID extraction from current `tracing` span |

## Design Philosophy

- **Standards first.** Follow RFC 7807 faithfully. Extensions are additive, not breaking.
- **Minimal dependencies.** The core crate depends only on `serde` and `serde_json`. Framework integrations are separate crates.
- **Explicit over magic.** No global error registries. No derive macros that hide behavior. You implement `IntoProblem`, you see exactly what maps to what.
- **Secure by default.** 500 errors produce generic messages. Internal causes must be opted into and are never serialized.
- **Predictable output.** `None` fields are omitted. Empty extensions produce no extra keys. What you build is what gets serialized.

## Roadmap

- [ ] Actix Web integration crate
- [ ] `#[derive(IntoProblem)]` macro for enum error types
- [ ] `utoipa` / OpenAPI schema generation for `Problem`
- [ ] `validator` crate bridge for automatic field error extraction
- [ ] Common error converters (sqlx, reqwest, etc.)

## MSRV

Minimum supported Rust version: **1.75**.

## License

Licensed under the [MIT License](LICENSE).
