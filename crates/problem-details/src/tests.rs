use crate::{IntoProblem, Problem, ValidationError, APPLICATION_PROBLEM_JSON};

#[test]
fn content_type_constant() {
    assert_eq!(APPLICATION_PROBLEM_JSON, "application/problem+json");
}

#[test]
fn serialization_omits_none_fields() {
    let problem = Problem::with_status(404);
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["status"], 404);
    assert!(json.get("type").is_none());
    assert!(json.get("title").is_none());
    assert!(json.get("detail").is_none());
    assert!(json.get("instance").is_none());
}

#[test]
fn full_problem_serialization() {
    let problem = Problem::not_found()
        .with_type("https://example.com/not-found")
        .with_title("Not Found")
        .with_detail("Resource 42 does not exist")
        .with_instance("/resources/42");

    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["type"], "https://example.com/not-found");
    assert_eq!(json["title"], "Not Found");
    assert_eq!(json["status"], 404);
    assert_eq!(json["detail"], "Resource 42 does not exist");
    assert_eq!(json["instance"], "/resources/42");
}

#[test]
fn code_is_placed_in_extensions() {
    let problem = Problem::bad_request().with_code("INVALID_INPUT");
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["code"], "INVALID_INPUT");
}

#[test]
fn validation_errors_format() {
    let errors = vec![
        ValidationError::new("email", "must be a valid email").with_code("INVALID_EMAIL"),
        ValidationError::new("name", "is required"),
    ];

    let problem = Problem::unprocessable_entity()
        .with_title("Validation Failed")
        .with_errors(errors);

    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["status"], 422);

    let errors = json["errors"]
        .as_array()
        .expect("errors should be an array");
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0]["field"], "email");
    assert_eq!(errors[0]["message"], "must be a valid email");
    assert_eq!(errors[0]["code"], "INVALID_EMAIL");
    assert_eq!(errors[1]["field"], "name");
    assert!(errors[1].get("code").is_none());
}

#[test]
fn extensions_are_included() {
    let problem = Problem::bad_request()
        .with_extension("custom_key", serde_json::json!("custom_value"))
        .with_extension("count", serde_json::json!(42));

    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["custom_key"], "custom_value");
    assert_eq!(json["count"], 42);
}

#[test]
fn internal_cause_not_serialized() {
    let problem = Problem::internal_server_error()
        .with_internal_cause("database connection failed: timeout after 30s");

    let json = serde_json::to_string(&problem).unwrap();
    assert!(!json.contains("database"));
    assert!(!json.contains("timeout"));
    assert!(!json.contains("internal_cause"));

    // But it is accessible programmatically
    assert_eq!(
        problem.internal_cause(),
        Some("database connection failed: timeout after 30s")
    );
}

#[test]
fn internal_server_error_has_generic_detail() {
    let problem = Problem::internal_server_error();
    assert_eq!(
        problem.detail.as_deref(),
        Some("An internal error occurred")
    );
}

#[test]
fn trace_id_extension() {
    let problem = Problem::bad_request().with_trace_id("abc-123");
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["trace_id"], "abc-123");
    assert_eq!(problem.trace_id(), Some("abc-123"));
}

#[test]
fn request_id_extension() {
    let problem = Problem::bad_request().with_request_id("req-456");
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["request_id"], "req-456");
}

#[test]
fn deserialization_roundtrip() {
    let original = Problem::not_found()
        .with_title("Not Found")
        .with_code("NOT_FOUND");

    let json = serde_json::to_string(&original).unwrap();
    let restored: Problem = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.status, Some(404));
    assert_eq!(restored.title.as_deref(), Some("Not Found"));
    assert_eq!(restored.code(), Some("NOT_FOUND"));
}

#[test]
fn status_or_default_returns_500() {
    let problem = Problem::new();
    assert_eq!(problem.status_or_default(), 500);
}

#[test]
fn display_implementation() {
    let problem = Problem::not_found()
        .with_title("Not Found")
        .with_detail("User 42");
    assert_eq!(format!("{problem}"), "Not Found (404): User 42");
}

#[test]
fn to_problem_trait() {
    enum TestError {
        NotFound,
    }

    impl IntoProblem for TestError {
        fn to_problem(&self) -> Problem {
            match self {
                TestError::NotFound => Problem::not_found()
                    .with_title("Not Found")
                    .with_code("NOT_FOUND"),
            }
        }
    }

    let problem = TestError::NotFound.to_problem();
    assert_eq!(problem.status, Some(404));
    assert_eq!(problem.code(), Some("NOT_FOUND"));
}

#[test]
fn empty_extensions_not_in_json() {
    let problem = Problem::not_found();
    let json = serde_json::to_string(&problem).unwrap();
    // Should only have "status" key
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = parsed.as_object().unwrap();
    assert_eq!(obj.len(), 1);
    assert!(obj.contains_key("status"));
}
