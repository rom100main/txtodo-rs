use crate::TaskFilter;
use crate::task::{ExtensionValue, Task};
use std::rc::Rc;

pub struct TaskFilters;

impl TaskFilters {
    fn box_f<F: Fn(&Task) -> bool + 'static>(f: F) -> TaskFilter {
        Rc::new(f) as TaskFilter
    }

    #[must_use]
    pub fn by_context(ctx: &str) -> TaskFilter {
        let ctx = ctx.to_string();
        Self::box_f(move |t: &Task| t.contexts.contains(&ctx))
    }

    #[must_use]
    pub fn by_contexts(ctxs: &[&str]) -> TaskFilter {
        let owned: Vec<String> = ctxs.iter().map(|s| s.to_string()).collect();
        Self::box_f(move |t: &Task| owned.iter().any(|c| t.contexts.contains(c)))
    }

    #[must_use]
    pub fn by_project(proj: &str) -> TaskFilter {
        let proj = proj.to_string();
        Self::box_f(move |t: &Task| t.projects.contains(&proj))
    }

    #[must_use]
    pub fn by_projects(projs: &[&str]) -> TaskFilter {
        let owned: Vec<String> = projs.iter().map(|s| s.to_string()).collect();
        Self::box_f(move |t: &Task| owned.iter().any(|p| t.projects.contains(p)))
    }

    #[must_use]
    pub fn by_priority(p: crate::task::Priority) -> TaskFilter {
        Self::box_f(move |t: &Task| t.priority == Some(p))
    }

    #[must_use]
    pub fn by_priorities(ps: &[crate::task::Priority]) -> TaskFilter {
        let ps = ps.to_vec();
        Self::box_f(move |t: &Task| match t.priority {
            Some(p) => ps.contains(&p),
            None => false,
        })
    }

    #[must_use]
    pub fn by_completion_status(done: bool) -> TaskFilter {
        Self::box_f(move |t: &Task| t.completed == done)
    }

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

    #[must_use]
    pub fn completed() -> TaskFilter {
        Self::by_completion_status(true)
    }

    #[must_use]
    pub fn incomplete() -> TaskFilter {
        Self::by_completion_status(false)
    }

    #[must_use]
    pub fn has_priority() -> TaskFilter {
        Self::box_f(|t: &Task| t.priority.is_some())
    }

    #[must_use]
    pub fn no_priority() -> TaskFilter {
        Self::box_f(|t: &Task| t.priority.is_none())
    }

    #[must_use]
    pub fn has_context() -> TaskFilter {
        Self::box_f(|t: &Task| !t.contexts.is_empty())
    }

    #[must_use]
    pub fn no_context() -> TaskFilter {
        Self::box_f(|t: &Task| t.contexts.is_empty())
    }

    #[must_use]
    pub fn has_project() -> TaskFilter {
        Self::box_f(|t: &Task| !t.projects.is_empty())
    }

    #[must_use]
    pub fn no_project() -> TaskFilter {
        Self::box_f(|t: &Task| t.projects.is_empty())
    }

    #[must_use]
    pub fn created_after(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.creation_date {
            Some(td) => td > d,
            None => false,
        })
    }

    #[must_use]
    pub fn created_before(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.creation_date {
            Some(td) => td < d,
            None => false,
        })
    }

    #[must_use]
    pub fn created_on(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.creation_date {
            Some(td) => td == d,
            None => false,
        })
    }

    #[must_use]
    pub fn completed_after(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.completion_date {
            Some(td) => td > d,
            None => false,
        })
    }

    #[must_use]
    pub fn completed_before(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.completion_date {
            Some(td) => td < d,
            None => false,
        })
    }

    #[must_use]
    pub fn completed_on(d: time::Date) -> TaskFilter {
        Self::box_f(move |t: &Task| match t.completion_date {
            Some(td) => td == d,
            None => false,
        })
    }

    #[must_use]
    pub fn and(filters: &[&TaskFilter]) -> TaskFilter {
        let owned: Vec<TaskFilter> = filters.iter().map(|f| (*f).clone()).collect();
        Self::box_f(move |t: &Task| owned.iter().all(|f| f(t)))
    }

    #[must_use]
    pub fn or(filters: &[&TaskFilter]) -> TaskFilter {
        let owned: Vec<TaskFilter> = filters.iter().map(|f| (*f).clone()).collect();
        Self::box_f(move |t: &Task| owned.iter().any(|f| f(t)))
    }

    #[must_use]
    pub fn not(filter: &TaskFilter) -> TaskFilter {
        let filter = filter.clone();
        Self::box_f(move |t: &Task| !filter(t))
    }
}
