use crate::error::TxtodoError;
use crate::task::{ExtensionValue, Task};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;

pub type ParsingFn = Arc<dyn Fn(&str) -> Result<ExtensionValue, TxtodoError> + Send + Sync>;
pub type SerializingFn = Arc<dyn Fn(&ExtensionValue) -> Result<String, TxtodoError> + Send + Sync>;

#[derive(Clone)]
pub struct TodoTxtExtension {
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

    #[must_use]
    pub fn with_parser(mut self, f: ParsingFn) -> Self {
        self.parsing_function = Some(f);
        self
    }

    #[must_use]
    pub fn with_serializer(mut self, f: SerializingFn) -> Self {
        self.serializing_function = Some(f);
        self
    }

    #[must_use]
    pub fn inherit(mut self, b: bool) -> Self {
        self.inherit = b;
        self
    }

    #[must_use]
    pub fn shadow(mut self, b: bool) -> Self {
        self.shadow = b;
        self
    }
}

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

    #[must_use]
    pub fn has_extension(&self, key: &str) -> bool {
        self.extensions.contains_key(&key.to_lowercase())
    }

    #[must_use]
    pub fn get_extension(&self, key: &str) -> Option<&TodoTxtExtension> {
        self.extensions.get(&key.to_lowercase())
    }

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
