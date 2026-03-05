use axum_core::response::IntoResponse;
use http::header;
use problem_details::{Problem, APPLICATION_PROBLEM_JSON};

use crate::{attach_trace, ApiError};

// ---------------------------------------------------------------------------
// IntoResponse sets Content-Type correctly
// ---------------------------------------------------------------------------

#[test]
fn into_response_sets_content_type() {
    let problem = Problem::bad_request();
    let response = problem.into_response();
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        APPLICATION_PROBLEM_JSON
    );
}

// ---------------------------------------------------------------------------
// Status is applied properly
// ---------------------------------------------------------------------------

#[test]
fn into_response_sets_status_code() {
    let problem = Problem::not_found().title("Not Found");
    let response = problem.into_response();
    assert_eq!(response.status().as_u16(), 404);
}

#[test]
fn into_response_forbidden() {
    let response = Problem::forbidden().into_response();
    assert_eq!(response.status().as_u16(), 403);
}

// ---------------------------------------------------------------------------
// Unknown errors become safe 500
// ---------------------------------------------------------------------------

#[test]
fn unknown_error_converts_to_safe_500() {
    let err = ApiError::internal(std::io::Error::other("secret database password"));
    let response = err.into_response();

    assert_eq!(response.status().as_u16(), 500);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        APPLICATION_PROBLEM_JSON
    );
}

#[test]
fn api_error_internal_variant_does_not_leak() {
    // Verify by checking Problem serialization directly since we can't
    // easily read the response body in sync tests.
    let problem = Problem::internal_server_error().with_cause_str("secret password=hunter2");
    let json = serde_json::to_string(&problem).unwrap();
    assert!(!json.contains("hunter2"));
    assert!(!json.contains("secret password"));
    assert!(json.contains("An unexpected error occurred."));
}

// ---------------------------------------------------------------------------
// trace_id attaches when provided
// ---------------------------------------------------------------------------

#[test]
fn trace_id_attaches_via_helper() {
    let problem = attach_trace(Problem::bad_request(), "trace-xyz-789");
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["trace_id"], "trace-xyz-789");
}

// ---------------------------------------------------------------------------
// ApiError from Problem
// ---------------------------------------------------------------------------

#[test]
fn api_error_from_problem() {
    let err = ApiError::from(Problem::not_found().title("Gone"));
    let response = err.into_response();
    assert_eq!(response.status().as_u16(), 404);
}

// ---------------------------------------------------------------------------
// ApiError from domain error
// ---------------------------------------------------------------------------

#[test]
fn api_error_from_domain() {
    use problem_details::IntoProblem;

    struct MyError;
    impl IntoProblem for MyError {
        fn into_problem(self) -> Problem {
            Problem::conflict().code("DUPLICATE")
        }
    }

    let err = ApiError::from_domain(MyError);
    let response = err.into_response();
    assert_eq!(response.status().as_u16(), 409);
}

// ---------------------------------------------------------------------------
// Serialization sanity
// ---------------------------------------------------------------------------

#[test]
fn problem_json_body_is_valid() {
    let problem = Problem::not_found().title("Not Found").code("NOT_FOUND");
    let json_str = serde_json::to_string(&problem).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["status"], 404);
    assert_eq!(parsed["title"], "Not Found");
    assert_eq!(parsed["code"], "NOT_FOUND");
}
