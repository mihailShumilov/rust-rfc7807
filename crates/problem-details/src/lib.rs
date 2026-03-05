//! # problem-details
//!
//! Implementation of [RFC 7807 Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc7807).
//!
//! This crate provides a framework-agnostic [`Problem`] type that serializes to
//! `application/problem+json` responses, along with pragmatic extensions for
//! validation errors, error codes, and trace correlation.
//!
//! # Quick Start
//!
//! ```
//! use problem_details::Problem;
//!
//! let problem = Problem::not_found()
//!     .with_title("User not found")
//!     .with_detail("No user with ID 42 exists")
//!     .with_code("USER_NOT_FOUND");
//!
//! let json = serde_json::to_value(&problem).unwrap();
//! assert_eq!(json["status"], 404);
//! ```
//!
//! # Security
//!
//! For 5xx errors, [`Problem`] defaults the public detail to a generic message
//! to prevent leaking internal information. Use [`Problem::with_internal_cause`]
//! to store diagnostic details that are never serialized.
//!
//! # Extensions
//!
//! RFC 7807 allows arbitrary extension fields. This crate provides first-class
//! support for common extensions:
//!
//! - **`code`**: A stable error code string for client consumption.
//! - **`errors`**: A list of field-level validation errors.
//! - **`trace_id`** / **`request_id`**: Correlation identifiers.
//!
//! You can also add arbitrary extensions via [`Problem::with_extension`].

mod problem;
mod traits;
mod validation;

#[cfg(feature = "axum")]
mod axum_impl;

pub use problem::Problem;
pub use traits::IntoProblem;
pub use validation::ValidationError;

/// The `Content-Type` header value for RFC 7807 problem responses.
pub const APPLICATION_PROBLEM_JSON: &str = "application/problem+json";

#[cfg(test)]
mod tests;
