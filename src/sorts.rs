use crate::task::Task;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub(crate) type TaskSorter = Rc<dyn Fn(&Task, &Task) -> Ordering>;

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

    #[must_use]
    pub fn by_description(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| {
            a.description
                .to_lowercase()
                .cmp(&b.description.to_lowercase())
        });
        reverse(s, dir)
    }

    #[must_use]
    pub fn by_completion_status(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| (a.completed as i32).cmp(&(b.completed as i32)));
        reverse(s, dir)
    }

    #[must_use]
    pub fn by_indent_level(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.indent_level.cmp(&b.indent_level));
        reverse(s, dir)
    }

    #[must_use]
    pub fn by_raw(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.raw.to_lowercase().cmp(&b.raw.to_lowercase()));
        reverse(s, dir)
    }

    #[must_use]
    pub fn by_context_count(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.contexts.len().cmp(&b.contexts.len()));
        reverse(s, dir)
    }

    #[must_use]
    pub fn by_project_count(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.projects.len().cmp(&b.projects.len()));
        reverse(s, dir)
    }

    #[must_use]
    pub fn by_subtask_count(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.subtasks.len().cmp(&b.subtasks.len()));
        reverse(s, dir)
    }

    #[must_use]
    pub fn by_extension_count(dir: SortDirection) -> TaskSorter {
        let s = box_s(|a: &Task, b: &Task| a.extensions.len().cmp(&b.extensions.len()));
        reverse(s, dir)
    }

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

    #[must_use]
    pub fn then(primary: &TaskSorter, secondary: &TaskSorter) -> TaskSorter {
        Self::composite(&[primary, secondary])
    }
}
