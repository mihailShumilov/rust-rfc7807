use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use http_body_util::BodyExt;
use problem_details::APPLICATION_PROBLEM_JSON;
use tower::ServiceExt;

fn app() -> axum::Router {
    axum_api::app()
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// ---------------------------------------------------------------------------
// /ok returns 200
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ok_returns_200() {
    let response = app()
        .oneshot(Request::get("/ok").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// /not-found returns 404 problem+json
// ---------------------------------------------------------------------------

#[tokio::test]
async fn not_found_returns_404_problem() {
    let response = app()
        .oneshot(Request::get("/not-found").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response.headers().get(http::header::CONTENT_TYPE).unwrap(),
        APPLICATION_PROBLEM_JSON
    );

    let json = response_json(response).await;
    assert_eq!(json["status"], 404);
    assert_eq!(json["code"], "RESOURCE_NOT_FOUND");
    assert!(json["detail"].as_str().unwrap().contains("User 42"));
}

// ---------------------------------------------------------------------------
// /validate returns 422 with errors array
// ---------------------------------------------------------------------------

#[tokio::test]
async fn validate_returns_422_with_errors() {
    let response = app()
        .oneshot(
            Request::post("/validate")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.headers().get(http::header::CONTENT_TYPE).unwrap(),
        APPLICATION_PROBLEM_JSON
    );

    let json = response_json(response).await;
    assert_eq!(json["status"], 422);
    assert_eq!(json["code"], "VALIDATION_ERROR");
    assert_eq!(json["type"], "validation_error");

    let errors = json["errors"].as_array().expect("errors should be array");
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0]["field"], "email");
    assert_eq!(errors[1]["field"], "name");
}

// ---------------------------------------------------------------------------
// /internal returns 500 and does NOT leak internal error message
// ---------------------------------------------------------------------------

#[tokio::test]
async fn internal_returns_safe_500_without_leaking() {
    let response = app()
        .oneshot(Request::get("/internal").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response.headers().get(http::header::CONTENT_TYPE).unwrap(),
        APPLICATION_PROBLEM_JSON
    );

    let json = response_json(response).await;
    assert_eq!(json["status"], 500);

    // Must NOT leak internal details
    let body = serde_json::to_string(&json).unwrap();
    assert!(!body.contains("hunter2"));
    assert!(!body.contains("password"));
    assert!(!body.contains("5432"));
    assert!(!body.contains("db.internal"));
}
