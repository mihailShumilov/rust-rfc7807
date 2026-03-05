use axum::extract::Json;
use axum::routing::{get, post};
use axum::Router;
use problem_details::{IntoProblem, Problem, ValidationError};
use problem_details_axum::ApiError;
use serde::{Deserialize, Serialize};

// --- Domain types ---

#[derive(Serialize)]
struct OkResponse {
    message: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    email: Option<String>,
    name: Option<String>,
}

// --- Domain errors ---

enum AppError {
    NotFound { resource: String },
    InternalFailure(String),
}

impl IntoProblem for AppError {
    fn to_problem(&self) -> Problem {
        match self {
            AppError::NotFound { resource } => Problem::not_found()
                .with_title("Resource not found")
                .with_detail(format!("{resource} does not exist"))
                .with_code("RESOURCE_NOT_FOUND"),
            AppError::InternalFailure(msg) => {
                Problem::internal_server_error().with_internal_cause(msg.clone())
            }
        }
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError::from_domain(&err)
    }
}

// --- Handlers ---

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
    let mut errors = Vec::new();

    if body.email.as_ref().map_or(true, |e| e.is_empty()) {
        errors.push(ValidationError::new("email", "is required").with_code("REQUIRED"));
    }
    if body.name.as_ref().map_or(true, |n| n.is_empty()) {
        errors.push(ValidationError::new("name", "is required").with_code("REQUIRED"));
    }

    if !errors.is_empty() {
        return Err(ApiError::from(
            Problem::unprocessable_entity()
                .with_title("Validation Failed")
                .with_code("VALIDATION_ERROR")
                .with_errors(errors),
        ));
    }

    Ok(Json(OkResponse {
        message: format!("Created user {}", body.name.unwrap()),
    }))
}

async fn error_handler() -> Result<Json<()>, ApiError> {
    Err(AppError::InternalFailure(
        "connection to db.internal:5432 refused — password=hunter2".into(),
    )
    .into())
}

/// Build the application router.
pub fn app() -> Router {
    Router::new()
        .route("/ok", get(ok_handler))
        .route("/not-found", get(not_found_handler))
        .route("/validate", post(validate_handler))
        .route("/panic-or-error", get(error_handler))
}
