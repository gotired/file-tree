use crate::repository::Node;

impl Node {
    pub fn insert(&mut self, parts: &[&str]) {
        if let Some((first, rest)) = parts.split_first() {
            let child = self.children.entry(first.to_string()).or_default();
            child.insert(rest);
        }
    }

    pub fn generate_tree_data(
        &self,
        prefix: &str,
        current_path: &str,
        rows: &mut Vec<(String, String)>,
    ) {
        let count = self.children.len();

        for (i, (name, node)) in self.children.iter().enumerate() {
            let is_last = i == count - 1;
            let is_dir = !node.children.is_empty();

            let full_path = if current_path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", current_path, name)
            };

            let connector = if is_last { "└── " } else { "├── " };
            let display_name = if is_dir {
                format!("{}/", name)
            } else {
                name.clone()
            };
            let visual_line = format!("{}{}{}", prefix, connector, display_name);

            rows.push((visual_line, full_path.clone()));

            let child_prefix = if is_last { "    " } else { "│   " };
            let new_prefix = format!("{}{}", prefix, child_prefix);

            node.generate_tree_data(&new_prefix, &full_path, rows);
        }
    }
}
