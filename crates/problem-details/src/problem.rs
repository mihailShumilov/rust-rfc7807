use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::ValidationError;

/// An RFC 7807 Problem Details object.
///
/// Represents a structured error response per [RFC 7807](https://www.rfc-editor.org/rfc/rfc7807).
/// All standard fields are optional and omitted from JSON when `None`.
///
/// # Extensions
///
/// Arbitrary extension fields are supported via a `BTreeMap<String, Value>`.
/// Common extensions (`code`, `errors`, `trace_id`, `request_id`) have
/// dedicated builder methods.
///
/// # Internal Cause
///
/// Use [`Problem::with_internal_cause`] to attach a diagnostic message that is
/// **never serialized** to JSON. This is essential for 5xx errors where you want
/// to log the root cause without exposing it to clients.
///
/// # Example
///
/// ```
/// use problem_details::Problem;
///
/// let problem = Problem::bad_request()
///     .with_title("Invalid input")
///     .with_detail("The 'email' field is required");
///
/// let json = serde_json::to_value(&problem).unwrap();
/// assert_eq!(json["status"], 400);
/// assert_eq!(json["title"], "Invalid input");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    /// A URI reference that identifies the problem type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub problem_type: Option<String>,

    /// A short, human-readable summary of the problem type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The HTTP status code for this occurrence of the problem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,

    /// A human-readable explanation specific to this occurrence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// A URI reference that identifies this specific occurrence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    /// Extension fields beyond the RFC 7807 standard fields.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,

    /// Internal cause message for diagnostics. Never serialized.
    #[serde(skip)]
    internal_cause: Option<String>,
}

impl Problem {
    /// Create a new empty problem.
    pub fn new() -> Self {
        Self {
            problem_type: None,
            title: None,
            status: None,
            detail: None,
            instance: None,
            extensions: BTreeMap::new(),
            internal_cause: None,
        }
    }

    /// Create a problem with the given HTTP status code.
    pub fn with_status(status: u16) -> Self {
        Self {
            status: Some(status),
            ..Self::new()
        }
    }

    // --- Common status constructors ---

    /// 400 Bad Request.
    pub fn bad_request() -> Self {
        Self::with_status(400)
    }

    /// 401 Unauthorized.
    pub fn unauthorized() -> Self {
        Self::with_status(401)
    }

    /// 403 Forbidden.
    pub fn forbidden() -> Self {
        Self::with_status(403)
    }

    /// 404 Not Found.
    pub fn not_found() -> Self {
        Self::with_status(404)
    }

    /// 409 Conflict.
    pub fn conflict() -> Self {
        Self::with_status(409)
    }

    /// 422 Unprocessable Entity.
    pub fn unprocessable_entity() -> Self {
        Self::with_status(422)
    }

    /// 429 Too Many Requests.
    pub fn too_many_requests() -> Self {
        Self::with_status(429)
    }

    /// 500 Internal Server Error.
    ///
    /// The detail is set to a generic message by default to prevent leaking
    /// internal information. Use [`Self::with_internal_cause`] to store
    /// diagnostic details.
    pub fn internal_server_error() -> Self {
        Self::with_status(500).with_detail("An internal error occurred")
    }

    // --- Builder methods ---

    /// Set the problem type URI.
    pub fn with_type(mut self, problem_type: impl Into<String>) -> Self {
        self.problem_type = Some(problem_type.into());
        self
    }

    /// Set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the public detail message.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Set the instance URI.
    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    /// Set the `"code"` extension field (a stable string code for clients).
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.extensions
            .insert("code".into(), serde_json::Value::String(code.into()));
        self
    }

    /// Set the `"errors"` extension field with a list of validation errors.
    pub fn with_errors(mut self, errors: Vec<ValidationError>) -> Self {
        self.extensions.insert(
            "errors".into(),
            serde_json::to_value(errors).expect("ValidationError is always serializable"),
        );
        self
    }

    /// Set the `"trace_id"` extension field.
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.extensions.insert(
            "trace_id".into(),
            serde_json::Value::String(trace_id.into()),
        );
        self
    }

    /// Set the `"request_id"` extension field.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.extensions.insert(
            "request_id".into(),
            serde_json::Value::String(request_id.into()),
        );
        self
    }

    /// Add an arbitrary extension field.
    pub fn with_extension(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extensions.insert(key.into(), value);
        self
    }

    /// Store an internal cause message for diagnostics.
    ///
    /// This value is **never serialized** to JSON. It is intended for server-side
    /// logging only.
    pub fn with_internal_cause(mut self, cause: impl Into<String>) -> Self {
        self.internal_cause = Some(cause.into());
        self
    }

    // --- Accessors ---

    /// Returns the HTTP status code, defaulting to 500 if not set.
    pub fn status_or_default(&self) -> u16 {
        self.status.unwrap_or(500)
    }

    /// Returns the `"code"` extension value, if set.
    pub fn code(&self) -> Option<&str> {
        self.extensions.get("code").and_then(|v| v.as_str())
    }

    /// Returns the `"trace_id"` extension value, if set.
    pub fn trace_id(&self) -> Option<&str> {
        self.extensions.get("trace_id").and_then(|v| v.as_str())
    }

    /// Returns the internal cause, if set.
    pub fn internal_cause(&self) -> Option<&str> {
        self.internal_cause.as_deref()
    }
}

impl Default for Problem {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(title) = &self.title {
            write!(f, "{title}")?;
        } else {
            write!(f, "Problem")?;
        }
        if let Some(status) = self.status {
            write!(f, " ({status})")?;
        }
        if let Some(detail) = &self.detail {
            write!(f, ": {detail}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Problem {}
