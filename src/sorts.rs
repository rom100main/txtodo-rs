use crate::task::Task;
use std::cmp::Ordering;
use std::rc::Rc;

/// Direction for sorting tasks.
///
/// # Examples
///
/// ```
/// # use txtodo::*;
/// fn main() -> Result<(), TxtodoError> {
///     let sorter = TaskSorts::by_description(SortDirection::Asc);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Sort in ascending order (A to Z, oldest to newest, lowest to highest).
    Asc,
    /// Sort in descending order (Z to A, newest to oldest, highest to lowest).
    Desc,
}

/// A reference-counted comparator used to sort [`Task`]s.
///
/// See [`TaskSorts`] for common sorter constructors.
pub type TaskSorter = Rc<dyn Fn(&Task, &Task) -> Ordering>;

/// Utility struct providing factory methods for creating task sort functions.
///
/// All methods return a [`TaskSorter`] (a boxed closure)
/// that can be used with task listing functions to order tasks.
/// Use [`TaskSorts::composite`] or [`TaskSorts::then`] to combine multiple sorters for multi-level sorting.
///
/// # Examples
///
/// ```
/// # use txtodo::*;
/// fn main() -> Result<(), TxtodoError> {
///     let sort = TaskSorts::by_priority(SortDirection::Asc);
///     let multi = TaskSorts::then(&sort, &TaskSorts::by_description(SortDirection::Asc));
///     Ok(())
/// }
/// ```
pub struct TaskSorts;

fn box_s<F: Fn(&Task, &Task) -> Ordering + 'static>(f: F) -> TaskSorter {
    Rc::new(f) as TaskSorter
}

fn reverse(sorter: TaskSorter, dir: SortDirection) -> TaskSorter {
    match dir {
        SortDirection::Asc => sorter,
        SortDirection::Desc => {
            let s = sorter;
            box_s(move |a, b| s(b, a))
        }
    }
}

impl TaskSorts {
    /// Sorts tasks alphabetically by their context tags.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_context(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_context(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| {
            a.contexts
                .join(",")
                .to_lowercase()
                .cmp(&b.contexts.join(",").to_lowercase())
        });
        reverse(s, dir)
    }

    /// Sorts tasks alphabetically by their project tags.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_project(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_project(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| {
            a.projects
                .join(",")
                .to_lowercase()
                .cmp(&b.projects.join(",").to_lowercase())
        });
        reverse(s, dir)
    }

    /// Sorts tasks by creation date. Tasks without a creation date are sorted last.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_date_created(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_date_created(dir: SortDirection) -> TaskSorter {
        let s = box_s(
            |a: &Task, b: &Task| match (a.creation_date, b.creation_date) {
                (None, None) => Ordering::Equal,
                (None, _) => Ordering::Greater,
                (_, None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(&b),
            },
        );
        reverse(s, dir)
    }

    /// Sorts tasks by completion date. Tasks without a completion date are sorted last.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_date_completed(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_date_completed(dir: SortDirection) -> TaskSorter {
        let s = box_s(
            |a: &Task, b: &Task| match (a.completion_date, b.completion_date) {
                (None, None) => Ordering::Equal,
                (None, _) => Ordering::Greater,
                (_, None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(&b),
            },
        );
        reverse(s, dir)
    }

    /// Sorts tasks by priority level. Tasks without priority are sorted last.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_priority(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_priority(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| match (a.priority, b.priority) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Greater,
            (_, None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(&b),
        });
        reverse(s, dir)
    }

    /// Sorts tasks by a custom extension field value. Tasks without the field are sorted last.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_extension_field("due", SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_extension_field(key: &str, dir: SortDirection) -> TaskSorter {
        let key = key.to_string();
        let s = box_s(move |a: &Task, b: &Task| {
            match (a.extensions.get(&key), b.extensions.get(&key)) {
                (None, None) => Ordering::Equal,
                (None, _) => Ordering::Greater,
                (_, None) => Ordering::Less,
                (Some(a), Some(b)) => a.compare_to(b),
            }
        });
        reverse(s, dir)
    }

    /// Sorts tasks alphabetically by their description text.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_description(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_description(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| {
            a.description
                .to_lowercase()
                .cmp(&b.description.to_lowercase())
        });
        reverse(s, dir)
    }

    /// Sorts tasks by completion status. Incomplete tasks sort before completed ones in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_completion_status(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_completion_status(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| (a.completed as i32).cmp(&(b.completed as i32)));
        reverse(s, dir)
    }

    /// Sorts tasks by their indentation level.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_indent_level(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_indent_level(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.indent_level.cmp(&b.indent_level));
        reverse(s, dir)
    }

    /// Sorts tasks alphabetically by their raw todo.txt line content.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_raw(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_raw(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.raw.to_lowercase().cmp(&b.raw.to_lowercase()));
        reverse(s, dir)
    }

    /// Sorts tasks by the number of context tags.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_context_count(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_context_count(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.contexts.len().cmp(&b.contexts.len()));
        reverse(s, dir)
    }

    /// Sorts tasks by the number of project tags.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_project_count(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_project_count(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.projects.len().cmp(&b.projects.len()));
        reverse(s, dir)
    }

    /// Sorts tasks by the number of subtasks.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_subtask_count(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_subtask_count(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.subtasks.len().cmp(&b.subtasks.len()));
        reverse(s, dir)
    }

    /// Sorts tasks by the number of extension fields.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::by_extension_count(SortDirection::Asc);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_extension_count(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.extensions.len().cmp(&b.extensions.len()));
        reverse(s, dir)
    }

    /// Creates a multi-level sorter from a slice of sorters.
    ///
    /// Sorters are applied in order: if the first sorter returns `Equal`,
    /// the second is used, and so on. This allows grouping by one criterion
    /// then breaking ties with additional criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let by_pri = TaskSorts::by_priority(SortDirection::Asc);
    ///     let by_desc = TaskSorts::by_description(SortDirection::Asc);
    ///     let sort = TaskSorts::composite(&[&by_pri, &by_desc]);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn composite(sorters: &[&TaskSorter]) -> TaskSorter {
        let owned: Vec<TaskSorter> = sorters.iter().map(|s| (*s).clone()).collect();
        box_s(move |a: &Task, b: &Task| {
            for s in &owned {
                let r = s(a, b);
                if r != Ordering::Equal {
                    return r;
                }
            }
            Ordering::Equal
        })
    }

    /// Creates a two-level sorter: uses `primary` first, then `secondary` to break ties.
    ///
    /// This is a convenience wrapper around [`TaskSorts::composite`]
    /// for the common case of combining exactly two sorters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let sort = TaskSorts::then(
    ///         &TaskSorts::by_priority(SortDirection::Asc),
    ///         &TaskSorts::by_description(SortDirection::Asc),
    ///     );
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn then(primary: &TaskSorter, secondary: &TaskSorter) -> TaskSorter {
        Self::composite(&[primary, secondary])
    }
}
