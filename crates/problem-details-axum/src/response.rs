use problem_details::Problem;

/// Set the `trace_id` extension on a [`Problem`].
///
/// This is a convenience function equivalent to [`Problem::with_trace_id`].
///
/// ```
/// use problem_details::Problem;
/// use problem_details_axum::with_trace;
///
/// let problem = with_trace(Problem::not_found(), "trace-abc-123");
/// assert_eq!(problem.trace_id(), Some("trace-abc-123"));
/// ```
pub fn with_trace(problem: Problem, trace_id: impl Into<String>) -> Problem {
    problem.with_trace_id(trace_id)
}
