use axum_core::response::{IntoResponse, Response};
use problem_details::{IntoProblem, Problem};
use std::fmt;

/// An error type for Axum handlers that produces RFC 7807 Problem responses.
///
/// `ApiError` is an enum with two variants:
/// - [`ApiError::Problem`] wraps an explicit [`Problem`] response.
/// - [`ApiError::Internal`] wraps an opaque error, producing a safe 500 response
///   that never leaks internal details.
///
/// # Usage with `Result`
///
/// ```rust,no_run
/// use problem_details::Problem;
/// use problem_details_axum::ApiError;
///
/// async fn handler() -> Result<String, ApiError> {
///     Err(Problem::not_found().title("Not Found").into())
/// }
/// ```
///
/// # Converting unknown errors
///
/// ```rust,no_run
/// use problem_details_axum::ApiError;
///
/// async fn handler() -> Result<String, ApiError> {
///     let value: i32 = "not a number".parse().map_err(ApiError::internal)?;
///     Ok(value.to_string())
/// }
/// ```
pub enum ApiError {
    /// An explicit problem response.
    Problem(Problem),
    /// An opaque internal error that will be converted to a safe 500 response.
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl ApiError {
    /// Create an `ApiError` from an explicit [`Problem`].
    pub fn from_problem(problem: Problem) -> Self {
        Self::Problem(problem)
    }

    /// Create a safe 500 `ApiError` from any error.
    ///
    /// The original error is stored as the internal cause and never exposed
    /// in the response body.
    pub fn internal(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Internal(Box::new(err))
    }

    /// Create an `ApiError` from a domain error implementing [`IntoProblem`].
    pub fn from_domain(err: impl IntoProblem) -> Self {
        Self::Problem(err.into_problem())
    }
}

impl From<Problem> for ApiError {
    fn from(problem: Problem) -> Self {
        Self::Problem(problem)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Problem(problem) => problem.into_response(),
            ApiError::Internal(err) => Problem::internal_server_error()
                .with_cause_str(err.to_string())
                .into_response(),
        }
    }
}

impl fmt::Debug for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Problem(p) => f.debug_tuple("ApiError::Problem").field(p).finish(),
            ApiError::Internal(e) => f
                .debug_tuple("ApiError::Internal")
                .field(&e.to_string())
                .finish(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Problem(p) => write!(f, "{p}"),
            ApiError::Internal(e) => write!(f, "Internal error: {e}"),
        }
    }
}
