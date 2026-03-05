//! # rust-rfc7807
//!
//! [RFC 7807](https://www.rfc-editor.org/rfc/rfc7807) Problem Details for HTTP APIs.
//!
//! This crate provides a lightweight, framework-agnostic [`Problem`] type that serializes
//! to `application/problem+json` with safe defaults, an ergonomic builder API, and
//! first-class support for validation errors, error codes, and trace correlation.
//!
//! Core dependencies: `serde` and `serde_json` only.
//!
//! # Creating a Problem
//!
//! ```
//! use rust_rfc7807::Problem;
//!
//! let problem = Problem::not_found()
//!     .type_("https://api.example.com/problems/user-not-found")
//!     .title("User not found")
//!     .detail("No user with ID 42 exists")
//!     .instance("/users/42")
//!     .code("USER_NOT_FOUND")
//!     .trace_id("abc-123");
//!
//! let json = serde_json::to_value(&problem).unwrap();
//! assert_eq!(json["status"], 404);
//! assert_eq!(json["type"], "https://api.example.com/problems/user-not-found");
//! assert_eq!(json["code"], "USER_NOT_FOUND");
//! assert_eq!(json["trace_id"], "abc-123");
//! ```
//!
//! # Validation Errors
//!
//! [`Problem::validation()`] returns a 422 with `type` set to `"validation_error"`.
//! Use [`Problem::push_error`] and [`Problem::push_error_code`] to add field-level errors:
//!
//! ```
//! use rust_rfc7807::Problem;
//!
//! let problem = Problem::validation()
//!     .push_error_code("email", "must be a valid email address", "INVALID_EMAIL")
//!     .push_error("name", "is required")
//!     .code("VALIDATION_ERROR");
//!
//! let json = serde_json::to_value(&problem).unwrap();
//! assert_eq!(json["status"], 422);
//! assert_eq!(json["type"], "validation_error");
//!
//! let errors = json["errors"].as_array().unwrap();
//! assert_eq!(errors[0]["field"], "email");
//! assert_eq!(errors[0]["code"], "INVALID_EMAIL");
//! assert_eq!(errors[1]["field"], "name");
//! ```
//!
//! # Security: Internal Causes Never Serialize
//!
//! For 5xx errors, [`Problem`] defaults to a generic public message. Use
//! [`Problem::with_cause`] to attach a diagnostic error for server-side logging
//! that is **never** included in JSON output:
//!
//! ```
//! use rust_rfc7807::Problem;
//!
//! let problem = Problem::internal_server_error()
//!     .with_cause(std::io::Error::other("connection to db:5432 refused"));
//!
//! // Safe for clients — no internal details
//! let json = serde_json::to_string(&problem).unwrap();
//! assert!(!json.contains("db:5432"));
//! assert!(json.contains("An unexpected error occurred."));
//!
//! // Available for server-side logging
//! let cause = problem.internal_cause().unwrap();
//! assert!(cause.to_string().contains("db:5432"));
//! ```
//!
//! # Mapping Domain Errors
//!
//! Implement [`IntoProblem`] on your application's error types:
//!
//! ```
//! use rust_rfc7807::{IntoProblem, Problem};
//!
//! enum AppError {
//!     UserNotFound(u64),
//! }
//!
//! impl IntoProblem for AppError {
//!     fn into_problem(self) -> Problem {
//!         match self {
//!             AppError::UserNotFound(id) => Problem::not_found()
//!                 .detail(format!("No user with ID {id}"))
//!                 .code("USER_NOT_FOUND"),
//!         }
//!     }
//! }
//!
//! let problem = AppError::UserNotFound(42).into_problem();
//! assert_eq!(problem.status, Some(404));
//! assert_eq!(problem.get_code(), Some("USER_NOT_FOUND"));
//! ```
//!
//! # Extension Fields
//!
//! Arbitrary extension fields are flattened into the top-level JSON object:
//!
//! ```
//! use rust_rfc7807::Problem;
//!
//! let problem = Problem::new(429)
//!     .code("RATE_LIMITED")
//!     .extension("retry_after", 30);
//!
//! let json = serde_json::to_value(&problem).unwrap();
//! assert_eq!(json["retry_after"], 30);
//! assert_eq!(json["code"], "RATE_LIMITED");
//! ```
//!
//! # Axum Integration
//!
//! Enable the `axum` feature for [`IntoResponse`](axum_core::response::IntoResponse)
//! on [`Problem`], or use the companion crate
//! [`rust-rfc7807-axum`](https://docs.rs/rust-rfc7807-axum) for the full integration
//! including [`ApiError`](https://docs.rs/rust-rfc7807-axum/latest/rust_rfc7807_axum/enum.ApiError.html).

mod problem;
mod traits;
mod validation;

#[cfg(feature = "axum")]
mod axum_impl;

pub use problem::Problem;
pub use traits::IntoProblem;
pub use validation::ValidationItem;

/// The `Content-Type` header value for RFC 7807 problem responses.
pub const APPLICATION_PROBLEM_JSON: &str = "application/problem+json";

#[cfg(test)]
mod tests;
