use crate::date_utils;
use crate::error::TxtodoError;
use crate::extension::ExtensionHandler;
use crate::task::Task;

/// Serializer that converts [`Task`]s back into the todo.txt format.
///
/// Supports custom [`ExtensionHandler`]s for serializing extension key-value pairs.
///
/// # Examples
///
/// ```
/// # use txtodo::*;
/// # fn main() -> Result<(), TxtodoError> {
/// let parser = TodoTxtParser::new();
/// let tasks = parser.parse_file("(A) 2024-01-15 Buy milk @home +shopping")?;
/// let serializer = TodoTxtSerializer::new();
/// let output = serializer.serialize_tasks(&tasks)?;
/// assert!(output.contains("Buy milk"));
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct TodoTxtSerializer {
    handler: ExtensionHandler,
}

impl Default for TodoTxtSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoTxtSerializer {
    /// Creates a new serializer with default settings (no extensions).
    #[must_use]
    pub fn new() -> Self {
        Self {
            handler: ExtensionHandler::new(),
        }
    }

    /// Creates a new serializer with a custom [`ExtensionHandler`].
    #[must_use]
    pub fn with_handler(handler: ExtensionHandler) -> Self {
        Self { handler }
    }

    /// Returns a reference to the serializer's [`ExtensionHandler`].
    #[must_use]
    pub fn handler(&self) -> &ExtensionHandler {
        &self.handler
    }

    /// Serializes a slice of [`Task`]s into a single todo.txt string.
    ///
    /// Each task (including its subtasks) is rendered on its own line,
    /// with subtasks indented according to their nesting level.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError::Serialization`] if any task fails to serialize.
    pub fn serialize_tasks(&self, tasks: &[Task]) -> Result<String, TxtodoError> {
        let mut lines: Vec<String> = Vec::new();
        for task in tasks {
            lines.extend(self.serialize_task(task)?);
        }
        Ok(lines.join("\n"))
    }

    /// Serializes a single [`Task`] (and its subtasks) into a list of lines.
    ///
    /// Returns one line per task/subtask, with indentation reflecting the subtask hierarchy.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError::Serialization`] if the task or any of its subtasks fail to serialize.
    pub fn serialize_task(&self, task: &Task) -> Result<Vec<String>, TxtodoError> {
        let mut out = Vec::new();
        match self.serialize_single(task, task.indent_level) {
            Ok(line) => out.push(line),
            Err(e) => {
                return Err(TxtodoError::Serialization {
                    message: format!("Failed to serialize task: {e}"),
                });
            }
        }
        for sub in &task.subtasks {
            match self.serialize_task(sub) {
                Ok(lines) => out.extend(lines),
                Err(e) => {
                    return Err(TxtodoError::Serialization {
                        message: format!("Failed to serialize subtask: {e}"),
                    });
                }
            }
        }
        Ok(out)
    }

    fn serialize_single(&self, task: &Task, indent_level: usize) -> Result<String, TxtodoError> {
        let mut prefix_parts: Vec<String> = Vec::new();
        let indent = if indent_level > 0 {
            " ".repeat(indent_level)
        } else {
            String::new()
        };

        if task.completed {
            prefix_parts.push("x".to_string());
            if let Some(d) = task.completion_date {
                prefix_parts.push(date_utils::format_date(d));
            }
            if let Some(p) = task.priority {
                prefix_parts.push(p.to_string());
            }
            if let Some(d) = task.creation_date {
                prefix_parts.push(date_utils::format_date(d));
            }
        } else {
            if let Some(p) = task.priority {
                prefix_parts.push(p.to_string());
            }
            if let Some(d) = task.creation_date {
                prefix_parts.push(date_utils::format_date(d));
            }
        }

        // Description with extensions replaced in-place
        let mut description = task.description.clone();
        let serialized_exts = self.handler.serialize_extensions(&task.extensions)?;
        for ext_token in &serialized_exts {
            description = replace_extension_token(&description, ext_token);
        }
        let description = description.trim().to_string();

        let prefix = prefix_parts.join(" ");
        let line = if !prefix.is_empty() {
            format!("{prefix} {description}")
        } else {
            description
        };

        let result = if !indent.is_empty() {
            format!("{indent}{}", line.trim_start())
        } else {
            line
        };

        Ok(result.trim_end().to_string())
    }
}

fn replace_extension_token(text: &str, new_token: &str) -> String {
    // Extract the key from the new token (everything before the first ':')
    let key = match new_token.split_once(':') {
        Some((k, _)) => k,
        None => return text.to_string(),
    };

    let chars: Vec<char> = text.chars().collect();
    let new_chars: Vec<char> = new_token.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let at_start = i == 0;
        if at_start || chars[i].is_whitespace() {
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            let word_start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            if i > word_start && i < chars.len() && chars[i] == ':' {
                let word: String = chars[word_start..i].iter().collect();
                if word == key {
                    let val_start = i + 1;
                    let mut val_end = val_start;
                    while val_end < chars.len() && !chars[val_end].is_whitespace() {
                        val_end += 1;
                    }
                    if val_end > val_start {
                        // Found the token, replace it in-place
                        let mut result = String::with_capacity(
                            word_start + new_chars.len() + chars.len() - val_end,
                        );
                        result.extend(chars[..word_start].iter());
                        result.extend(new_chars.iter());
                        result.extend(chars[val_end..].iter());
                        return result;
                    }
                }
            }
        } else {
            i += 1;
        }
    }

    text.to_string()
}
