use crate::date_utils;
use crate::error::TxtodoError;
use indexmap::IndexMap;
use std::cmp::Ordering;
use std::fmt;
use std::ptr::NonNull;

/// A single uppercase A–Z character representing task priority.
///
/// `(A)` is the highest priority, `(Z)` the lowest.
/// Displayed in parentheses in the todo.txt format, e.g. `(A)`.
///
/// # Examples
///
/// ```rust
/// # use txtodo::*;
/// fn main() -> Result<(), TxtodoError> {
///     let p = Priority::from_token("(A)")?;
///     assert_eq!(p.as_char(), 'A');
///     assert_eq!(p.to_string(), "(A)");
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Priority(pub char);

impl Priority {
    /// All 26 priority levels from `A` (highest) to `Z` (lowest).
    pub const ALL: [Priority; 26] = [
        Priority('A'),
        Priority('B'),
        Priority('C'),
        Priority('D'),
        Priority('E'),
        Priority('F'),
        Priority('G'),
        Priority('H'),
        Priority('I'),
        Priority('J'),
        Priority('K'),
        Priority('L'),
        Priority('M'),
        Priority('N'),
        Priority('O'),
        Priority('P'),
        Priority('Q'),
        Priority('R'),
        Priority('S'),
        Priority('T'),
        Priority('U'),
        Priority('V'),
        Priority('W'),
        Priority('X'),
        Priority('Y'),
        Priority('Z'),
    ];

    /// Parses a priority from a token such as `"(A)"` or `"A"`.
    ///
    /// The token must be either a single ASCII uppercase character or
    /// a three-character string wrapped in parentheses (e.g. `(B)`).
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError::Priority`] if the token is not a valid
    /// uppercase ASCII letter in the expected format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let p = Priority::from_token("(C)")?;
    ///     assert_eq!(p.as_char(), 'C');
    ///     Ok(())
    /// }
    /// ```
    pub fn from_token(token: &str) -> Result<Self, TxtodoError> {
        let inner = if token.starts_with('(') && token.ends_with(')') && token.len() == 3 {
            &token[1..2]
        } else if token.len() == 1 {
            token
        } else {
            return Err(TxtodoError::Priority {
                message: format!("Invalid priority token: {token}"),
                priority: Some(token.to_string()),
            });
        };

        let ch = inner.chars().next().unwrap();
        if !ch.is_ascii_uppercase() {
            return Err(TxtodoError::Priority {
                message: format!("Invalid priority character: {ch}"),
                priority: Some(token.to_string()),
            });
        }
        Ok(Priority(ch))
    }

    /// Returns the inner `A`–`Z` character.
    #[must_use]
    pub fn as_char(self) -> char {
        self.0
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.0)
    }
}

/// A value stored by an [extension](crate::extension::ExtensionHandler)
/// in a task's metadata.
///
/// Extensions map string keys to `ExtensionValue`s,
/// enabling typed custom fields beyond the standard todo.txt format.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionValue {
    /// A UTF-8 text value.
    String(String),
    /// A 64-bit floating-point numeric value.
    Number(f64),
    /// A boolean flag.
    Boolean(bool),
    /// A calendar date (no time component).
    Date(time::Date),
    /// An ordered collection of [`ExtensionValue`]s.
    Array(Vec<ExtensionValue>),
}

impl ExtensionValue {
    /// Deep structural equality check that compares nested [`Array`](ExtensionValue::Array)
    /// contents element-by-element.
    #[must_use]
    pub fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (ExtensionValue::String(a), ExtensionValue::String(b)) => a == b,
            (ExtensionValue::Number(a), ExtensionValue::Number(b)) => a == b,
            (ExtensionValue::Boolean(a), ExtensionValue::Boolean(b)) => a == b,
            (ExtensionValue::Date(a), ExtensionValue::Date(b)) => a == b,
            (ExtensionValue::Array(a), ExtensionValue::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            _ => false,
        }
    }

    pub(crate) fn compare_to(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ExtensionValue::String(a), ExtensionValue::String(b)) => a.cmp(b),
            (ExtensionValue::Number(a), ExtensionValue::Number(b)) => {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            (ExtensionValue::Boolean(a), ExtensionValue::Boolean(b)) => {
                Ordering::Equal.then_with(|| a.cmp(b))
            }
            (ExtensionValue::Date(a), ExtensionValue::Date(b)) => a.cmp(b),
            (ExtensionValue::Array(a), ExtensionValue::Array(b)) => {
                for (x, y) in a.iter().zip(b.iter()) {
                    let c = x.compare_to(y);
                    if c != Ordering::Equal {
                        return c;
                    }
                }
                a.len().cmp(&b.len())
            }
            _ => Ordering::Equal,
        }
    }

    /// Compares two values of the **same** variant, returning their ordering.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError::Extension`] if the two values are different
    /// variants and cannot be compared.
    pub fn try_compare_to(&self, other: &Self) -> Result<Ordering, TxtodoError> {
        match (self, other) {
            (ExtensionValue::String(_), ExtensionValue::String(_))
            | (ExtensionValue::Number(_), ExtensionValue::Number(_))
            | (ExtensionValue::Boolean(_), ExtensionValue::Boolean(_))
            | (ExtensionValue::Date(_), ExtensionValue::Date(_))
            | (ExtensionValue::Array(_), ExtensionValue::Array(_)) => Ok(self.compare_to(other)),
            _ => Err(TxtodoError::Extension {
                message: "Cannot compare different extension value types".to_string(),
                extension_key: None,
            }),
        }
    }
}

impl fmt::Display for ExtensionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtensionValue::String(s) => write!(f, "{s}"),
            ExtensionValue::Number(n) => {
                if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e16 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            ExtensionValue::Boolean(b) => write!(f, "{b}"),
            ExtensionValue::Date(d) => write!(f, "{}", date_utils::format_date(*d)),
            ExtensionValue::Array(arr) => {
                let parts: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "{}", parts.join(","))
            }
        }
    }
}

/// A single task parsed from a todo.txt line.
///
/// Tasks are the core domain type. Every field corresponds to a part of the [todo.txt](https://todotxt.org) specification
/// plus txtodo extensions for subtasks and indentation.
#[derive(Debug, Clone)]
pub struct Task {
    /// The original, unmodified line of text this task was parsed from.
    pub raw: String,
    /// Whether the task is marked as done (`x` prefix).
    pub completed: bool,
    /// Optional single-letter priority, e.g. [`Priority`]`(A)`.
    pub priority: Option<Priority>,
    /// The date the task was created (second date token for incomplete tasks).
    pub creation_date: Option<time::Date>,
    /// The date the task was completed (first date token after `x`).
    pub completion_date: Option<time::Date>,
    /// The task text after stripping priority, dates, and the completion marker.
    pub description: String,
    /// Project tags (`+project`) extracted from the description.
    pub projects: Vec<String>,
    /// Context tags (`@context`) extracted from the description.
    pub contexts: Vec<String>,
    /// Key-value pairs parsed by the active extension handler.
    pub extensions: IndexMap<String, ExtensionValue>,
    /// Child tasks indented beneath this task.
    pub subtasks: Vec<Task>,
    /// Leading whitespace depth used to nest subtasks.
    pub indent_level: usize,
    /// Pointer to the parent [`Task`], if this is a subtask.
    pub parent: Option<NonNull<Task>>,
}

impl Task {
    /// Returns a reference to the parent [`Task`],
    /// or `None` if this is a top-level task.
    #[must_use]
    pub fn parent(&self) -> Option<&Task> {
        self.parent.map(|p| unsafe { p.as_ref() })
    }
}

/// A partial update for a [`Task`].
///
/// Each field is `Option<Option<T>>`,
/// `None` means "leave unchanged",
/// while `Some(None)` means "clear the value"
/// and `Some(v)` means "set to `v`".
///
/// # Examples
///
/// ```rust
/// # use txtodo::*;
/// fn main() -> Result<(), TxtodoError> {
///     let patch = TaskPatch {
///         description: Some("buy milk".into()),
///         priority: Some(None), // clear priority
///         ..Default::default()
///     };
///     Ok(())
/// }
/// ```
#[derive(Debug, Default, Clone)]
pub struct TaskPatch {
    /// Replacement raw line, or `None` to keep the existing one.
    pub raw: Option<String>,
    /// Override the completion flag, or `None` to keep it unchanged.
    pub completed: Option<bool>,
    /// Set a new [`Priority`], `Some(None)` to clear it, or `None` to keep it.
    pub priority: Option<Option<Priority>>,
    /// Set a new creation date, `Some(None)` to clear it, or `None` to keep it.
    pub creation_date: Option<Option<time::Date>>,
    /// Set a new completion date, `Some(None)` to clear it, or `None` to keep it.
    pub completion_date: Option<Option<time::Date>>,
    /// Replacement description text, or `None` to keep the existing one.
    pub description: Option<String>,
    /// Replacement project tags, or `None` to keep the existing ones.
    pub projects: Option<Vec<String>>,
    /// Replacement context tags, or `None` to keep the existing ones.
    pub contexts: Option<Vec<String>>,
    /// Replacement extension map, or `None` to keep the existing one.
    pub extensions: Option<IndexMap<String, ExtensionValue>>,
}

impl TaskPatch {
    pub(crate) fn apply(self, task: &mut Task) {
        if let Some(v) = self.raw {
            task.raw = v;
        }
        if let Some(v) = self.completed {
            task.completed = v;
        }
        if let Some(v) = self.priority {
            task.priority = v;
        }
        if let Some(v) = self.creation_date {
            task.creation_date = v;
        }
        if let Some(v) = self.completion_date {
            task.completion_date = v;
        }
        if let Some(v) = self.description {
            task.description = v;
        }
        if let Some(v) = self.projects {
            task.projects = v;
        }
        if let Some(v) = self.contexts {
            task.contexts = v;
        }
        if let Some(v) = self.extensions {
            task.extensions = v;
        }
    }
}
