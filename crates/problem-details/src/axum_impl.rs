use axum_core::response::{IntoResponse, Response};
use http::header;

use crate::{Problem, APPLICATION_PROBLEM_JSON};

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = http::StatusCode::from_u16(self.status_or_default())
            .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);

        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"type":"about:blank","title":"Internal Server Error","status":500}"#.to_string()
        });

        (
            status,
            [(header::CONTENT_TYPE, APPLICATION_PROBLEM_JSON)],
            body,
        )
            .into_response()
    }
}
