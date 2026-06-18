use crate::error::TxtodoError;
use crate::task::{ExtensionValue, Task};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;

/// A thread-safe function pointer for parsing a raw string value into an [`ExtensionValue`].
///
/// The function receives the raw string extracted from a todo.txt line (after the `key:` prefix)
/// and must return an [`ExtensionValue`] on success.
/// Return a [`TxtodoError`] to signal a parse failure.
///
/// # Signature
///
/// ```text
/// fn(raw_value: &str) -> Result<ExtensionValue, TxtodoError>
/// ```
pub type ParsingFn = Arc<dyn Fn(&str) -> Result<ExtensionValue, TxtodoError> + Send + Sync>;

/// A thread-safe function pointer for serializing an [`ExtensionValue`] back into a string.
///
/// The function receives an [`ExtensionValue`] and produces its string representation,
/// which will be appended after the `key:` prefix when writing todo.txt lines.
///
/// # Signature
///
/// ```text
/// fn(value: &ExtensionValue) -> Result<String, TxtodoError>
/// ```
pub type SerializingFn = Arc<dyn Fn(&ExtensionValue) -> Result<String, TxtodoError> + Send + Sync>;

/// A custom extension definition for the todo.txt format.
///
/// Extensions allow storing arbitrary `key:value` pairs in todo.txt task lines.
/// Use the builder methods [`with_parser`](Self::with_parser),
/// [`with_serializer`](Self::with_serializer), [`inherit`](Self::inherit),
/// and [`shadow`](Self::shadow) to configure behaviour,
/// then register the extension with [`ExtensionHandler::add_extension`].
///
/// When no custom parser is provided, values are auto-detected as
/// booleans, numbers, dates, quoted strings, comma-separated lists, or plain strings.
///
/// # Examples
///
/// ```rust
/// use txtodo::*;
/// use std::sync::Arc;
///
/// fn main() -> Result<(), TxtodoError> {
///     let ext = TodoTxtExtension::new("pri")
///         .inherit(false)
///         .shadow(true);
///
///     let mut handler = ExtensionHandler::new();
///     handler.add_extension(ext)?;
///     assert!(handler.has_extension("pri"));
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct TodoTxtExtension {
    /// The extension key as it appears in todo.txt lines (e.g. `"due"` in `due:2025-01-01`).
    pub key: String,
    pub(crate) parsing_function: Option<ParsingFn>,
    pub(crate) serializing_function: Option<SerializingFn>,
    pub(crate) inherit: bool,
    pub(crate) shadow: bool,
}

impl std::fmt::Debug for TodoTxtExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TodoTxtExtension")
            .field("key", &self.key)
            .field("has_parser", &self.parsing_function.is_some())
            .field("has_serializer", &self.serializing_function.is_some())
            .field("inherit", &self.inherit)
            .field("shadow", &self.shadow)
            .finish()
    }
}

impl TodoTxtExtension {
    /// Creates a new extension with the given `key`.
    ///
    /// By default [`inherit`](Self::inherit) and [`shadow`](Self::shadow) are both `true`,
    /// and no custom parser or serializer is set (auto-detection is used).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let ext = TodoTxtExtension::new("due");
    ///     assert_eq!(ext.key, "due");
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            parsing_function: None,
            serializing_function: None,
            inherit: true,
            shadow: true,
        }
    }

    /// Attaches a custom parsing function to this extension.
    ///
    /// When set, the parser is called instead of the default auto-detection logic
    /// whenever a `key:value` pair with this extension's key is encountered.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    /// use std::sync::Arc;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let ext = TodoTxtExtension::new("pri").with_parser(Arc::new(|raw| {
    ///         Ok(ExtensionValue::String(raw.to_uppercase()))
    ///     }));
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(ext)?;
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn with_parser(mut self, f: ParsingFn) -> Self {
        self.parsing_function = Some(f);
        self
    }

    /// Attaches a custom serialization function to this extension.
    ///
    /// When set, the serializer is called instead of the default serialization
    /// whenever an [`ExtensionValue`] for this key needs to be written back to a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    /// use std::sync::Arc;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let ext = TodoTxtExtension::new("pri").with_serializer(Arc::new(|val| {
    ///         Ok(val.to_string().to_lowercase())
    ///     }));
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(ext)?;
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn with_serializer(mut self, f: SerializingFn) -> Self {
        self.serializing_function = Some(f);
        self
    }

    /// Sets whether this extension's value is inherited from a parent task.
    ///
    /// When `true` (the default), subtasks copy this extension's value from their parent during parsing.
    /// When `false`, the parent's value is ignored for this key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let ext = TodoTxtExtension::new("project").inherit(false);
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(ext)?;
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn inherit(mut self, b: bool) -> Self {
        self.inherit = b;
        self
    }

    /// Sets whether this extension's value shadows (replaces) the parent's value.
    ///
    /// When `true` (the default), a value on the current task fully replaces the parent's value for the same key.
    /// When `false`, both values are collected into an [`ExtensionValue::Array`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let ext = TodoTxtExtension::new("tag").shadow(false);
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(ext)?;
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn shadow(mut self, b: bool) -> Self {
        self.shadow = b;
        self
    }
}

/// A registry of [`TodoTxtExtension`]s that handles parsing and serializing `key:value` pairs in todo.txt task lines.
///
/// Extensions are stored with case-insensitive keys.
/// Use [`add_extension`](Self::add_extension) to register extensions and then pass the handler to the task parsing/serialization pipeline.
///
/// # Examples
///
/// ```rust
/// use txtodo::*;
///
/// fn main() -> Result<(), TxtodoError> {
///     let mut handler = ExtensionHandler::new();
///     handler.add_extension(TodoTxtExtension::new("due"))?;
///     handler.add_extension(TodoTxtExtension::new("pri"))?;
///     assert!(handler.has_extension("due"));
///     assert_eq!(handler.all_extensions().len(), 2);
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ExtensionHandler {
    extensions: HashMap<String, TodoTxtExtension>,
}

impl std::fmt::Debug for ExtensionHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionHandler")
            .field("extensions", &self.extensions.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Default for ExtensionHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionHandler {
    /// Creates an empty extension handler with no registered extensions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let handler = ExtensionHandler::new();
    ///     assert!(handler.all_extensions().is_empty());
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }

    pub(crate) fn with_extensions(
        exts: impl IntoIterator<Item = TodoTxtExtension>,
    ) -> Result<Self, TxtodoError> {
        let mut h = Self::new();
        for e in exts {
            h.add_extension(e)?;
        }
        Ok(h)
    }

    /// Registers a new extension in the handler.
    ///
    /// The extension key must be non-empty (after trimming) and must not collide
    /// with an already-registered extension (comparison is case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError::Validation`] if the key is empty or whitespace-only.
    /// Returns [`TxtodoError::Extension`] if an extension with the same key already exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(TodoTxtExtension::new("due"))?;
    ///     assert!(handler.has_extension("due"));
    ///     Ok(())
    /// }
    /// ```
    pub fn add_extension(&mut self, extension: TodoTxtExtension) -> Result<(), TxtodoError> {
        if extension.key.trim().is_empty() {
            return Err(TxtodoError::Validation {
                message: "Extension key cannot be empty".to_string(),
                field: Some("key".to_string()),
            });
        }
        let lk = extension.key.to_lowercase();
        if self.extensions.contains_key(&lk) {
            return Err(TxtodoError::Extension {
                message: format!("Extension '{}' already exists", extension.key),
                extension_key: Some(extension.key.clone()),
            });
        }
        self.extensions.insert(lk, extension);
        Ok(())
    }

    /// Removes a previously registered extension by key.
    ///
    /// Returns `true` if the extension was present and removed.
    ///
    /// # Errors
    ///
    /// Returns [`TxtodoError::Validation`] if the key is empty.
    /// Returns [`TxtodoError::Extension`] if no extension with that key exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(TodoTxtExtension::new("due"))?;
    ///     let removed = handler.remove_extension("due")?;
    ///     assert!(removed);
    ///     assert!(!handler.has_extension("due"));
    ///     Ok(())
    /// }
    /// ```
    pub fn remove_extension(&mut self, key: &str) -> Result<bool, TxtodoError> {
        if key.is_empty() {
            return Err(TxtodoError::Validation {
                message: "Extension key cannot be empty".to_string(),
                field: Some("key".to_string()),
            });
        }
        let lk = key.to_lowercase();
        if !self.extensions.contains_key(&lk) {
            return Err(TxtodoError::Extension {
                message: format!("Extension '{key}' does not exist"),
                extension_key: Some(key.to_string()),
            });
        }
        Ok(self.extensions.remove(&lk).is_some())
    }

    /// Returns `true` if an extension with the given key is registered.
    ///
    /// The lookup is case-insensitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(TodoTxtExtension::new("due"))?;
    ///     assert!(handler.has_extension("DUE"));
    ///     assert!(!handler.has_extension("pri"));
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn has_extension(&self, key: &str) -> bool {
        self.extensions.contains_key(&key.to_lowercase())
    }

    /// Returns a reference to the extension registered under the given key, or `None`.
    ///
    /// The lookup is case-insensitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(TodoTxtExtension::new("due"))?;
    ///     let ext = handler.get_extension("due");
    ///     assert!(ext.is_some());
    ///     assert_eq!(ext.unwrap().key, "due");
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn get_extension(&self, key: &str) -> Option<&TodoTxtExtension> {
        self.extensions.get(&key.to_lowercase())
    }

    /// Returns references to all registered extensions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use txtodo::*;
    ///
    /// fn main() -> Result<(), TxtodoError> {
    ///     let mut handler = ExtensionHandler::new();
    ///     handler.add_extension(TodoTxtExtension::new("due"))?;
    ///     handler.add_extension(TodoTxtExtension::new("pri"))?;
    ///     let all = handler.all_extensions();
    ///     assert_eq!(all.len(), 2);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn all_extensions(&self) -> Vec<&TodoTxtExtension> {
        self.extensions.values().collect()
    }

    pub(crate) fn parse_extensions(
        &self,
        text: &str,
        parent: Option<&Task>,
    ) -> Result<IndexMap<String, ExtensionValue>, TxtodoError> {
        let mut extensions: IndexMap<String, ExtensionValue> = IndexMap::new();

        if let Some(p) = parent {
            for (key, value) in &p.extensions {
                let ext = self.get_extension(key);
                if let Some(ext) = ext
                    && !ext.inherit
                {
                    continue;
                }
                extensions.insert(key.clone(), value.clone());
            }
        }

        for (key, raw_value) in extract_key_value_tokens(text) {
            let ext = self.get_extension(&key);
            let parsed = match ext.as_ref().and_then(|e| e.parsing_function.as_ref()) {
                Some(parser) => match parser(&raw_value) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(TxtodoError::Extension {
                            message: format!("Failed to parse extension '{key}': {e}"),
                            extension_key: Some(key),
                        });
                    }
                },
                None => parse_value_by_type(&raw_value, true)?,
            };

            if let Some(ext) = ext
                && !ext.shadow
            {
                if let Some(parent_value) = extensions.get(&key)
                    && !parent_value.equals(&parsed)
                {
                    let merged = ExtensionValue::Array(vec![parent_value.clone(), parsed.clone()]);
                    extensions.insert(key.clone(), merged);
                    continue;
                }
                if extensions.contains_key(&key) {
                    continue;
                }
            }
            extensions.insert(key, parsed);
        }

        Ok(extensions)
    }

    pub(crate) fn serialize_extensions(
        &self,
        extensions: &IndexMap<String, ExtensionValue>,
    ) -> Result<Vec<String>, TxtodoError> {
        let mut result = Vec::new();
        for (key, value) in extensions {
            let ext = self.get_extension(key);
            let serialized = match ext.and_then(|e| e.serializing_function.as_ref()) {
                Some(ser) => match ser(value) {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(TxtodoError::Extension {
                            message: format!("Failed to serialize extension '{key}': {e}"),
                            extension_key: Some(key.clone()),
                        });
                    }
                },
                None => serialize_value_by_type(value),
            };
            result.push(format!("{key}:{serialized}"));
        }
        Ok(result)
    }
}

fn extract_key_value_tokens(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
            i += 1;
        }
        let key = &chars[start..i];
        if !key.is_empty() && i < chars.len() && chars[i] == ':' {
            let k: String = key.iter().collect();
            i += 1;
            let val_start = i;
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            let v: String = chars[val_start..i].iter().collect();
            out.push((k, v));
        } else if i < chars.len() {
            i += 1;
        }
    }
    out
}

fn parse_value_by_type(value: &str, listable: bool) -> Result<ExtensionValue, TxtodoError> {
    let bytes = value.as_bytes();
    if value.len() >= 2 {
        let first = bytes[0];
        let last = bytes[value.len() - 1];
        if (first == b'(' && last == b')')
            || (first == b'[' && last == b']')
            || (first == b'{' && last == b'}')
        {
            let inner = &value[1..value.len() - 1];
            return parse_value_by_type(inner.trim(), listable);
        }
    }

    if value.len() >= 2 {
        let first = bytes[0];
        let last = bytes[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return Ok(ExtensionValue::String(
                value[1..value.len() - 1].to_string(),
            ));
        }
    }

    if listable && value.contains(',') {
        let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
        let mut arr = Vec::new();
        for p in parts {
            arr.push(parse_value_by_type(p, false)?);
        }
        return Ok(ExtensionValue::Array(arr));
    }

    if crate::date_utils::is_date(value) {
        return Ok(ExtensionValue::Date(crate::date_utils::parse_date(value)?));
    }

    let lower = value.to_lowercase();
    if lower == "true" || lower == "false" {
        return Ok(ExtensionValue::Boolean(lower == "true"));
    }
    if lower == "yes" || lower == "no" {
        return Ok(ExtensionValue::Boolean(lower == "yes"));
    }
    if lower == "y" || lower == "n" {
        return Ok(ExtensionValue::Boolean(lower == "y"));
    }
    if lower == "on" || lower == "off" {
        return Ok(ExtensionValue::Boolean(lower == "on"));
    }

    if let Ok(n) = value.parse::<f64>() {
        return Ok(ExtensionValue::Number(n));
    }

    Ok(ExtensionValue::String(value.to_string()))
}

fn serialize_value_by_type(value: &ExtensionValue) -> String {
    match value {
        ExtensionValue::Array(arr) => arr
            .iter()
            .map(serialize_value_by_type)
            .collect::<Vec<_>>()
            .join(","),
        ExtensionValue::String(s) if s.contains(' ') => format!("\"{s}\""),
        _ => value.to_string(),
    }
}
