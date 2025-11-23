use crate::{
    repository::{App, Node},
    types::InputMode,
};

use arboard::Clipboard;
use ratatui::widgets::ListState;

impl App {
    pub fn new(initial_paths: Vec<String>) -> Self {
        let mut app = App {
            raw_paths: initial_paths,
            input_buffer: String::new(),
            mode: InputMode::Normal,
            list_state: ListState::default(),
            tree_items: Vec::new(),
            status_message: String::from("Ready. Press '?' for help."),
            pending_children: Vec::new(),
            edit_original_path: String::new(),
        };
        app.rebuild_tree();
        if !app.tree_items.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    pub fn rebuild_tree(&mut self) {
        let mut root = Node::default();
        for path in &self.raw_paths {
            let parts: Vec<&str> = path.trim().split('/').filter(|s| !s.is_empty()).collect();
            if !parts.is_empty() {
                root.insert(&parts);
            }
        }

        self.tree_items.clear();
        root.generate_tree_data("", "", &mut self.tree_items);

        // Fix selection bounds
        if let Some(selected) = self.list_state.selected() {
            if !self.tree_items.is_empty() && selected >= self.tree_items.len() {
                self.list_state.select(Some(self.tree_items.len() - 1));
            } else if self.tree_items.is_empty() {
                self.list_state.select(None);
            }
        }
    }

    pub fn add_path(&mut self) {
        if !self.input_buffer.trim().is_empty() {
            let new_path = self.input_buffer.clone();

            // 1. Add the new path (renamed folder or new item)
            self.raw_paths.push(new_path.clone());

            // 2. If in Edit mode, restore the children with the NEW parent name
            if self.mode == InputMode::Edit {
                for child_suffix in &self.pending_children {
                    // Reconstruct child path: "new_name" + "/" + "child_suffix"
                    let new_child_full = format!("{}/{}", new_path, child_suffix);
                    self.raw_paths.push(new_child_full);
                }

                // Clear edit memory
                self.pending_children.clear();
                self.edit_original_path.clear();

                self.status_message = "Path and children renamed.".to_string();
                self.mode = InputMode::Normal;
            } else {
                self.status_message = "Path added. Type next or Esc to finish.".to_string();
            }

            self.input_buffer.clear();
            self.rebuild_tree();
        }
    }

    pub fn initiate_delete(&mut self) {
        if self.list_state.selected().is_some() {
            self.mode = InputMode::DeleteConfirm;
            self.status_message =
                "WARNING: Press 'd' again to CONFIRM delete, 'Esc' to cancel.".to_string();
        } else {
            self.status_message = "Nothing selected to delete.".to_string();
        }
    }

    pub fn confirm_delete(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some((_, full_path)) = self.tree_items.get(idx) {
                if let Some(pos) = self.raw_paths.iter().position(|x| x == full_path) {
                    self.raw_paths.remove(pos);
                    self.status_message = format!("Deleted: {}", full_path);
                } else {
                    let prefix = format!("{}/", full_path);
                    self.raw_paths
                        .retain(|p| p != full_path && !p.starts_with(&prefix));
                    self.status_message = format!("Deleted hierarchy: {}", full_path);
                }
                self.rebuild_tree();
            }
        }
        self.mode = InputMode::Normal;
    }

    pub fn cancel_delete(&mut self) {
        self.mode = InputMode::Normal;
        self.status_message = "Deletion cancelled.".to_string();
    }

    pub fn edit_selected(&mut self) {
        // Step 1: Get the path and Clone it to end the borrow of self.tree_items immediately
        let path_to_edit = if let Some(idx) = self.list_state.selected() {
            self.tree_items.get(idx).map(|(_, p)| p.clone())
        } else {
            None
        };

        // Step 2: Perform mutations if we successfully got a path
        if let Some(path_to_edit) = path_to_edit {
            let full_path = path_to_edit.clone(); // Keep a copy for display/logic
            let prefix = format!("{}/", full_path);

            // --- 1. Backup children before deleting ---
            self.pending_children.clear();
            self.edit_original_path = path_to_edit.clone();

            // Identify strings in raw_paths that are children of this path
            for p in &self.raw_paths {
                if p.starts_with(&prefix) {
                    // Strip the prefix to get the relative path
                    let relative_suffix = &p[prefix.len()..];
                    self.pending_children.push(relative_suffix.to_string());
                }
            }

            // --- 2. Remove from list (Visual indication that we picked it up) ---
            self.raw_paths
                .retain(|p| p != &path_to_edit && !p.starts_with(&prefix));
            self.rebuild_tree(); // Now safe: the borrow of tree_items has ended

            // --- 3. Setup Edit Mode ---
            self.input_buffer = path_to_edit;
            self.mode = InputMode::Edit;
            self.status_message = format!(
                "Editing '{}' ({} children)...",
                full_path,
                self.pending_children.len()
            );
        }
    }

    pub fn cancel_edit(&mut self) {
        // Restore original path
        if !self.edit_original_path.is_empty() {
            self.raw_paths.push(self.edit_original_path.clone());

            // Restore children using original name
            for child_suffix in &self.pending_children {
                let original_child = format!("{}/{}", self.edit_original_path, child_suffix);
                self.raw_paths.push(original_child);
            }

            self.pending_children.clear();
            self.edit_original_path.clear();
            self.rebuild_tree();
        }

        self.mode = InputMode::Normal;
        self.input_buffer.clear();
        self.status_message = "Edit Cancelled.".to_string();
    }

    pub fn copy_to_clipboard(&mut self) {
        let content: String = self
            .tree_items
            .iter()
            .map(|(line, _)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        match Clipboard::new() {
            Ok(mut ctx) => {
                if let Err(e) = ctx.set_text(content) {
                    self.status_message = format!("Clipboard error: {}", e);
                } else {
                    self.status_message = "Tree copied to clipboard!".to_string();
                }
            }
            Err(e) => self.status_message = format!("Clipboard init error: {}", e),
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tree_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tree_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
