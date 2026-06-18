use crate::extension::TodoTxtExtension;

/// Configuration options for the [`TodoTxt`](crate::todotxt::TodoTxt) manager.
///
/// Controls file persistence, extension registration, and subtask handling.
/// Implements [`Default`] with sensible defaults.
///
/// # Examples
///
/// ```
/// # use txtodo::*;
/// let opts = TodoOptions {
///     file_path: Some("todo.txt".into()),
///     auto_save: true,
///     extensions: Vec::new(),
///     handle_subtasks: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TodoOptions {
    /// Path to the todo.txt file on disk.
    ///
    /// When set, tasks are loaded from and saved to this file.
    pub file_path: Option<String>,
    /// Whether to automatically save changes to disk after every mutation.
    ///
    /// Defaults to `false`.
    pub auto_save: bool,
    /// Registered [`TodoTxtExtension`]s for parsing and serializing custom key-value pairs (e.g. `due:2024-01-15`) by default fallback auto-detect values type.
    pub extensions: Vec<TodoTxtExtension>,
    /// Whether to parse and maintain hierarchical subtask relationships based on indentation.
    ///
    /// Defaults to `true`.
    pub handle_subtasks: bool,
}

impl Default for TodoOptions {
    fn default() -> Self {
        Self {
            file_path: None,
            auto_save: false,
            extensions: Vec::new(),
            handle_subtasks: true,
        }
    }
}
