use crate::error::TxtodoError;
use crate::extension::ExtensionHandler;
use crate::options::TodoOptions;
use crate::parser::TodoTxtParser;
use crate::serializer::TodoTxtSerializer;
use crate::task::Task;
use crate::task::TaskPatch;
use std::fs;
use std::path::Path;

pub enum TaskInput {
    Line(String),
    Task(Task),
}

impl From<&str> for TaskInput {
    fn from(s: &str) -> Self {
        TaskInput::Line(s.to_string())
    }
}

impl From<String> for TaskInput {
    fn from(s: String) -> Self {
        TaskInput::Line(s)
    }
}

impl From<Task> for TaskInput {
    fn from(t: Task) -> Self {
        TaskInput::Task(t)
    }
}

pub struct TodoTxt {
    pub tasks: Vec<Task>,
    pub file_path: Option<String>,
    pub auto_save: bool,
    pub handle_subtasks: bool,
    pub parser: TodoTxtParser,
    pub serializer: TodoTxtSerializer,
    pub handler: ExtensionHandler,
}

impl std::fmt::Debug for TodoTxt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TodoTxt")
            .field("tasks", &self.tasks.len())
            .field("file_path", &self.file_path)
            .field("auto_save", &self.auto_save)
            .field("handle_subtasks", &self.handle_subtasks)
            .finish()
    }
}

impl TodoTxt {
    pub fn new(options: TodoOptions) -> Result<Self, TxtodoError> {
        let handle_subtasks = options.handle_subtasks;
        let handler = ExtensionHandler::with_extensions(options.extensions.clone())?;
        let parser = TodoTxtParser::with_options(handler.clone(), handle_subtasks);
        let serializer = TodoTxtSerializer::with_handler(handler.clone());

        let file_path = options.file_path.clone();

        Ok(Self {
            tasks: Vec::new(),
            file_path,
            auto_save: options.auto_save,
            handle_subtasks,
            parser,
            serializer,
            handler,
        })
    }

    pub fn load(&mut self, file_path: Option<&str>) -> Result<(), TxtodoError> {
        let path_str = resolve_path(self.file_path.as_deref(), file_path)?;
        let content = fs::read_to_string(&path_str)?;
        self.tasks = self.parser.parse_file(&content)?;
        crate::parser::relink_parents(&mut self.tasks);
        if self.file_path.is_none() {
            self.file_path = Some(path_str);
        }
        Ok(())
    }

    pub fn save(&self, file_path: Option<&str>) -> Result<(), TxtodoError> {
        let path_str = resolve_path(self.file_path.as_deref(), file_path)?;
        let content = self.serializer.serialize_tasks(&self.tasks)?;
        fs::write(&path_str, content)?;
        Ok(())
    }

    pub fn set_auto_save(&mut self, on: bool) {
        self.auto_save = on;
    }

    pub fn extension_handler(&self) -> &ExtensionHandler {
        &self.handler
    }

    pub fn extension_handler_mut(&mut self) -> &mut ExtensionHandler {
        &mut self.handler
    }

    pub fn list(&self) -> Vec<&Task> {
        self.list_filtered(None, None)
    }

    pub fn list_filtered(
        &self,
        filter: Option<&crate::TaskFilter>,
        sorter: Option<&crate::TaskSorter>,
    ) -> Vec<&Task> {
        let mut flat: Vec<&Task> = Vec::new();
        for t in &self.tasks {
            flatten_into(&mut flat, t);
        }
        if let Some(f) = filter {
            flat.retain(|t| f(t));
        }
        if let Some(s) = sorter {
            flat.sort_by(|a, b| s(a, b));
        }
        flat
    }

    pub fn filter(&self, filter: &crate::TaskFilter) -> Vec<Task> {
        let mut out = Vec::new();
        for t in &self.tasks {
            if let Some(clone) = filter_tree(t, filter) {
                out.push(clone);
            }
        }
        out
    }

    pub fn sort(&mut self, sorter: &crate::TaskSorter) {
        sort_recursive(&mut self.tasks, sorter);
    }

    pub fn add(&mut self, input: impl Into<TaskInput>) -> Result<(), TxtodoError> {
        let input = input.into();
        let task = match input {
            TaskInput::Line(s) => self.parser.parse_line(&s)?,
            TaskInput::Task(t) => t,
        };
        if self.handle_subtasks {
            self.add_task_with_subtasks(task, true)?;
        } else {
            self.tasks.push(task);
        }
        self.save_if_needed()
    }

    pub fn add_many(
        &mut self,
        inputs: impl IntoIterator<Item = impl Into<TaskInput>>,
    ) -> Result<(), TxtodoError> {
        let mut tasks: Vec<Task> = Vec::new();
        for input in inputs {
            let input = input.into();
            let task = match input {
                TaskInput::Line(s) => self.parser.parse_line(&s)?,
                TaskInput::Task(t) => t,
            };
            tasks.push(task);
        }
        if self.handle_subtasks {
            self.add_tasks_batch(tasks, true)?;
        } else {
            for t in tasks {
                self.tasks.push(t);
            }
        }
        self.save_if_needed()
    }

    pub fn insert(&mut self, index: i64, input: impl Into<TaskInput>) -> Result<(), TxtodoError> {
        let input = input.into();
        let task = match input {
            TaskInput::Line(s) => self.parser.parse_line(&s)?,
            TaskInput::Task(t) => t,
        };

        let flat_len = self.flat_len() as i64;
        let mut resolved = if index < 0 { flat_len + index } else { index };
        if resolved < 0 {
            resolved = 0;
        }

        let at_end = resolved >= flat_len || (index == -(flat_len) && task.indent_level == 0);

        if at_end {
            return self.add(task);
        }

        if self.handle_subtasks {
            // Re-parse the whole file with the new raw line inserted.
            let mut raws: Vec<String> = self.list().iter().map(|t| t.raw.clone()).collect();
            let insert_at = if task.indent_level > 0 || index >= 0 {
                (resolved as usize + 1).min(raws.len())
            } else {
                (resolved as usize).min(raws.len())
            };
            raws.insert(insert_at, task.raw.clone());
            self.tasks = self.parser.parse_file(&raws.join("\n"))?;
            crate::parser::relink_parents(&mut self.tasks);
        } else {
            let insert_at = if index >= 0 {
                (resolved as usize + 1).min(self.tasks.len())
            } else {
                (resolved as usize).min(self.tasks.len())
            };
            self.tasks.insert(insert_at, task);
        }
        self.save_if_needed()
    }

    pub fn mark(&mut self, numbers: impl IntoIterator<Item = i64>) -> Result<(), TxtodoError> {
        let nums: Vec<i64> = numbers.into_iter().collect();
        let flat_len = self.list().len();
        let mut indices: Vec<usize> = Vec::new();
        for n in &nums {
            indices.push(resolve_index(*n, flat_len)?);
        }
        indices.sort_unstable();
        indices.dedup();
        for &idx in &indices {
            if let Some(t) = Self::nth_flat_mut(&mut self.tasks, idx) {
                t.completed = true;
                if t.completion_date.is_none() {
                    t.completion_date = Some(time::OffsetDateTime::now_utc().date());
                }
            }
        }
        self.save_if_needed()
    }

    pub fn unmark(&mut self, numbers: impl IntoIterator<Item = i64>) -> Result<(), TxtodoError> {
        let nums: Vec<i64> = numbers.into_iter().collect();
        let flat_len = self.list().len();
        let mut indices: Vec<usize> = Vec::new();
        for n in &nums {
            indices.push(resolve_index(*n, flat_len)?);
        }
        indices.sort_unstable();
        indices.dedup();
        for &idx in &indices {
            if let Some(t) = Self::nth_flat_mut(&mut self.tasks, idx) {
                t.completed = false;
                t.completion_date = None;
            }
        }
        self.save_if_needed()
    }

    pub fn remove(&mut self, numbers: impl IntoIterator<Item = i64>) -> Result<(), TxtodoError> {
        let nums: Vec<i64> = numbers.into_iter().collect();
        let flat_len = self.list().len();
        let mut indices: Vec<usize> = Vec::new();
        for n in &nums {
            indices.push(resolve_index(*n, flat_len)?);
        }
        indices.sort_unstable();
        indices.dedup();
        // Remove in reverse so earlier indices remain valid.
        for &idx in indices.iter().rev() {
            remove_nth_flat(&mut self.tasks, idx);
        }
        self.save_if_needed()
    }

    pub fn update(&mut self, index: i64, patch: TaskPatch) -> Result<(), TxtodoError> {
        let flat_len = self.list().len();
        let idx = resolve_index(index, flat_len)?;
        if let Some(t) = Self::nth_flat_mut(&mut self.tasks, idx) {
            patch.apply(t);
        }
        self.save_if_needed()
    }

    pub fn update_many(
        &mut self,
        updates: impl IntoIterator<Item = (i64, TaskPatch)>,
    ) -> Result<(), TxtodoError> {
        for (idx, patch) in updates {
            self.update(idx, patch)?;
        }
        Ok(())
    }

    fn add_task_with_subtasks(
        &mut self,
        task: Task,
        is_first_in_batch: bool,
    ) -> Result<(), TxtodoError> {
        self.add_tasks_batch(vec![task], is_first_in_batch)
    }

    fn add_tasks_batch(&mut self, tasks: Vec<Task>, _is_first: bool) -> Result<(), TxtodoError> {
        for task in tasks {
            if task.indent_level == 0 {
                self.tasks.push(task);
            } else {
                if let Some(p_idx) = find_parent_in_flat(&self.tasks, task.indent_level) {
                    let mut task = task;
                    let parent: *mut Task = &mut self.tasks[p_idx];
                    unsafe {
                        task.parent = Some(std::ptr::NonNull::new_unchecked(parent));
                    }
                    unsafe {
                        (*parent).subtasks.push(task);
                    }
                } else {
                    self.tasks.push(task);
                }
            }
        }
        Ok(())
    }

    fn flat_len(&self) -> usize {
        self.list().len()
    }

    fn save_if_needed(&self) -> Result<(), TxtodoError> {
        if self.auto_save
            && let Some(ref path) = self.file_path
        {
            self.save(Some(path))?;
        }
        Ok(())
    }

    /// Walk the tree depth-first (pre-order) and return the n-th task as a
    /// mutable reference, or `None` if `target` is out of bounds.
    fn nth_flat_mut(tasks: &mut [Task], target: usize) -> Option<&mut Task> {
        let mut counter = 0usize;
        nth_flat_mut_inner(tasks, target, &mut counter)
    }
}

fn resolve_path(current: Option<&str>, given: Option<&str>) -> Result<String, TxtodoError> {
    if let Some(p) = given {
        return Ok(p.to_string());
    }
    if let Some(p) = current {
        return Ok(p.to_string());
    }
    Err(TxtodoError::Generic("No file path specified".to_string()))
}

pub fn resolve_index(original: i64, len: usize) -> Result<usize, TxtodoError> {
    if original < 0 {
        let wrapped = (len as i64) + original;
        if wrapped < 0 || wrapped >= len as i64 {
            return Err(out_of_bounds(original, len));
        }
        Ok(wrapped as usize)
    } else {
        if (original as usize) >= len {
            return Err(out_of_bounds(original, len));
        }
        Ok(original as usize)
    }
}

fn out_of_bounds(original: i64, len: usize) -> TxtodoError {
    let msg = if len == 0 {
        format!("Index out of bounds: {original}. Valid range is 0..-1 or -0..-1")
    } else {
        let max = len - 1;
        format!("Index out of bounds: {original}. Valid range is 0..{max} or -{len}..-1")
    };
    TxtodoError::Generic(msg)
}

fn flatten_into<'a>(out: &mut Vec<&'a Task>, task: &'a Task) {
    out.push(task);
    for c in &task.subtasks {
        flatten_into(out, c);
    }
}

fn filter_tree(task: &Task, filter: &crate::TaskFilter) -> Option<Task> {
    if !filter(task) {
        return None;
    }
    let mut clone = task.clone();
    clone.subtasks = Vec::new();
    for sub in &task.subtasks {
        if let Some(c) = filter_tree(sub, filter) {
            clone.subtasks.push(c);
        }
    }
    Some(clone)
}

fn sort_recursive(tasks: &mut [Task], sorter: &crate::TaskSorter) {
    for t in tasks.iter_mut() {
        sort_recursive(&mut t.subtasks, sorter);
    }
    tasks.sort_by(|a, b| sorter(a, b));
}

fn find_parent_in_flat(tasks: &[Task], indent: usize) -> Option<usize> {
    (0..tasks.len())
        .rev()
        .find(|&i| tasks[i].indent_level < indent)
}

fn nth_flat_mut_inner<'a>(
    tasks: &'a mut [Task],
    target: usize,
    counter: &mut usize,
) -> Option<&'a mut Task> {
    for t in tasks.iter_mut() {
        if *counter == target {
            return Some(t);
        }
        *counter += 1;
        if let Some(found) = nth_flat_mut_inner(&mut t.subtasks, target, counter) {
            return Some(found);
        }
    }
    None
}

/// Remove the n-th task (pre-order) from the tree.
fn remove_nth_flat(tasks: &mut Vec<Task>, target: usize) -> bool {
    let mut counter = 0usize;
    remove_nth_flat_inner(tasks, target, &mut counter)
}

fn remove_nth_flat_inner(tasks: &mut Vec<Task>, target: usize, counter: &mut usize) -> bool {
    for i in 0..tasks.len() {
        if *counter == target {
            tasks.remove(i);
            return true;
        }
        *counter += 1;
        if remove_nth_flat_inner(&mut tasks[i].subtasks, target, counter) {
            return true;
        }
    }
    false
}
