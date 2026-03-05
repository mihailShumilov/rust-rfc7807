use axum::extract::Json;
use axum::routing::{get, post};
use axum::Router;
use problem_details::{IntoProblem, Problem};
use problem_details_axum::ApiError;
use serde::{Deserialize, Serialize};

// -- Domain types --

#[derive(Serialize)]
struct OkResponse {
    message: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    email: Option<String>,
    name: Option<String>,
}

// -- Domain errors --

enum AppError {
    NotFound { resource: String },
    InternalFailure(String),
}

impl IntoProblem for AppError {
    fn into_problem(self) -> Problem {
        match self {
            AppError::NotFound { resource } => Problem::not_found()
                .title("Resource not found")
                .detail(format!("{resource} does not exist"))
                .code("RESOURCE_NOT_FOUND"),
            AppError::InternalFailure(msg) => Problem::internal_server_error().with_cause_str(msg),
        }
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError::from_domain(err)
    }
}

// -- Handlers --

async fn ok_handler() -> Json<OkResponse> {
    Json(OkResponse {
        message: "Hello, world!".into(),
    })
}

async fn not_found_handler() -> Result<Json<()>, ApiError> {
    Err(AppError::NotFound {
        resource: "User 42".into(),
    }
    .into())
}

async fn validate_handler(
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    let mut problem = Problem::validation();
    let mut has_errors = false;

    if body.email.as_ref().map_or(true, |e| e.is_empty()) {
        problem = problem.push_error_code("email", "is required", "REQUIRED");
        has_errors = true;
    }
    if body.name.as_ref().map_or(true, |n| n.is_empty()) {
        problem = problem.push_error_code("name", "is required", "REQUIRED");
        has_errors = true;
    }

    if has_errors {
        return Err(problem.code("VALIDATION_ERROR").into());
    }

    Ok(Json(OkResponse {
        message: format!("Created user {}", body.name.unwrap()),
    }))
}

async fn internal_handler() -> Result<Json<()>, ApiError> {
    Err(AppError::InternalFailure(
        "connection to db.internal:5432 refused -- password=hunter2".into(),
    )
    .into())
}

/// Build the application router.
pub fn app() -> Router {
    Router::new()
        .route("/ok", get(ok_handler))
        .route("/not-found", get(not_found_handler))
        .route("/validate", post(validate_handler))
        .route("/internal", get(internal_handler))
}
