use serde::{Deserialize, Serialize};

/// A single field-level validation error.
///
/// Used within the `"errors"` extension key to report multiple validation
/// failures in a structured format suitable for frontend form binding.
///
/// # Example
///
/// ```
/// use rust_rfc7807::ValidationItem;
///
/// let item = ValidationItem::new("email", "must be a valid email address")
///     .code("INVALID_EMAIL");
///
/// let json = serde_json::to_value(&item).unwrap();
/// assert_eq!(json["field"], "email");
/// assert_eq!(json["code"], "INVALID_EMAIL");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationItem {
    /// The field path that failed validation (e.g., `"email"` or `"address.zip"`).
    pub field: String,
    /// A human-readable description of the validation failure.
    pub message: String,
    /// An optional machine-readable error code for this specific validation failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl ValidationItem {
    /// Create a new validation item for the given field and message.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: None,
        }
    }

    /// Set an error code for this validation item.
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}
