use crate::error::TxtodoError;
use crate::extension::ExtensionHandler;
use crate::task::{Task, build_task_from_line};
use indexmap::IndexMap;
use std::ptr::NonNull;

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
    #[must_use]
    pub fn new() -> Self {
        Self {
            handler: ExtensionHandler::new(),
            handle_subtasks: true,
        }
    }

    #[must_use]
    pub fn with_handler(handler: ExtensionHandler) -> Self {
        Self {
            handler,
            handle_subtasks: true,
        }
    }

    #[must_use]
    pub fn with_options(handler: ExtensionHandler, handle_subtasks: bool) -> Self {
        Self {
            handler,
            handle_subtasks,
        }
    }

    #[must_use]
    pub fn handler(&self) -> &ExtensionHandler {
        &self.handler
    }

    #[must_use]
    pub fn handle_subtasks(&self) -> bool {
        self.handle_subtasks
    }

    pub fn parse_file(&self, content: &str) -> Result<Vec<Task>, TxtodoError> {
        let lines: Vec<&str> = content.split('\n').collect();

        // Two-pass: first build flat list, then assemble the tree.
        let mut flat: Vec<Task> = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                flat.push(empty_task());
                continue;
            }
            match build_task_from_line(line, &self.handler, None) {
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
            attach_subtree(&mut node, &flat, &children, root_idx, &self.handler)?;
            out.push(node);
        }
        relink_parents(&mut out);
        Ok(out)
    }

    pub fn parse_line(&self, line: &str) -> Result<Task, TxtodoError> {
        build_task_from_line(line, &self.handler, None)
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

fn attach_subtree(
    node: &mut Task,
    flat: &[Task],
    children: &[Vec<usize>],
    idx: usize,
    handler: &ExtensionHandler,
) -> Result<(), TxtodoError> {
    for &child_idx in &children[idx] {
        // Re-parse with the parent so extension inheritance is applied.
        let mut child = build_task_from_line(&flat[child_idx].raw, handler, Some(node))?;
        attach_subtree(&mut child, flat, children, child_idx, handler)?;
        node.subtasks.push(child);
    }
    Ok(())
}

pub fn relink_parents(tasks: &mut [Task]) {
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
