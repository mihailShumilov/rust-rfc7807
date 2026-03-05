use crate::Problem;

/// Convert a value into an RFC 7807 [`Problem`].
///
/// Implement this trait on your application's error types to enable
/// automatic conversion into structured problem responses.
///
/// # Example
///
/// ```
/// use rust_rfc7807::{IntoProblem, Problem};
///
/// enum AppError {
///     UserNotFound(u64),
///     Unauthorized,
/// }
///
/// impl IntoProblem for AppError {
///     fn into_problem(self) -> Problem {
///         match self {
///             AppError::UserNotFound(id) => Problem::not_found()
///                 .title("User not found")
///                 .detail(format!("No user with ID {id}"))
///                 .code("USER_NOT_FOUND"),
///             AppError::Unauthorized => Problem::unauthorized()
///                 .title("Unauthorized")
///                 .code("UNAUTHORIZED"),
///         }
///     }
/// }
/// ```
pub trait IntoProblem {
    /// Convert this value into a [`Problem`] instance.
    fn into_problem(self) -> Problem;
}

impl IntoProblem for Problem {
    fn into_problem(self) -> Problem {
        self
    }
}
