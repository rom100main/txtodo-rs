use crate::date_utils;
use crate::error::TxtodoError;
use crate::extension::ExtensionHandler;
use indexmap::IndexMap;
use std::cmp::Ordering;
use std::fmt;
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Priority(pub char);

impl Priority {
    pub const ALL: [Priority; 26] = [
        Priority('A'),
        Priority('B'),
        Priority('C'),
        Priority('D'),
        Priority('E'),
        Priority('F'),
        Priority('G'),
        Priority('H'),
        Priority('I'),
        Priority('J'),
        Priority('K'),
        Priority('L'),
        Priority('M'),
        Priority('N'),
        Priority('O'),
        Priority('P'),
        Priority('Q'),
        Priority('R'),
        Priority('S'),
        Priority('T'),
        Priority('U'),
        Priority('V'),
        Priority('W'),
        Priority('X'),
        Priority('Y'),
        Priority('Z'),
    ];

    pub fn from_token(token: &str) -> Result<Self, TxtodoError> {
        let inner = if token.starts_with('(') && token.ends_with(')') && token.len() == 3 {
            &token[1..2]
        } else if token.len() == 1 {
            token
        } else {
            return Err(TxtodoError::Priority {
                message: format!("Invalid priority token: {token}"),
                priority: Some(token.to_string()),
            });
        };

        let ch = inner.chars().next().unwrap();
        if !ch.is_ascii_uppercase() {
            return Err(TxtodoError::Priority {
                message: format!("Invalid priority character: {ch}"),
                priority: Some(token.to_string()),
            });
        }
        Ok(Priority(ch))
    }

    pub fn as_char(self) -> char {
        self.0
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(time::Date),
    Array(Vec<ExtensionValue>),
}

impl ExtensionValue {
    pub fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (ExtensionValue::String(a), ExtensionValue::String(b)) => a == b,
            (ExtensionValue::Number(a), ExtensionValue::Number(b)) => a == b,
            (ExtensionValue::Boolean(a), ExtensionValue::Boolean(b)) => a == b,
            (ExtensionValue::Date(a), ExtensionValue::Date(b)) => a == b,
            (ExtensionValue::Array(a), ExtensionValue::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            _ => false,
        }
    }

    pub fn compare_to(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ExtensionValue::String(a), ExtensionValue::String(b)) => a.cmp(b),
            (ExtensionValue::Number(a), ExtensionValue::Number(b)) => {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            (ExtensionValue::Boolean(a), ExtensionValue::Boolean(b)) => {
                Ordering::Equal.then_with(|| a.cmp(b))
            }
            (ExtensionValue::Date(a), ExtensionValue::Date(b)) => a.cmp(b),
            (ExtensionValue::Array(a), ExtensionValue::Array(b)) => {
                for (x, y) in a.iter().zip(b.iter()) {
                    let c = x.compare_to(y);
                    if c != Ordering::Equal {
                        return c;
                    }
                }
                a.len().cmp(&b.len())
            }
            _ => Ordering::Equal,
        }
    }

    pub fn try_compare_to(&self, other: &Self) -> Result<Ordering, TxtodoError> {
        match (self, other) {
            (ExtensionValue::String(_), ExtensionValue::String(_))
            | (ExtensionValue::Number(_), ExtensionValue::Number(_))
            | (ExtensionValue::Boolean(_), ExtensionValue::Boolean(_))
            | (ExtensionValue::Date(_), ExtensionValue::Date(_))
            | (ExtensionValue::Array(_), ExtensionValue::Array(_)) => Ok(self.compare_to(other)),
            _ => Err(TxtodoError::Extension {
                message: "Cannot compare different extension value types".to_string(),
                extension_key: None,
            }),
        }
    }
}

impl fmt::Display for ExtensionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtensionValue::String(s) => write!(f, "{s}"),
            ExtensionValue::Number(n) => {
                if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e16 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            ExtensionValue::Boolean(b) => write!(f, "{b}"),
            ExtensionValue::Date(d) => write!(f, "{}", date_utils::format_date(*d)),
            ExtensionValue::Array(arr) => {
                let parts: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "{}", parts.join(","))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub raw: String,
    pub completed: bool,
    pub priority: Option<Priority>,
    pub creation_date: Option<time::Date>,
    pub completion_date: Option<time::Date>,
    pub description: String,
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
    pub extensions: IndexMap<String, ExtensionValue>,
    pub subtasks: Vec<Task>,
    pub indent_level: usize,
    pub parent: Option<NonNull<Task>>,
}

impl Task {
    pub fn parent(&self) -> Option<&Task> {
        self.parent.map(|p| unsafe { p.as_ref() })
    }
}

#[derive(Debug, Default, Clone)]
pub struct TaskPatch {
    pub raw: Option<String>,
    pub completed: Option<bool>,
    pub priority: Option<Option<Priority>>,
    pub creation_date: Option<Option<time::Date>>,
    pub completion_date: Option<Option<time::Date>>,
    pub description: Option<String>,
    pub projects: Option<Vec<String>>,
    pub contexts: Option<Vec<String>>,
    pub extensions: Option<IndexMap<String, ExtensionValue>>,
}

impl TaskPatch {
    pub fn apply(self, task: &mut Task) {
        if let Some(v) = self.raw {
            task.raw = v;
        }
        if let Some(v) = self.completed {
            task.completed = v;
        }
        if let Some(v) = self.priority {
            task.priority = v;
        }
        if let Some(v) = self.creation_date {
            task.creation_date = v;
        }
        if let Some(v) = self.completion_date {
            task.completion_date = v;
        }
        if let Some(v) = self.description {
            task.description = v;
        }
        if let Some(v) = self.projects {
            task.projects = v;
        }
        if let Some(v) = self.contexts {
            task.contexts = v;
        }
        if let Some(v) = self.extensions {
            task.extensions = v;
        }
    }
}

pub fn get_indent_level(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}

fn is_priority_token(token: &str) -> bool {
    if token.len() != 3 {
        return false;
    }
    let bytes = token.as_bytes();
    bytes[0] == b'(' && bytes[2] == b')' && bytes[1].is_ascii_uppercase()
}

pub fn extract_projects_and_contexts(description: &str) -> (Vec<String>, Vec<String>) {
    let mut projects = Vec::new();
    let mut contexts = Vec::new();
    let bytes = description.as_bytes();
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
    let _ = bytes; // suppress unused
    (projects, contexts)
}

pub fn build_task_from_line(
    line: &str,
    handler: &ExtensionHandler,
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

    task.extensions = handler.parse_extensions(&task.description, parent)?;

    if let Some(p) = parent {
        inherit_parent_properties(&mut task, p);
    }

    Ok(task)
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

fn inherit_parent_properties(task: &mut Task, parent: &Task) {
    if task.projects.is_empty() && !parent.projects.is_empty() {
        task.projects = parent.projects.clone();
    }
    if task.contexts.is_empty() && !parent.contexts.is_empty() {
        task.contexts = parent.contexts.clone();
    }
}
