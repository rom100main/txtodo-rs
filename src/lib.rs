#![deny(warnings)]

pub mod error;
pub mod extension;
pub mod filters;
pub mod options;
pub mod parser;
pub mod serializer;
pub mod sorts;
pub mod task;
pub mod todotxt;

mod date_utils;

pub use error::TxtodoError;
pub use extension::{ExtensionHandler, TodoTxtExtension};
pub use filters::TaskFilters;
pub use options::TodoOptions;
pub use parser::TodoTxtParser;
pub use serializer::TodoTxtSerializer;
pub use sorts::{SortDirection, TaskSorts};
pub use task::{ExtensionValue, Priority, Task, TaskPatch};
pub use todotxt::{TaskInput, TodoTxt, resolve_index};

pub type TaskFilter = std::rc::Rc<dyn Fn(&Task) -> bool>;
pub type TaskSorter = std::rc::Rc<dyn Fn(&Task, &Task) -> std::cmp::Ordering>;
