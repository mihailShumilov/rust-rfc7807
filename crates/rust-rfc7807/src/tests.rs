use crate::{IntoProblem, Problem, ValidationItem, APPLICATION_PROBLEM_JSON};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn content_type_constant() {
    assert_eq!(APPLICATION_PROBLEM_JSON, "application/problem+json");
}

// ---------------------------------------------------------------------------
// Serialization: None fields omitted
// ---------------------------------------------------------------------------

#[test]
fn none_fields_omitted_in_json() {
    let problem = Problem::new(404);
    let json = serde_json::to_value(&problem).unwrap();
    let obj = json.as_object().unwrap();

    // status + title (auto-set per RFC 7807 §4.2 about:blank behavior)
    assert_eq!(obj.len(), 2);
    assert_eq!(json["status"], 404);
    assert_eq!(json["title"], "Not Found");
    assert!(obj.get("type").is_none());
    assert!(obj.get("detail").is_none());
    assert!(obj.get("instance").is_none());
}

// ---------------------------------------------------------------------------
// Full serialization
// ---------------------------------------------------------------------------

#[test]
fn full_problem_serialization() {
    let problem = Problem::not_found()
        .type_("https://example.com/not-found")
        .title("Not Found")
        .detail("Resource 42 does not exist")
        .instance("/resources/42");

    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["type"], "https://example.com/not-found");
    assert_eq!(json["title"], "Not Found");
    assert_eq!(json["status"], 404);
    assert_eq!(json["detail"], "Resource 42 does not exist");
    assert_eq!(json["instance"], "/resources/42");
}

// ---------------------------------------------------------------------------
// Extensions flattened at top-level
// ---------------------------------------------------------------------------

#[test]
fn extensions_flattened_at_top_level() {
    let problem = Problem::bad_request()
        .extension("custom_key", "custom_value")
        .extension("count", 42);

    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["custom_key"], "custom_value");
    assert_eq!(json["count"], 42);
}

// ---------------------------------------------------------------------------
// code(), trace_id(), request_id() set correct keys
// ---------------------------------------------------------------------------

#[test]
fn code_sets_extension_key() {
    let problem = Problem::bad_request().code("INVALID_INPUT");
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["code"], "INVALID_INPUT");
    assert_eq!(problem.get_code(), Some("INVALID_INPUT"));
}

#[test]
fn trace_id_sets_extension_key() {
    let problem = Problem::bad_request().trace_id("abc-123");
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["trace_id"], "abc-123");
    assert_eq!(problem.get_trace_id(), Some("abc-123"));
}

#[test]
fn request_id_sets_extension_key() {
    let problem = Problem::bad_request().request_id("req-456");
    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["request_id"], "req-456");
}

// ---------------------------------------------------------------------------
// Validation errors
// ---------------------------------------------------------------------------

#[test]
fn validation_errors_serialize_to_errors_array() {
    let problem = Problem::validation()
        .push_error_code("email", "must be a valid email", "INVALID_EMAIL")
        .push_error("name", "is required");

    let json = serde_json::to_value(&problem).unwrap();
    assert_eq!(json["status"], 422);
    assert_eq!(json["type"], "validation_error");
    assert_eq!(json["title"], "Validation failed");

    let errors = json["errors"]
        .as_array()
        .expect("errors should be an array");
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0]["field"], "email");
    assert_eq!(errors[0]["message"], "must be a valid email");
    assert_eq!(errors[0]["code"], "INVALID_EMAIL");
    assert_eq!(errors[1]["field"], "name");
    assert_eq!(errors[1]["message"], "is required");
    assert!(errors[1].get("code").is_none());
}

#[test]
fn errors_method_replaces_array() {
    let problem = Problem::validation().errors(vec![
        ValidationItem::new("a", "msg_a").code("CODE_A"),
        ValidationItem::new("b", "msg_b"),
    ]);

    let json = serde_json::to_value(&problem).unwrap();
    let errors = json["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0]["code"], "CODE_A");
}

// ---------------------------------------------------------------------------
// Internal cause NOT serialized
// ---------------------------------------------------------------------------

#[test]
fn internal_cause_not_serialized() {
    let problem =
        Problem::internal_server_error().with_cause(std::io::Error::other("db password=hunter2"));

    let json = serde_json::to_string(&problem).unwrap();
    assert!(!json.contains("hunter2"), "internal cause must not leak");
    assert!(
        !json.contains("db password"),
        "internal cause must not leak"
    );
    assert!(
        !json.contains("internal_cause"),
        "field name must not appear"
    );
    assert!(
        !json.contains("cause"),
        "field name 'cause' must not appear in JSON keys"
    );

    // But it IS accessible programmatically
    let cause = problem.internal_cause().unwrap();
    assert!(cause.to_string().contains("hunter2"));
}

#[test]
fn with_cause_str_not_serialized() {
    let problem = Problem::internal_server_error().with_cause_str("secret database info");

    let json = serde_json::to_string(&problem).unwrap();
    assert!(!json.contains("secret database info"));

    let cause = problem.internal_cause().unwrap();
    assert_eq!(cause.to_string(), "secret database info");
}

// ---------------------------------------------------------------------------
// internal_server_error uses generic defaults
// ---------------------------------------------------------------------------

#[test]
fn internal_server_error_generic_defaults() {
    let problem = Problem::internal_server_error();
    assert_eq!(problem.title.as_deref(), Some("Internal Server Error"));
    assert_eq!(
        problem.detail.as_deref(),
        Some("An unexpected error occurred.")
    );
    assert_eq!(problem.status, Some(500));
}

#[test]
fn internal_server_error_detail_can_be_overridden() {
    let problem = Problem::internal_server_error().detail("Custom safe message");
    assert_eq!(problem.detail.as_deref(), Some("Custom safe message"));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[test]
fn status_code_defaults_to_500() {
    let problem = Problem::default();
    assert_eq!(problem.status_code(), 500);
}

#[test]
fn is_server_error() {
    assert!(Problem::internal_server_error().is_server_error());
    assert!(Problem::new(503).is_server_error());
    assert!(!Problem::not_found().is_server_error());
    assert!(!Problem::bad_request().is_server_error());
}

#[test]
fn display_implementation() {
    let problem = Problem::not_found().title("Not Found").detail("User 42");
    assert_eq!(format!("{problem}"), "Not Found (404): User 42");
}

#[test]
fn to_json_string_pretty_works() {
    let problem = Problem::not_found().title("Not Found");
    let pretty = problem.to_json_string_pretty();
    assert!(pretty.contains("\"status\": 404"));
    assert!(pretty.contains("\"title\": \"Not Found\""));
}

// ---------------------------------------------------------------------------
// RFC 7807 §4.2: about:blank default and auto-title
// ---------------------------------------------------------------------------

#[test]
fn get_type_returns_about_blank_when_unset() {
    let problem = Problem::new(404);
    assert_eq!(problem.get_type(), "about:blank");
}

#[test]
fn get_type_returns_explicit_type_when_set() {
    let problem = Problem::new(404).type_("https://example.com/not-found");
    assert_eq!(problem.get_type(), "https://example.com/not-found");
}

#[test]
fn new_auto_sets_title_from_status_phrase() {
    assert_eq!(Problem::new(400).title.as_deref(), Some("Bad Request"));
    assert_eq!(Problem::new(404).title.as_deref(), Some("Not Found"));
    assert_eq!(
        Problem::new(429).title.as_deref(),
        Some("Too Many Requests")
    );
    assert_eq!(
        Problem::new(500).title.as_deref(),
        Some("Internal Server Error")
    );
}

#[test]
fn new_with_unknown_status_has_no_auto_title() {
    let problem = Problem::new(599);
    assert!(problem.title.is_none());
}

#[test]
fn explicit_title_overrides_auto_title() {
    let problem = Problem::new(404).title("Custom Title");
    assert_eq!(problem.title.as_deref(), Some("Custom Title"));
}

#[test]
fn about_blank_constant() {
    assert_eq!(Problem::ABOUT_BLANK, "about:blank");
}

// ---------------------------------------------------------------------------
// Deserialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn deserialization_roundtrip() {
    let original = Problem::not_found().title("Not Found").code("NOT_FOUND");

    let json = serde_json::to_string(&original).unwrap();
    let restored: Problem = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.status, Some(404));
    assert_eq!(restored.title.as_deref(), Some("Not Found"));
    assert_eq!(restored.get_code(), Some("NOT_FOUND"));
}

// ---------------------------------------------------------------------------
// IntoProblem trait
// ---------------------------------------------------------------------------

#[test]
fn into_problem_identity() {
    let problem = Problem::not_found().title("Test");
    let converted = problem.into_problem();
    assert_eq!(converted.status, Some(404));
    assert_eq!(converted.title.as_deref(), Some("Test"));
}

#[test]
fn into_problem_custom_type() {
    enum TestError {
        NotFound,
    }

    impl IntoProblem for TestError {
        fn into_problem(self) -> Problem {
            match self {
                TestError::NotFound => Problem::not_found().code("NOT_FOUND"),
            }
        }
    }

    let problem = TestError::NotFound.into_problem();
    assert_eq!(problem.status, Some(404));
    assert_eq!(problem.get_code(), Some("NOT_FOUND"));
}

// ---------------------------------------------------------------------------
// Error trait impl
// ---------------------------------------------------------------------------

#[test]
fn error_trait_source_returns_cause() {
    use std::error::Error;

    let problem = Problem::internal_server_error().with_cause(std::io::Error::other("inner error"));

    let source = problem.source().unwrap();
    assert_eq!(source.to_string(), "inner error");
}

#[test]
fn error_trait_source_returns_none_without_cause() {
    use std::error::Error;

    let problem = Problem::not_found();
    assert!(problem.source().is_none());
}

// ---------------------------------------------------------------------------
// Empty extensions produce no extra keys
// ---------------------------------------------------------------------------

#[test]
fn empty_extensions_not_in_json() {
    let problem = Problem::not_found();
    let json = serde_json::to_string(&problem).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = parsed.as_object().unwrap();
    // status + title (auto-set), no extensions
    assert_eq!(obj.len(), 2);
    assert!(obj.contains_key("status"));
    assert!(obj.contains_key("title"));
}
