use axum::response::IntoResponse;
use http::header;
use problem_details::{Problem, APPLICATION_PROBLEM_JSON};

use crate::{with_trace, ApiError};

#[test]
fn into_response_sets_correct_status() {
    let problem = Problem::not_found().with_title("Not Found");
    let response = problem.into_response();
    assert_eq!(response.status().as_u16(), 404);
}

#[test]
fn into_response_sets_content_type() {
    let problem = Problem::bad_request();
    let response = problem.into_response();
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        APPLICATION_PROBLEM_JSON
    );
}

#[test]
fn into_response_body_is_valid_json() {
    let problem = Problem::not_found()
        .with_title("Not Found")
        .with_code("NOT_FOUND");
    let json_str = serde_json::to_string(&problem).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["status"], 404);
    assert_eq!(parsed["title"], "Not Found");
    assert_eq!(parsed["code"], "NOT_FOUND");
}

#[test]
fn unknown_error_converts_to_safe_500() {
    let err = ApiError::internal("secret database password leaked");
    let problem = err.problem();

    assert_eq!(problem.status, Some(500));
    assert_eq!(
        problem.detail.as_deref(),
        Some("An internal error occurred")
    );

    // The internal cause is stored but never serialized
    assert_eq!(
        problem.internal_cause(),
        Some("secret database password leaked")
    );

    let json = serde_json::to_string(problem).unwrap();
    assert!(!json.contains("secret"));
    assert!(!json.contains("password"));
    assert!(!json.contains("leaked"));
}

#[test]
fn api_error_from_problem() {
    let problem = Problem::not_found().with_title("Gone");
    let err = ApiError::from(problem);
    assert_eq!(err.problem().status, Some(404));
}

#[test]
fn trace_id_appears_when_set() {
    let problem = with_trace(Problem::bad_request(), "trace-xyz-789");
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["trace_id"], "trace-xyz-789");
}

#[test]
fn api_error_into_response_status() {
    let err = ApiError::from(Problem::forbidden());
    let response = err.into_response();
    assert_eq!(response.status().as_u16(), 403);
}
