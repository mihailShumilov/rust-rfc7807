use crate::Problem;

/// Convert a domain error into an RFC 7807 [`Problem`].
///
/// Implement this trait on your application's error types to enable
/// automatic conversion into structured problem responses.
///
/// # Example
///
/// ```
/// use problem_details::{IntoProblem, Problem};
///
/// enum AppError {
///     UserNotFound(u64),
///     Unauthorized,
/// }
///
/// impl IntoProblem for AppError {
///     fn to_problem(&self) -> Problem {
///         match self {
///             AppError::UserNotFound(id) => Problem::not_found()
///                 .with_title("User not found")
///                 .with_detail(format!("No user with ID {id}"))
///                 .with_code("USER_NOT_FOUND"),
///             AppError::Unauthorized => Problem::unauthorized()
///                 .with_title("Unauthorized")
///                 .with_code("UNAUTHORIZED"),
///         }
///     }
/// }
/// ```
pub trait IntoProblem {
    /// Convert this error into a [`Problem`] instance.
    fn to_problem(&self) -> Problem;
}
