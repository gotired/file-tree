use crate::types::InputMode;

use ratatui::widgets::ListState;
use std::collections::BTreeMap;

pub struct App {
    pub raw_paths: Vec<String>,
    pub input_buffer: String,
    pub mode: InputMode,
    pub list_state: ListState,
    pub tree_items: Vec<(String, String)>,
    pub status_message: String,
    pub pending_children: Vec<String>,
    pub edit_original_path: String,
}

#[derive(Debug, Default)]
pub struct Node {
    pub children: BTreeMap<String, Node>,
}

pub struct Handle;
