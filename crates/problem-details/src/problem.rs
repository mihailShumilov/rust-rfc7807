use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::fmt;

use crate::ValidationItem;

/// An RFC 7807 Problem Details object.
///
/// Represents a structured error response per
/// [RFC 7807](https://www.rfc-editor.org/rfc/rfc7807). All standard fields
/// are optional and omitted from JSON when `None`. Extension fields are
/// flattened into the top-level JSON object.
///
/// # Internal Cause
///
/// Use [`Problem::with_cause`] to attach a diagnostic error that is **never
/// serialized** to JSON. This is essential for 5xx errors where you want to
/// log the root cause server-side without exposing it to clients.
///
/// # Example
///
/// ```
/// use problem_details::Problem;
///
/// let problem = Problem::bad_request()
///     .title("Invalid input")
///     .detail("The 'email' field is required");
///
/// let json = serde_json::to_value(&problem).unwrap();
/// assert_eq!(json["status"], 400);
/// assert_eq!(json["title"], "Invalid input");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    /// A URI reference that identifies the problem type.
    /// Defaults to `"about:blank"` per RFC 7807 when absent.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_uri: Option<String>,

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
    #[serde(flatten, skip_serializing_if = "Map::is_empty")]
    pub extensions: Map<String, serde_json::Value>,

    /// Internal cause for diagnostics. Never serialized.
    #[serde(skip)]
    cause: Option<InternalCause>,
}

/// Holds an internal error cause that is never serialized.
///
/// This wrapper stores either a boxed `Error` trait object or a plain string,
/// providing server-side diagnostic information for logging without risking
/// exposure in API responses.
struct InternalCause {
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl fmt::Debug for InternalCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InternalCause({:?})", self.source.to_string())
    }
}

impl Clone for InternalCause {
    fn clone(&self) -> Self {
        // Clone by converting to string — the original typed error cannot be cloned generically.
        InternalCause {
            source: Box::new(StringError(self.source.to_string())),
        }
    }
}

/// A simple string-based error for cloning internal causes.
#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StringError {}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl Problem {
    /// Create a new problem with the given HTTP status code.
    ///
    /// ```
    /// use problem_details::Problem;
    ///
    /// let p = Problem::new(429).title("Too Many Requests");
    /// assert_eq!(p.status, Some(429));
    /// ```
    pub fn new(status: u16) -> Self {
        Self {
            type_uri: None,
            title: None,
            status: Some(status),
            detail: None,
            instance: None,
            extensions: Map::new(),
            cause: None,
        }
    }

    /// 400 Bad Request.
    pub fn bad_request() -> Self {
        Self::new(400)
    }

    /// 401 Unauthorized.
    pub fn unauthorized() -> Self {
        Self::new(401)
    }

    /// 403 Forbidden.
    pub fn forbidden() -> Self {
        Self::new(403)
    }

    /// 404 Not Found.
    pub fn not_found() -> Self {
        Self::new(404)
    }

    /// 409 Conflict.
    pub fn conflict() -> Self {
        Self::new(409)
    }

    /// 422 Unprocessable Entity with validation defaults.
    ///
    /// Sets status to 422, type to `"validation_error"`, and title to
    /// `"Validation failed"`. Add field errors with [`push_error`](Self::push_error)
    /// and [`push_error_code`](Self::push_error_code).
    ///
    /// ```
    /// use problem_details::Problem;
    ///
    /// let p = Problem::validation()
    ///     .push_error("email", "is required");
    ///
    /// let json = serde_json::to_value(&p).unwrap();
    /// assert_eq!(json["status"], 422);
    /// assert_eq!(json["type"], "validation_error");
    /// ```
    pub fn validation() -> Self {
        Self::new(422)
            .type_("validation_error")
            .title("Validation failed")
    }

    /// 422 Unprocessable Entity (without validation defaults).
    pub fn unprocessable_entity() -> Self {
        Self::new(422)
    }

    /// 429 Too Many Requests.
    pub fn too_many_requests() -> Self {
        Self::new(429)
    }

    /// 500 Internal Server Error.
    ///
    /// Returns a problem with safe generic defaults:
    /// - title: `"Internal Server Error"`
    /// - detail: `"An unexpected error occurred."`
    ///
    /// Use [`with_cause`](Self::with_cause) to attach a diagnostic error for
    /// server-side logging without leaking it to clients.
    pub fn internal_server_error() -> Self {
        Self::new(500)
            .title("Internal Server Error")
            .detail("An unexpected error occurred.")
    }
}

// ---------------------------------------------------------------------------
// Builder methods
// ---------------------------------------------------------------------------

impl Problem {
    /// Set the problem type URI.
    ///
    /// The method is named `type_` because `type` is a Rust keyword.
    pub fn type_(mut self, type_uri: impl Into<String>) -> Self {
        self.type_uri = Some(type_uri.into());
        self
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Override the HTTP status code.
    pub fn status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }

    /// Set the public detail message.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Set the instance URI.
    pub fn instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    /// Set the `"code"` extension field — a stable string code for clients.
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.extensions
            .insert("code".into(), serde_json::Value::String(code.into()));
        self
    }

    /// Set the `"trace_id"` extension field.
    pub fn trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.extensions.insert(
            "trace_id".into(),
            serde_json::Value::String(trace_id.into()),
        );
        self
    }

    /// Set the `"request_id"` extension field.
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.extensions.insert(
            "request_id".into(),
            serde_json::Value::String(request_id.into()),
        );
        self
    }

    /// Add an arbitrary extension field.
    pub fn extension(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.extensions.insert(key.into(), value.into());
        self
    }

    /// Append a field-level validation error.
    ///
    /// Creates or appends to the `"errors"` extension array.
    pub fn push_error(self, field: impl Into<String>, message: impl Into<String>) -> Self {
        self.push_validation_item(ValidationItem::new(field, message))
    }

    /// Append a field-level validation error with an error code.
    ///
    /// Creates or appends to the `"errors"` extension array.
    pub fn push_error_code(
        self,
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        self.push_validation_item(ValidationItem::new(field, message).code(code))
    }

    /// Append a [`ValidationItem`] to the `"errors"` extension array.
    fn push_validation_item(mut self, item: ValidationItem) -> Self {
        let value = serde_json::to_value(&item).expect("ValidationItem is always serializable");
        match self.extensions.get_mut("errors") {
            Some(serde_json::Value::Array(arr)) => {
                arr.push(value);
            }
            _ => {
                self.extensions
                    .insert("errors".into(), serde_json::Value::Array(vec![value]));
            }
        }
        self
    }

    /// Replace the `"errors"` extension with a complete list of validation items.
    pub fn errors(mut self, items: Vec<ValidationItem>) -> Self {
        self.extensions.insert(
            "errors".into(),
            serde_json::to_value(items).expect("ValidationItem is always serializable"),
        );
        self
    }

    /// Attach an internal cause for server-side diagnostics.
    ///
    /// The cause is **never serialized** to JSON. Access it via
    /// [`internal_cause`](Self::internal_cause) for logging.
    pub fn with_cause(mut self, err: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.cause = Some(InternalCause {
            source: Box::new(err),
        });
        self
    }

    /// Attach a string message as the internal cause.
    ///
    /// Convenience alternative to [`with_cause`](Self::with_cause) when you
    /// don't have a typed error.
    pub fn with_cause_str(mut self, message: impl Into<String>) -> Self {
        self.cause = Some(InternalCause {
            source: Box::new(StringError(message.into())),
        });
        self
    }
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

impl Problem {
    /// Returns the HTTP status code, defaulting to 500 if not set.
    pub fn status_code(&self) -> u16 {
        self.status.unwrap_or(500)
    }

    /// Returns `true` if the status code is 5xx.
    pub fn is_server_error(&self) -> bool {
        self.status_code() >= 500
    }

    /// Returns the `"code"` extension value, if set.
    pub fn get_code(&self) -> Option<&str> {
        self.extensions.get("code").and_then(|v| v.as_str())
    }

    /// Returns the `"trace_id"` extension value, if set.
    pub fn get_trace_id(&self) -> Option<&str> {
        self.extensions.get("trace_id").and_then(|v| v.as_str())
    }

    /// Returns the internal cause message, if set.
    ///
    /// This value is never included in serialized output and is intended
    /// for server-side logging only.
    pub fn internal_cause(&self) -> Option<&(dyn std::error::Error + Send + Sync)> {
        self.cause.as_ref().map(|c| c.source.as_ref())
    }

    /// Serialize to a pretty-printed JSON string. Useful in tests and debugging.
    pub fn to_json_string_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("Problem is always serializable")
    }
}

// ---------------------------------------------------------------------------
// Trait impls
// ---------------------------------------------------------------------------

impl Default for Problem {
    fn default() -> Self {
        Self {
            type_uri: None,
            title: None,
            status: None,
            detail: None,
            instance: None,
            extensions: Map::new(),
            cause: None,
        }
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

impl std::error::Error for Problem {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|c| c.source.as_ref() as &(dyn std::error::Error + 'static))
    }
}
