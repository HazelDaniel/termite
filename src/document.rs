
use crate::filetype::FileType;
use crate::row::Row;
use crate::utils::HighlightingOptions;

use std::env;
use std::thread;
use std::time::Duration;
use std::fs;
use unicode_segmentation;

pub struct Document {
    pub rows: Vec<Row>,
    pub file_name: String,
    pub is_loaded: bool,
    pub dirty: bool,
    pub file_type: FileType,
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

impl Document {
    pub fn load(&mut self) {
        let variables: Vec<String> = env::args().collect();
        if let Some(file_name) = &variables.get(1) {
            let contents = fs::read_to_string(file_name).expect("Something went wrong reading the file");
            // todo: later we use a more elegant way to exit with error message
            for (index, row) in contents.lines().enumerate() {
                self.rows.push(Row::from(row.to_owned()));
            }
        }

        self.is_loaded = true;
    }
}