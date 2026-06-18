use crate::task::{ExtensionValue, Task};
use std::rc::Rc;

/// A reference-counted predicate used to filter [`Task`]s.
///
/// See [`TaskFilters`] for common filter constructors.
pub type TaskFilter = Rc<dyn Fn(&Task) -> bool>;

/// Utility struct providing factory methods for creating task filter functions.
///
/// All methods return a [`TaskFilter`] (a boxed closure) that can be used with
/// task listing functions to select tasks matching specific criteria.
///
/// # Examples
///
/// ```
/// # use txtodo::*;
/// fn main() -> Result<(), TxtodoError> {
///     let filter = TaskFilters::completed();
///     let ctx_filter = TaskFilters::by_context("work");
///     Ok(())
/// }
/// ```
pub struct TaskFilters;

impl TaskFilters {
    fn box_f<F: Fn(&Task) -> bool + 'static>(f: F) -> TaskFilter {
        Rc::new(f) as TaskFilter
    }

    /// Creates a filter matching tasks that have the given context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_context("work");
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_context(ctx: &str) -> TaskFilter {
        let ctx = ctx.to_string();
        Self::box_f(move |t: &Task| t.contexts.contains(&ctx))
    }

    /// Creates a filter matching tasks that have any of the given contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_contexts(&["work", "office"]);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_contexts(ctxs: &[&str]) -> TaskFilter {
        let owned: Vec<String> = ctxs.iter().map(|s| s.to_string()).collect();
        Self::box_f(move |t: &Task| owned.iter().any(|c| t.contexts.contains(c)))
    }

    /// Creates a filter matching tasks that belong to the given project.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_project("txtodo");
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_project(proj: &str) -> TaskFilter {
        let proj = proj.to_string();
        Self::box_f(move |t: &Task| t.projects.contains(&proj))
    }

    /// Creates a filter matching tasks that belong to any of the given projects.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_projects(&["txtodo", "backend"]);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_projects(projs: &[&str]) -> TaskFilter {
        let owned: Vec<String> = projs.iter().map(|s| s.to_string()).collect();
        Self::box_f(move |t: &Task| owned.iter().any(|p| t.projects.contains(p)))
    }

    /// Creates a filter matching tasks with exactly the given priority.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_priority(Priority('A'));
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_priority(p: crate::task::Priority) -> TaskFilter {
        Self::box_f(move |t: &Task| t.priority == Some(p))
    }

    /// Creates a filter matching tasks with any of the given priorities.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_priorities(&[
    ///         Priority('A'),
    ///         Priority('B'),
    ///     ]);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_priorities(ps: &[crate::task::Priority]) -> TaskFilter {
        let ps = ps.to_vec();
        Self::box_f(move |t: &Task| match t.priority {
            Some(p) => ps.contains(&p),
            None => false,
        })
    }

    /// Creates a filter matching tasks by their completion status.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_completion_status(true);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_completion_status(done: bool) -> TaskFilter {
        Self::box_f(move |t: &Task| t.completed == done)
    }

    /// Creates a filter matching tasks by a custom extension field.
    ///
    /// If `value` is `Some`, matches tasks where the field equals the value.
    /// If `value` is `None`, matches tasks where the field key exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_extension_field("due", None);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_extension_field(key: &str, value: Option<&ExtensionValue>) -> TaskFilter {
        let key = key.to_string();
        match value {
            Some(v) => {
                let v = v.clone();
                Self::box_f(move |t: &Task| match t.extensions.get(&key) {
                    Some(tv) => tv.equals(&v),
                    None => false,
                })
            }
            None => Self::box_f(move |t: &Task| t.extensions.contains_key(&key)),
        }
    }

    /// Creates a filter matching tasks where all given extension fields match.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::by_extension_fields(&[]);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn by_extension_fields(kvs: &[(String, ExtensionValue)]) -> TaskFilter {
        let kvs: Vec<(String, ExtensionValue)> = kvs.to_vec();
        Self::box_f(move |t: &Task| {
            for (k, v) in &kvs {
                match t.extensions.get(k) {
                    Some(tv) if tv.equals(v) => continue,
                    _ => return false,
                }
            }
            true
        })
    }

    /// Creates a filter matching completed tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::completed();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn completed() -> TaskFilter {
        Self::by_completion_status(true)
    }

    /// Creates a filter matching incomplete (not yet done) tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::incomplete();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn incomplete() -> TaskFilter {
        Self::by_completion_status(false)
    }

    /// Creates a filter matching tasks that have a priority assigned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::has_priority();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn has_priority() -> TaskFilter {
        Self::box_f(|t: &Task| t.priority.is_some())
    }

    /// Creates a filter matching tasks that have no priority assigned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::no_priority();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn no_priority() -> TaskFilter {
        Self::box_f(|t: &Task| t.priority.is_none())
    }

    /// Creates a filter matching tasks that have at least one context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::has_context();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn has_context() -> TaskFilter {
        Self::box_f(|t: &Task| !t.contexts.is_empty())
    }

    /// Creates a filter matching tasks that have no contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::no_context();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn no_context() -> TaskFilter {
        Self::box_f(|t: &Task| t.contexts.is_empty())
    }

    /// Creates a filter matching tasks that belong to at least one project.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::has_project();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn has_project() -> TaskFilter {
        Self::box_f(|t: &Task| !t.projects.is_empty())
    }

    /// Creates a filter matching tasks that belong to no projects.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let filter = TaskFilters::no_project();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn no_project() -> TaskFilter {
        Self::box_f(|t: &Task| t.projects.is_empty())
    }

    /// Creates a filter matching tasks created after the given date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let d = time::Date::from_calendar_date(2024, time::Month::January, 1)
    ///         .map_err(|e| TxtodoError::Date { message: e.to_string(), date_str: None })?;
    ///     let filter = TaskFilters::created_after(d);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn created_after(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.creation_date {
            Some(td) => td > d,
            None => false,
        })
    }

    /// Creates a filter matching tasks created before the given date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let d = time::Date::from_calendar_date(2024, time::Month::January, 1)
    ///         .map_err(|e| TxtodoError::Date { message: e.to_string(), date_str: None })?;
    ///     let filter = TaskFilters::created_before(d);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn created_before(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.creation_date {
            Some(td) => td < d,
            None => false,
        })
    }

    /// Creates a filter matching tasks created on the given date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let d = time::Date::from_calendar_date(2024, time::Month::January, 1)
    ///         .map_err(|e| TxtodoError::Date { message: e.to_string(), date_str: None })?;
    ///     let filter = TaskFilters::created_on(d);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn created_on(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.creation_date {
            Some(td) => td == d,
            None => false,
        })
    }

    /// Creates a filter matching tasks completed after the given date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let d = time::Date::from_calendar_date(2024, time::Month::January, 1)
    ///         .map_err(|e| TxtodoError::Date { message: e.to_string(), date_str: None })?;
    ///     let filter = TaskFilters::completed_after(d);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn completed_after(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.completion_date {
            Some(td) => td > d,
            None => false,
        })
    }

    /// Creates a filter matching tasks completed before the given date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let d = time::Date::from_calendar_date(2024, time::Month::January, 1)
    ///         .map_err(|e| TxtodoError::Date { message: e.to_string(), date_str: None })?;
    ///     let filter = TaskFilters::completed_before(d);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn completed_before(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.completion_date {
            Some(td) => td < d,
            None => false,
        })
    }

    /// Creates a filter matching tasks completed on the given date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let d = time::Date::from_calendar_date(2024, time::Month::January, 1)
    ///         .map_err(|e| TxtodoError::Date { message: e.to_string(), date_str: None })?;
    ///     let filter = TaskFilters::completed_on(d);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn completed_on(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.completion_date {
            Some(td) => td == d,
            None => false,
        })
    }

    /// Creates a composite filter that matches only when all given filters match (logical AND).
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let f1 = TaskFilters::by_context("work");
    ///     let f2 = TaskFilters::completed();
    ///     let filter = TaskFilters::and(&[&f1, &f2]);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn and(filters: &[&TaskFilter]) -> TaskFilter {
        let owned: Vec<TaskFilter> = filters.iter().map(|f| (*f).clone()).collect();
        Self::box_f(move |t: &Task| owned.iter().all(|f| f(t)))
    }

    /// Creates a composite filter that matches when any of the given filters match (logical OR).
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let f1 = TaskFilters::by_context("work");
    ///     let f2 = TaskFilters::by_context("home");
    ///     let filter = TaskFilters::or(&[&f1, &f2]);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn or(filters: &[&TaskFilter]) -> TaskFilter {
        let owned: Vec<TaskFilter> = filters.iter().map(|f| (*f).clone()).collect();
        Self::box_f(move |t: &Task| owned.iter().any(|f| f(t)))
    }

    /// Creates a filter that inverts the given filter (logical NOT).
    ///
    /// # Examples
    ///
    /// ```
    /// # use txtodo::*;
    /// fn main() -> Result<(), TxtodoError> {
    ///     let f = TaskFilters::completed();
    ///     let filter = TaskFilters::not(&f);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn not(filter: &TaskFilter) -> TaskFilter {
        let filter = filter.clone();
        Self::box_f(move |t: &Task| !filter(t))
    }
}
