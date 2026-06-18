//! A library for parsing, serializing, and managing tasks in the [todo.txt](http://todotxt.org) format.
//!
//! txtodo provides a full-featured [`Task`] model with support for priorities, dates, projects, contexts, extensions, and hierarchical subtasks.
//!
//! # Core components
//!
//! - [`TodoTxtParser`]: parse raw todo.txt content into [`Task`]s.
//! - [`TodoTxtSerializer`]: serialize [`Task`]s back to todo.txt format.
//! - [`TodoOptions`]: configuration for parser/serializer behaviour.
//! - [`TxtodoError`]: unified error type for all operations.
//!
//! # Quick start
//!
//! ```no_run
//! # use txtodo::*;
//! # fn main() -> Result<(), TxtodoError> {
//! let parser = TodoTxtParser::new();
//! let tasks = parser.parse_file("(A) 2024-01-15 Buy milk @home +shopping")?;
//! let serializer = TodoTxtSerializer::new();
//! let output = serializer.serialize_tasks(&tasks)?;
//! println!("{output}");
//! # Ok(())
//! # }
//! ```

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
pub use filters::{TaskFilter, TaskFilters};
pub use options::TodoOptions;
pub use parser::TodoTxtParser;
pub use serializer::TodoTxtSerializer;
pub use sorts::{SortDirection, TaskSorts, TaskSorter};
pub use task::{ExtensionValue, Priority, Task, TaskPatch};
pub use todotxt::{TaskInput, TodoTxt, resolve_index};
