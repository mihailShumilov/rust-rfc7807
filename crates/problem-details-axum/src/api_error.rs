use axum::response::{IntoResponse, Response};
use problem_details::{IntoProblem, Problem};
use std::fmt;

/// A wrapper error type that converts into an RFC 7807 Problem response.
///
/// `ApiError` can wrap either an explicit [`Problem`] or an opaque boxed error.
/// Unknown errors are automatically converted into safe 500 responses that do
/// **not** leak internal details.
///
/// # Usage with `Result`
///
/// ```rust,no_run
/// use problem_details::Problem;
/// use problem_details_axum::ApiError;
///
/// async fn handler() -> Result<String, ApiError> {
///     // Explicit problem
///     Err(ApiError::from(Problem::not_found()))
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
pub struct ApiError {
    problem: Problem,
}

impl ApiError {
    /// Create an `ApiError` from an explicit [`Problem`].
    pub fn from_problem(problem: Problem) -> Self {
        Self { problem }
    }

    /// Create a safe 500 `ApiError` from any error, storing the original
    /// error message as an internal cause (never serialized).
    pub fn internal(err: impl fmt::Display) -> Self {
        Self {
            problem: Problem::internal_server_error().with_internal_cause(err.to_string()),
        }
    }

    /// Create an `ApiError` from a domain error that implements [`IntoProblem`].
    pub fn from_domain(err: &impl IntoProblem) -> Self {
        Self {
            problem: err.to_problem(),
        }
    }

    /// Returns a reference to the underlying [`Problem`].
    pub fn problem(&self) -> &Problem {
        &self.problem
    }
}

impl From<Problem> for ApiError {
    fn from(problem: Problem) -> Self {
        Self::from_problem(problem)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        self.problem.into_response()
    }
}

impl fmt::Debug for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiError")
            .field("problem", &self.problem)
            .finish()
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.problem)
    }
}
