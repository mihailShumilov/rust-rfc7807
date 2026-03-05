//! # rust-rfc7807-axum
//!
//! [Axum](https://docs.rs/axum) integration for
//! [RFC 7807 Problem Details](https://www.rfc-editor.org/rfc/rfc7807).
//!
//! This crate provides [`IntoResponse`](axum_core::response::IntoResponse) for
//! [`Problem`] and an [`ApiError`] wrapper that converts unknown errors into
//! safe 500 responses that never leak internal details.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use axum::{routing::get, Router};
//! use rust_rfc7807::Problem;
//! use rust_rfc7807_axum::ApiError;
//!
//! async fn handler() -> Result<String, ApiError> {
//!     Err(Problem::not_found()
//!         .detail("No user with ID 42")
//!         .code("USER_NOT_FOUND")
//!         .into())
//! }
//!
//! let app: Router = Router::new().route("/users/:id", get(handler));
//! ```
//!
//! The response will have:
//! - HTTP status `404`
//! - `Content-Type: application/problem+json`
//! - A JSON body with `status`, `title`, `detail`, and `code` fields
//!
//! # Safe 500 Handling
//!
//! Use [`ApiError::internal`] to wrap any error into a safe 500 response.
//! The original error is stored for logging but **never** serialized:
//!
//! ```
//! use rust_rfc7807_axum::ApiError;
//! use axum_core::response::IntoResponse;
//!
//! let err = ApiError::internal(std::io::Error::other("db connection refused"));
//! let response = err.into_response();
//!
//! assert_eq!(response.status().as_u16(), 500);
//! ```
//!
//! # Domain Error Conversion
//!
//! Implement [`IntoProblem`] on your domain errors, then convert via [`ApiError::from_domain`]:
//!
//! ```
//! use rust_rfc7807::{IntoProblem, Problem};
//! use rust_rfc7807_axum::ApiError;
//!
//! struct NotFound;
//!
//! impl IntoProblem for NotFound {
//!     fn into_problem(self) -> Problem {
//!         Problem::not_found().code("NOT_FOUND")
//!     }
//! }
//!
//! let err = ApiError::from_domain(NotFound);
//! ```
//!
//! # Features
//!
//! - **`tracing`**: Enables best-effort extraction of the current span's trace ID.

mod api_error;
mod trace;

pub use api_error::ApiError;
pub use trace::attach_trace;

// Re-export core types for convenience.
pub use rust_rfc7807::{IntoProblem, Problem, ValidationItem, APPLICATION_PROBLEM_JSON};

#[cfg(test)]
mod tests;
