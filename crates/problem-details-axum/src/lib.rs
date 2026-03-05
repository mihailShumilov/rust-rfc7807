//! # problem-details-axum
//!
//! [Axum](https://docs.rs/axum) integration for
//! [RFC 7807 Problem Details](https://www.rfc-editor.org/rfc/rfc7807).
//!
//! This crate provides `IntoResponse` for [`Problem`] and an [`ApiError`]
//! wrapper that converts unknown errors into safe 500 responses.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use axum::{routing::get, Router};
//! use problem_details::Problem;
//! use problem_details_axum::ApiError;
//!
//! async fn handler() -> Result<String, ApiError> {
//!     Err(Problem::not_found().title("Not Found").into())
//! }
//!
//! let app: Router = Router::new().route("/example", get(handler));
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
pub use problem_details::{IntoProblem, Problem, ValidationItem, APPLICATION_PROBLEM_JSON};

#[cfg(test)]
mod tests;
