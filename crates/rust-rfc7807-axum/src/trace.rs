use rust_rfc7807::Problem;

/// Attach a `trace_id` extension to a [`Problem`].
///
/// This is a convenience function equivalent to [`Problem::trace_id`].
///
/// ```
/// use rust_rfc7807::Problem;
/// use rust_rfc7807_axum::attach_trace;
///
/// let problem = attach_trace(Problem::not_found(), "trace-abc-123");
/// assert_eq!(problem.get_trace_id(), Some("trace-abc-123"));
/// ```
pub fn attach_trace(problem: Problem, trace_id: impl Into<String>) -> Problem {
    problem.trace_id(trace_id)
}
