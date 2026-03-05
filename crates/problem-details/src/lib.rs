//! # rust-rfc7807
//!
//! [RFC 7807](https://www.rfc-editor.org/rfc/rfc7807) Problem Details for HTTP APIs.
//!
//! This crate provides a lightweight, framework-agnostic [`Problem`] type that serializes
//! to `application/problem+json` with safe defaults, an ergonomic builder API, and
//! first-class support for validation errors, error codes, and trace correlation.
//!
//! # Quick Start
//!
//! ```
//! use rust_rfc7807::Problem;
//!
//! let problem = Problem::not_found()
//!     .title("User not found")
//!     .detail("No user with ID 42 exists")
//!     .code("USER_NOT_FOUND");
//!
//! let json = serde_json::to_value(&problem).unwrap();
//! assert_eq!(json["status"], 404);
//! assert_eq!(json["code"], "USER_NOT_FOUND");
//! ```
//!
//! # Security
//!
//! For 5xx errors, [`Problem`] defaults to a generic public message to prevent
//! leaking internal information. Use [`Problem::with_cause`] to attach a diagnostic
//! error that is **never serialized**.
//!
//! ```
//! use rust_rfc7807::Problem;
//!
//! let problem = Problem::internal_server_error()
//!     .with_cause(std::io::Error::new(std::io::ErrorKind::Other, "db timeout"));
//!
//! let json = serde_json::to_string(&problem).unwrap();
//! assert!(!json.contains("db timeout")); // never leaked
//! ```
//!
//! # Validation Errors
//!
//! ```
//! use rust_rfc7807::Problem;
//!
//! let problem = Problem::validation()
//!     .push_error("email", "must be a valid email address")
//!     .push_error_code("name", "is required", "REQUIRED");
//!
//! let json = serde_json::to_value(&problem).unwrap();
//! assert_eq!(json["status"], 422);
//! assert!(json["errors"].is_array());
//! ```

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
