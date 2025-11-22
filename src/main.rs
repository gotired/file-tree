use std::collections::BTreeMap;
use std::io::{self, BufRead};

#[derive(Debug, Default)]
struct Node {
    children: BTreeMap<String, Node>,
}

impl Node {
    fn insert(&mut self, parts: &[&str]) {
        if let Some((first, rest)) = parts.split_first() {
            let child = self.children.entry(first.to_string()).or_default();
            child.insert(rest);
        }
    }

    fn print_tree(&self, prefix: &str) {
        let count = self.children.len();

        for (i, (name, node)) in self.children.iter().enumerate() {
            let is_last = i == count - 1;
            let is_dir = !node.children.is_empty();

            let connector = if is_last { "└── " } else { "├── " };

            let display_name = if is_dir {
                format!("{}/", name)
            } else {
                name.clone()
            };
            println!("{}{}{}", prefix, connector, display_name);

            let child_prefix = if is_last { "   " } else { "│  " };
            let new_prefix = format!("{}{}", prefix, child_prefix);

            node.print_tree(&new_prefix);
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut root = Node::default();

    for line in stdin.lock().lines() {
        if let Ok(path) = line {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split('/').collect();
            root.insert(&parts);
        }
    }

    root.print_tree("");
}
