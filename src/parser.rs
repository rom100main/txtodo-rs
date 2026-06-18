use crate::date_utils;
use crate::error::TxtodoError;
use crate::extension::ExtensionHandler;
use crate::task::Priority;
use crate::task::Task;
use indexmap::IndexMap;
use std::ptr::NonNull;

/// Parser that converts raw todo.txt content into [`Task`]s.
///
/// Supports custom [`ExtensionHandler`]s for parsing extension key-value pairs and optional hierarchical subtask handling.
///
/// # Examples
///
/// ```
/// # use txtodo::*;
/// # fn main() -> Result<(), TxtodoError> {
/// let parser = TodoTxtParser::new();
/// let tasks = parser.parse_file("(A) 2024-01-15 Buy milk @home +shopping")?;
/// assert_eq!(tasks.len(), 1);
/// assert!(tasks[0].description.contains("Buy milk"));
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct TodoTxtParser {
    handler: ExtensionHandler,
    handle_subtasks: bool,
}

impl Default for TodoTxtParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoTxtParser {
    /// Creates a new parser with default settings (no extensions, subtasks enabled).
    #[must_use]
    pub fn new() -> Self {
        Self {
            handler: ExtensionHandler::new(),
            handle_subtasks: true,
        }
    }

    /// Creates a new parser with a custom [`ExtensionHandler`].
    ///
    /// Subtask handling is enabled by default.
    #[must_use]
    pub fn with_handler(handler: ExtensionHandler) -> Self {
        Self {
            handler,
            handle_subtasks: true,
        }
    }

    #[must_use]
    pub(crate) fn with_options(handler: ExtensionHandler, handle_subtasks: bool) -> Self {
        Self {
            handler,
            handle_subtasks,
        }
    }

    /// Returns a reference to the parser's [`ExtensionHandler`].
    #[must_use]
    pub fn handler(&self) -> &ExtensionHandler {
        &self.handler
    }

    /// Parses the full contents of a todo.txt file into a list of [`Task`]s.
    ///
    /// When subtask handling is enabled (the default),
    /// tasks are assembled into a hierarchy based on indentation.
    /// Blank lines produce empty separator tasks.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError::Parse`] if any line cannot be parsed.
    pub fn parse_file(&self, content: &str) -> Result<Vec<Task>, TxtodoError> {
        let lines: Vec<&str> = content.split('\n').collect();

        // Two-pass: first build flat list, then assemble the tree.
        let mut flat: Vec<Task> = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                flat.push(empty_task());
                continue;
            }
            match self.parse_line_with_parent(line, None) {
                Ok(t) => flat.push(t),
                Err(e) => {
                    return Err(TxtodoError::Parse {
                        message: format!("Failed to parse line {}: {}", idx + 1, e),
                        line: Some(line.to_string()),
                        line_number: Some(idx + 1),
                    });
                }
            }
        }

        if !self.handle_subtasks {
            return Ok(flat);
        }

        let mut roots: Vec<usize> = Vec::new();
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); flat.len()];
        let mut stack: Vec<usize> = Vec::new();

        for (i, task) in flat.iter().enumerate() {
            let indent = task.indent_level;
            if task.raw.is_empty() {
                roots.push(i);
                stack.clear();
                continue;
            }
            if stack.is_empty() {
                roots.push(i);
                stack.push(i);
                continue;
            }
            let mut attached = false;
            for j in (0..stack.len()).rev() {
                if indent > flat[stack[j]].indent_level {
                    children[stack[j]].push(i);
                    stack.truncate(j + 1);
                    stack.push(i);
                    attached = true;
                    break;
                }
            }
            if !attached {
                roots.push(i);
                stack.clear();
                stack.push(i);
            }
        }

        // Materialize tree from flat + children mapping
        let mut out: Vec<Task> = Vec::new();
        for &root_idx in &roots {
            let mut node = flat[root_idx].clone();
            self.attach_subtree(&mut node, &flat, &children, root_idx)?;
            out.push(node);
        }
        relink_parents(&mut out);
        Ok(out)
    }

    /// Parses a single todo.txt line into a [`Task`].
    ///
    /// Takes into account the parser's [`ExtensionHandler`] and
    /// allows specifying a `parent` task so that extension inheritance and
    /// project/context inheritance are applied.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// # fn main() -> Result<(), TxtodoError> {
    /// let parser = TodoTxtParser::new();
    /// let task = parser.parse_line_with_parent("(A) Buy milk +shopping", None)?;
    /// assert_eq!(task.priority, Some(Priority('A')));
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_line_with_parent(
        &self,
        line: &str,
        parent: Option<&Task>,
    ) -> Result<Task, TxtodoError> {
        let indent_level = get_indent_level(line);
        let trimmed = line.trim();
        let mut task = Task {
            raw: line.to_string(),
            completed: false,
            priority: None,
            creation_date: None,
            completion_date: None,
            description: String::new(),
            projects: Vec::new(),
            contexts: Vec::new(),
            extensions: IndexMap::new(),
            subtasks: Vec::new(),
            indent_level,
            parent: None,
        };

        if trimmed.starts_with("x ") {
            task.completed = true;
            parse_completed_task(trimmed, &mut task)?;
        } else {
            parse_incomplete_task(trimmed, &mut task)?;
        }

        let (projects, contexts) = extract_projects_and_contexts(&task.description);
        task.projects = projects;
        task.contexts = contexts;

        task.extensions = self.handler.parse_extensions(&task.description, parent)?;

        if let Some(p) = parent {
            inherit_parent_properties(&mut task, p);
        }

        Ok(task)
    }

    /// Parses a single todo.txt line into a [`Task`].
    ///
    /// Equivalent to [`parse_line_with_parent`](TodoTxtParser::parse_line_with_parent)
    /// with `parent` set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// # fn main() -> Result<(), TxtodoError> {
    /// let parser = TodoTxtParser::new();
    /// let task = parser.parse_line("(A) Buy milk +shopping")?;
    /// assert_eq!(task.priority, Some(Priority('A')));
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_line(&self, line: &str) -> Result<Task, TxtodoError> {
        self.parse_line_with_parent(line, None)
    }

    fn attach_subtree(
        &self,
        node: &mut Task,
        flat: &[Task],
        children: &[Vec<usize>],
        idx: usize,
    ) -> Result<(), TxtodoError> {
        for &child_idx in &children[idx] {
            let mut child = self.parse_line_with_parent(&flat[child_idx].raw, Some(node))?;
            self.attach_subtree(&mut child, flat, children, child_idx)?;
            node.subtasks.push(child);
        }
        Ok(())
    }
}

fn empty_task() -> Task {
    Task {
        raw: String::new(),
        completed: false,
        priority: None,
        creation_date: None,
        completion_date: None,
        description: String::new(),
        projects: Vec::new(),
        contexts: Vec::new(),
        extensions: IndexMap::new(),
        subtasks: Vec::new(),
        indent_level: 0,
        parent: None,
    }
}

fn get_indent_level(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}

fn is_priority_token(token: &str) -> bool {
    if token.len() != 3 {
        return false;
    }
    let bytes = token.as_bytes();
    bytes[0] == b'(' && bytes[2] == b')' && bytes[1].is_ascii_uppercase()
}

fn parse_completed_task(line: &str, task: &mut Task) -> Result<(), TxtodoError> {
    let parts: Vec<&str> = line.split(' ').collect();
    let mut remaining: Vec<&str> = parts.into_iter().skip(1).collect();

    if let Some(&first) = remaining.first()
        && date_utils::is_date(first)
    {
        task.completion_date = Some(date_utils::parse_date(first)?);
        remaining.remove(0);
    }

    if let Some(&first) = remaining.first()
        && is_priority_token(first)
    {
        task.priority = Some(Priority::from_token(first)?);
        remaining.remove(0);
    }

    if let Some(&first) = remaining.first()
        && date_utils::is_date(first)
    {
        task.creation_date = Some(date_utils::parse_date(first)?);
        remaining.remove(0);
    }

    task.description = remaining.join(" ");
    Ok(())
}

fn parse_incomplete_task(line: &str, task: &mut Task) -> Result<(), TxtodoError> {
    let parts: Vec<&str> = line.split(' ').collect();
    let mut remaining: Vec<&str> = parts.into_iter().collect();

    if let Some(&first) = remaining.first()
        && is_priority_token(first)
    {
        task.priority = Some(Priority::from_token(first)?);
        remaining.remove(0);
    }

    if let Some(&first) = remaining.first()
        && date_utils::is_date(first)
    {
        task.creation_date = Some(date_utils::parse_date(first)?);
        remaining.remove(0);
    }

    task.description = remaining.join(" ");
    Ok(())
}

fn extract_projects_and_contexts(description: &str) -> (Vec<String>, Vec<String>) {
    let mut projects = Vec::new();
    let mut contexts = Vec::new();
    let chars: Vec<char> = description.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '+' && i + 1 < chars.len() {
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            if end > start {
                let proj: String = chars[start..end].iter().collect();
                if !projects.contains(&proj) {
                    projects.push(proj);
                }
            }
            i = end;
        } else if c == '@' && i + 1 < chars.len() {
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            if end > start {
                let ctx: String = chars[start..end].iter().collect();
                if !contexts.contains(&ctx) {
                    contexts.push(ctx);
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
    (projects, contexts)
}

fn inherit_parent_properties(task: &mut Task, parent: &Task) {
    if task.projects.is_empty() && !parent.projects.is_empty() {
        task.projects = parent.projects.clone();
    }
    if task.contexts.is_empty() && !parent.contexts.is_empty() {
        task.contexts = parent.contexts.clone();
    }
}

fn relink_parents(tasks: &mut [Task]) {
    for t in tasks.iter_mut() {
        relink_parents_inner(t);
    }
}

fn relink_parents_inner(task: &mut Task) {
    let raw: *mut Task = task;
    for child in task.subtasks.iter_mut() {
        child.parent = NonNull::new(raw);
        relink_parents_inner(child);
    }
}
