use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxtodoError {
    #[error("Parse error: {message}")]
    Parse {
        message: String,
        line: Option<String>,
        line_number: Option<usize>,
    },

    #[error("Extension error: {message}")]
    Extension {
        message: String,
        extension_key: Option<String>,
    },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    #[error("Date error: {message}")]
    Date {
        message: String,
        date_str: Option<String>,
    },

    #[error("Priority error: {message}")]
    Priority {
        message: String,
        priority: Option<String>,
    },

    #[error("{0}")]
    Generic(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid date: {0}")]
    DateParse(#[from] time::error::Parse),
}

impl TxtodoError {
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
