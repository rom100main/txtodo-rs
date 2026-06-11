use crate::extension::TodoTxtExtension;

#[derive(Debug, Clone)]
pub struct TodoOptions {
    pub file_path: Option<String>,
    pub auto_save: bool,
    pub extensions: Vec<TodoTxtExtension>,
    pub handle_subtasks: bool,
}

impl Default for TodoOptions {
    fn default() -> Self {
        Self {
            file_path: None,
            auto_save: false,
            extensions: Vec::new(),
            handle_subtasks: true,
        }
    }
}
