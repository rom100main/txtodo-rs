use crate::error::TxtodoError;
use crate::extension::ExtensionHandler;
use crate::options::TodoOptions;
use crate::parser::TodoTxtParser;
use crate::serializer::TodoTxtSerializer;
use crate::task::Task;
use crate::task::TaskPatch;
use std::fs;

/// Input accepted by [`TodoTxt::add`] and related methods.
///
/// This enum allows callers to pass either a raw todo.txt line (as `&str` or `String`) or a pre-parsed [`Task`].
/// Conversion from `&str` and `String` is provided via [`From`] impls so callers can write:
///
/// ```
/// use txtodo::*;
/// let options = TodoOptions::default();
/// let mut todo = TodoTxt::new(options)?;
/// todo.add("(A) Buy milk @home")?;      // from &str
/// # Ok::<(), TxtodoError>(())
/// ```
pub enum TaskInput {
    /// A raw todo.txt line that still needs to be parsed.
    Line(String),
    /// An already-parsed [`Task`] that can be added directly.
    Task(Task),
}

/// Converts a `&str` into [`TaskInput::Line`].
impl From<&str> for TaskInput {
    fn from(s: &str) -> Self {
        TaskInput::Line(s.to_string())
    }
}

/// Converts a `String` into [`TaskInput::Line`].
impl From<String> for TaskInput {
    fn from(s: String) -> Self {
        TaskInput::Line(s)
    }
}

/// Converts a [`Task`] into [`TaskInput::Task`].
impl From<Task> for TaskInput {
    fn from(t: Task) -> Self {
        TaskInput::Task(t)
    }
}

/// The main entry point for managing todo.txt tasks.
///
/// `TodoTxt` holds a parsed task list and provides methods to load, save, query, add, update, and remove tasks.
/// When [`TodoOptions::auto_save`] is enabled, every mutating operation persists changes to disk automatically.
///
/// # Examples
///
/// ```no_run
/// use txtodo::*;
///
/// fn main() -> Result<(), TxtodoError> {
///     let options = TodoOptions {
///         file_path: Some("todo.txt".into()),
///         ..Default::default()
///     };
///     let mut todo = TodoTxt::new(options)?;
///     todo.load(None)?;
///     todo.add("(A) Write docs @work")?;
///     let tasks = todo.list();
///     println!("{} tasks loaded", tasks.len());
///     Ok(())
/// }
/// ```
pub struct TodoTxt {
    /// The current list of top-level tasks. Subtasks are nested inside each [`Task`].
    pub tasks: Vec<Task>,
    pub(crate) file_path: Option<String>,
    pub(crate) auto_save: bool,
    pub(crate) handle_subtasks: bool,
    pub(crate) parser: TodoTxtParser,
    pub(crate) serializer: TodoTxtSerializer,
    pub(crate) handler: ExtensionHandler,
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
    /// Creates a new `TodoTxt` instance from the given [`TodoOptions`].
    ///
    /// The returned instance has an empty task list.
    /// Call [`load`](Self::load) to populate it from a file.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if the configured extensions fail to initialise.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let todo = TodoTxt::new(options)?;
    /// assert!(todo.list().is_empty());
    /// # Ok::<(), TxtodoError>(())
    /// ```
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

    /// Loads tasks from a todo.txt file.
    ///
    /// If `file_path` is `Some`, that path is used;
    /// otherwise the path configured at construction time is used.
    /// When no path is available an error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if the file cannot be read or the content cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use txtodo::*;
    /// let options = TodoOptions {
    ///     file_path: Some("todo.txt".into()),
    ///     ..Default::default()
    /// };
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.load(None)?;
    /// # Ok::<(), TxtodoError>(())
    /// ```
    pub fn load(&mut self, file_path: Option<&str>) -> Result<(), TxtodoError> {
        let path_str = resolve_path(self.file_path.as_deref(), file_path)?;
        let content = fs::read_to_string(&path_str)?;
        self.tasks = self.parser.parse_file(&content)?;
        if self.file_path.is_none() {
            self.file_path = Some(path_str);
        }
        Ok(())
    }

    /// Saves the current task list to a todo.txt file.
    ///
    /// If `file_path` is `Some`, that path is used;
    /// otherwise the path configured at construction time is used.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if no file path is available,
    /// the tasks cannot be serialised, or the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use txtodo::*;
    /// let options = TodoOptions {
    ///     file_path: Some("todo.txt".into()),
    ///     ..Default::default()
    /// };
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.load(None)?;
    /// todo.save(None)?;
    /// # Ok::<(), TxtodoError>(())
    /// ```
    pub fn save(&self, file_path: Option<&str>) -> Result<(), TxtodoError> {
        let path_str = resolve_path(self.file_path.as_deref(), file_path)?;
        let content = self.serializer.serialize_tasks(&self.tasks)?;
        fs::write(&path_str, content)?;
        Ok(())
    }

    /// Enables or disables automatic saving after every mutation.
    ///
    /// When enabled, any method that modifies the task list will automatically call [`save`](Self::save) if a file path is configured.
    pub fn set_auto_save(&mut self, on: bool) {
        self.auto_save = on;
    }

    /// Returns a shared reference to the [`ExtensionHandler`].
    #[must_use]
    pub fn extension_handler(&self) -> &ExtensionHandler {
        &self.handler
    }

    /// Returns a mutable reference to the [`ExtensionHandler`].
    #[must_use]
    pub fn extension_handler_mut(&mut self) -> &mut ExtensionHandler {
        &mut self.handler
    }

    /// Returns all tasks (including subtasks) as a flat list.
    ///
    /// The list is in pre-order depth-first order: a parent appears before its subtasks.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("Buy groceries")?;
    /// assert_eq!(todo.list().len(), 1);
    /// # Ok::<(), TxtodoError>(())
    /// ```
    #[must_use]
    pub fn list(&self) -> Vec<&Task> {
        self.list_filtered(None, None)
    }

    /// Returns all tasks as a flat list, optionally filtered and sorted.
    ///
    /// When `filter` is `Some`, only tasks matching the predicate are included.
    /// When `sorter` is `Some`, the resulting list is sorted according to the comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// use std::rc::Rc;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("(A) Urgent task")?;
    /// todo.add("(B) Low priority")?;
    /// let filter: TaskFilter = Rc::new(|t: &Task| t.priority == Some(Priority('A')));
    /// let high: Vec<&Task> = todo.list_filtered(Some(&filter), None);
    /// assert_eq!(high.len(), 1);
    /// # Ok::<(), TxtodoError>(())
    /// ```
    #[must_use]
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

    /// Returns a clone of the task tree, keeping only tasks that match the filter.
    /// Subtasks of a matching parent are preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// use std::rc::Rc;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("(A) Important")?;
    /// todo.add("(B) Normal")?;
    /// let filter: TaskFilter = Rc::new(|t: &Task| t.priority == Some(Priority('A')));
    /// let filtered = todo.filter(&filter);
    /// assert_eq!(filtered.len(), 1);
    /// # Ok::<(), TxtodoError>(())
    /// ```
    #[must_use]
    pub fn filter(&self, filter: &crate::TaskFilter) -> Vec<Task> {
        let mut out = Vec::new();
        for t in &self.tasks {
            if let Some(clone) = filter_tree(t, filter) {
                out.push(clone);
            }
        }
        out
    }

    /// Sorts tasks in-place using the given comparator.
    ///
    /// Sorting is applied recursively: each level of the task tree is sorted independently.
    pub fn sort(&mut self, sorter: &crate::TaskSorter) {
        sort_recursive(&mut self.tasks, sorter);
    }

    /// Adds a single task to the list.
    ///
    /// Accepts anything that implements `Into<TaskInput>`, including `&str`, `String`, and [`Task`].
    /// If subtask handling is enabled, the task is inserted into the tree at the correct position
    /// based on its indent level.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if the input cannot be parsed or auto-save is
    /// enabled and the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("(A) Buy milk @home")?;
    /// assert_eq!(todo.list().len(), 1);
    /// # Ok::<(), TxtodoError>(())
    /// ```
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

    /// Adds multiple tasks to the list in a single operation.
    ///
    /// Each item in the iterator can be a `&str`, `String`, or [`Task`].
    /// When subtask handling is enabled, the entire batch is parsed and
    /// inserted into the tree atomically.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if any input cannot be parsed
    /// or auto-save is enabled and the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add_many(["Task 1", "Task 2", "Task 3"])?;
    /// assert_eq!(todo.list().len(), 3);
    /// # Ok::<(), TxtodoError>(())
    /// ```
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

    /// Inserts a task at the given position in the flat task list.
    ///
    /// Negative indices count from the end of the list (e.g., `-1` is the last position).
    /// If the index is beyond the end, the task is appended.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if the input cannot be parsed
    /// or auto-save is enabled and the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("First")?;
    /// todo.add("Third")?;
    /// todo.insert(1, "Second")?;
    /// let descs: Vec<&str> = todo.list().iter().map(|t| t.description.as_str()).collect();
    /// assert_eq!(descs, vec!["First", "Second", "Third"]);
    /// # Ok::<(), TxtodoError>(())
    /// ```
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
            let insert_at = (resolved as usize).min(raws.len());
            raws.insert(insert_at, task.raw.clone());
            self.tasks = self.parser.parse_file(&raws.join("\n"))?;
        } else {
            let insert_at = (resolved as usize).min(self.tasks.len());
            self.tasks.insert(insert_at, task);
        }
        self.save_if_needed()
    }

    /// Marks tasks as completed by their 1-based indices.
    ///
    /// Negative indices count from the end (e.g., `-1` is the last task).
    /// Duplicate indices are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if any index is out of bounds
    /// or auto-save is enabled and the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("Buy groceries")?;
    /// todo.mark([0_i64])?;
    /// assert!(todo.list()[0].completed);
    /// # Ok::<(), TxtodoError>(())
    /// ```
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

    /// Unmarks tasks (sets them as not completed) by their indices.
    ///
    /// Negative indices count from the end (e.g., `-1` is the last task).
    /// Duplicate indices are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if any index is out of bounds
    /// or auto-save is enabled and the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("Buy groceries")?;
    /// todo.mark([0_i64])?;
    /// assert!(todo.list()[0].completed);
    /// todo.unmark([0_i64])?;
    /// assert!(!todo.list()[0].completed);
    /// # Ok::<(), TxtodoError>(())
    /// ```
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

    /// Removes tasks by their indices.
    ///
    /// Negative indices count from the end (e.g., `-1` is the last task).
    /// Duplicate indices are silently ignored.
    /// Removal proceeds in reverse order so that earlier indices remain valid.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if any index is out of bounds
    /// or auto-save is enabled and the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("Task 1")?;
    /// todo.add("Task 2")?;
    /// todo.remove([1_i64])?;
    /// assert_eq!(todo.list().len(), 1);
    /// # Ok::<(), TxtodoError>(())
    /// ```
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

    /// Updates a single task at the given index using a [`TaskPatch`].
    ///
    /// Negative indices count from the end (e.g., `-1` is the last task).
    /// Only the fields present in the patch are modified.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if the index is out of bounds
    /// or auto-save is enabled and the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("(A) Buy milk")?;
    /// let patch = TaskPatch {
    ///     priority: Some(Some(Priority('B'))),
    ///     ..Default::default()
    /// };
    /// todo.update(0, patch)?;
    /// assert_eq!(todo.list()[0].priority, Some(Priority('B')));
    /// # Ok::<(), TxtodoError>(())
    /// ```
    pub fn update(&mut self, index: i64, patch: TaskPatch) -> Result<(), TxtodoError> {
        let flat_len = self.list().len();
        let idx = resolve_index(index, flat_len)?;
        if let Some(t) = Self::nth_flat_mut(&mut self.tasks, idx) {
            patch.apply(t);
        }
        self.save_if_needed()
    }

    /// Updates multiple tasks in a single operation.
    ///
    /// Each item in the iterator is a `(index, patch)` pair.
    /// Negative indices count from the end (e.g., `-1` is the last task).
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError`] if any index is out of bounds, the patch cannot be applied,
    /// or auto-save is enabled and the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use txtodo::*;
    /// let options = TodoOptions::default();
    /// let mut todo = TodoTxt::new(options)?;
    /// todo.add("Task 1")?;
    /// todo.add("Task 2")?;
    /// let updates = vec![
    ///     (0, TaskPatch { priority: Some(Some(Priority('A'))), ..Default::default() }),
    ///     (1, TaskPatch { priority: Some(Some(Priority('B'))), ..Default::default() }),
    /// ];
    /// todo.update_many(updates)?;
    /// # Ok::<(), TxtodoError>(())
    /// ```
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

/// Converts a signed index into a valid `usize` within the given range.
///
/// Negative values wrap from the end (`-1` = last element)
/// and are validated against `len`.
/// `0` and `-0` are treated specially: when `len` is `0` they succeed (wrapping to `0`),
/// otherwise `-0` times out as out of bounds.
///
/// # Errors
///
/// Returns [`TxtodoError::Generic`] if the resolved index does not
/// fall within the range for `len`.
///
/// # Examples
///
/// ```
/// # use txtodo::resolve_index;
/// assert_eq!(resolve_index(2, 5).unwrap(), 2);
/// assert_eq!(resolve_index(-1, 5).unwrap(), 4);
/// assert_eq!(resolve_index(-5, 5).unwrap(), 0);
/// assert!(resolve_index(5, 5).is_err());
/// ```
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
