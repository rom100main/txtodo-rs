use thiserror::Error;

/// The unified error type for all txtodo operations.
///
/// Each variant carries contextual information to help diagnose the failure.
/// Use [`code()`](TxtodoError::code) to obtain a stable, machine-readable error code string.
///
/// # Examples
///
/// ```
/// # use txtodo::*;
/// fn main() -> Result<(), TxtodoError> {
///     let parser = TodoTxtParser::new();
///     match parser.parse_file("bad line with invalid date 9999-99-99") {
///         Ok(_) => Ok(()),
///         Err(e) => {
///             assert!(!e.code().is_empty());
///             Ok(())
///         }
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum TxtodoError {
    /// A task line could not be parsed.
    #[error("Parse error: {message}")]
    Parse {
        /// Human-readable description of the parse failure.
        message: String,
        /// The raw line that failed to parse, if available.
        line: Option<String>,
        /// The 1-based line number in the source file.
        line_number: Option<usize>,
    },

    /// An error originating from an extension handler.
    #[error("Extension error: {message}")]
    Extension {
        /// Human-readable description of the extension failure.
        message: String,
        /// The extension key that caused the error, if identifiable.
        extension_key: Option<String>,
    },

    /// A task could not be serialized back to the todo.txt format.
    #[error("Serialization error: {message}")]
    Serialization {
        /// Human-readable description of the serialization failure.
        message: String,
    },

    /// A field value failed validation.
    #[error("Validation error: {message}")]
    Validation {
        /// Human-readable description of the validation failure.
        message: String,
        /// The name of the field that failed validation, if identifiable.
        field: Option<String>,
    },

    /// A date-related error (e.g. invalid format or out-of-range values).
    #[error("Date error: {message}")]
    Date {
        /// Human-readable description of the date failure.
        message: String,
        /// The date string that caused the error, if available.
        date_str: Option<String>,
    },

    /// A priority value is invalid or unsupported.
    #[error("Priority error: {message}")]
    Priority {
        /// Human-readable description of the priority failure.
        message: String,
        /// The invalid priority string, if available.
        priority: Option<String>,
    },

    /// A catch-all error for miscellaneous failures.
    #[error("{0}")]
    Generic(String),

    /// An I/O error (file read/write failure, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A date string could not be parsed by the `time` crate.
    #[error("Invalid date: {0}")]
    DateParse(#[from] time::error::Parse),
}

impl TxtodoError {
    /// Returns a stable, machine-readable error code for this error variant.
    ///
    /// Useful for programmatic error handling or logging. Returns an empty
    /// string for [`TxtodoError::Generic`].
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            TxtodoError::Parse { .. } => "PARSE_ERROR",
            TxtodoError::Extension { .. } => "EXTENSION_ERROR",
            TxtodoError::Serialization { .. } => "SERIALIZATION_ERROR",
            TxtodoError::Validation { .. } => "VALIDATION_ERROR",
            TxtodoError::Date { .. } => "DATE_ERROR",
            TxtodoError::Priority { .. } => "PRIORITY_ERROR",
            TxtodoError::Generic(_) => "",
            TxtodoError::Io(_) => "IO_ERROR",
            TxtodoError::DateParse(_) => "DATE_PARSE_ERROR",
        }
    }
}
