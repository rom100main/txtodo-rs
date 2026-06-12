use crate::date_utils;
use crate::error::TxtodoError;
use crate::extension::ExtensionHandler;
use crate::task::Task;

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
    pub fn new() -> Self {
        Self {
            handler: ExtensionHandler::new(),
        }
    }

    pub fn with_handler(handler: ExtensionHandler) -> Self {
        Self { handler }
    }

    pub fn handler(&self) -> &ExtensionHandler {
        &self.handler
    }

    pub fn serialize_tasks(&self, tasks: &[Task]) -> Result<String, TxtodoError> {
        let mut lines: Vec<String> = Vec::new();
        for task in tasks {
            lines.extend(self.serialize_task(task)?);
        }
        Ok(lines.join("\n"))
    }

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

        // Description with extensions stripped
        let description_clean = strip_extension_tokens(&task.description);
        let serialized_exts = self.handler.serialize_extensions(&task.extensions)?;
        let exts_str = serialized_exts.join(" ");
        let mut description = description_clean.trim().to_string();
        if !exts_str.is_empty() {
            if !description.is_empty() {
                description.push(' ');
            }
            description.push_str(&exts_str);
        }

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

fn strip_extension_tokens(text: &str) -> String {
    // Match pattern: (optional whitespace)(word chars)(:)(non-whitespace)+
    // Mirrors regex /(\s+|^)\w+:[^\s]+/g
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let at_start = i == 0;
        if at_start || chars[i].is_whitespace() {
            let ws_start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            let word_start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            if i > word_start && i < chars.len() && chars[i] == ':' {
                let val_start = i + 1;
                let mut val_end = val_start;
                while val_end < chars.len() && !chars[val_end].is_whitespace() {
                    val_end += 1;
                }
                if val_end > val_start {
                    // Whole token: drop the leading whitespace + token
                    i = val_end;
                    continue;
                } else {
                    // No value: emit word and continue
                    let s: String = chars[ws_start..word_start].iter().collect();
                    out.push_str(&s);
                    let s: String = chars[word_start..i].iter().collect();
                    out.push_str(&s);
                    continue;
                }
            } else {
                // Plain whitespace or start-of-string + word
                let s: String = chars[ws_start..i].iter().collect();
                out.push_str(&s);
                continue;
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}
