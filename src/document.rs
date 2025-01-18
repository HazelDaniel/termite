use crate::filetype::FileType;
use crate::row::Row;
use crate::utils::HighlightingOptions;

pub struct Document {
    rows: Vec<Row>,
    pub file_name: String,
    pub is_loaded: bool,
    dirty: bool,
    file_type: FileType,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            rows: vec![],
            file_name: "".to_owned(),
            dirty: false,
            is_loaded: false,
            file_type: FileType {
                name: "rust".to_owned(),
                highlighting_ops: HighlightingOptions::default(),
            }
        }
    }
}