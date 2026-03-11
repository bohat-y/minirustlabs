use anyhow::Result;
use std::{cmp::Ordering, fs, path::{Path, PathBuf}};

// --- Recursive tree (source of truth) ---

pub enum Entry {
    File { name: String, #[allow(dead_code)] path: PathBuf },
    Dir { name: String, path: PathBuf, children: Vec<Entry>, expanded: bool },
}

impl Entry {
    pub fn name(&self) -> &str {
        match self {
            Entry::File { name, .. } | Entry::Dir { name, .. } => name,
        }
    }

    pub fn marker(&self) -> &'static str {
        match self {
            Entry::File { .. } => "• ",
            Entry::Dir { expanded: true, .. } => "▼ ",
            Entry::Dir { expanded: false, .. } => "▶ ",
        }
    }

    pub fn load_children(dir: &Path) -> Result<Vec<Entry>> {
        let mut entries: Vec<Entry> = fs::read_dir(dir)?
            .filter_map(|res| res.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let path = e.path();
                let is_dir = e.file_type().ok()?.is_dir();
                Some(if is_dir {
                    Entry::Dir { name, path, children: Vec::new(), expanded: false }
                } else {
                    Entry::File { name, path }
                })
            })
            .collect();

        entries.sort_by(|a, b| match (a, b) {
            (Entry::Dir { .. }, Entry::File { .. }) => Ordering::Less,
            (Entry::File { .. }, Entry::Dir { .. }) => Ordering::Greater,
            _ => a.name().to_lowercase().cmp(&b.name().to_lowercase()),
        });

        Ok(entries)
    }

    pub fn toggle(&mut self) -> Result<()> {
        if let Entry::Dir { path, children, expanded, .. } = self {
            if *expanded {
                children.clear();
                *expanded = false;
            } else {
                *children = Entry::load_children(path)?;
                *expanded = true;
            }
        }
        Ok(())
    }
}

// --- Flattened display rows (render cache) ---

pub struct DisplayRow {
    pub prefix: String,
    pub marker: &'static str,
    pub name: String,
    pub is_dir: bool,
    pub tree_path: Vec<usize>,
}

#[derive(Default)]
struct Flattener {
    rows: Vec<DisplayRow>,
    ancestors_is_last: Vec<bool>,
    path: Vec<usize>,
}

impl Flattener {
    fn walk(&mut self, entries: &[Entry]) {
        let len = entries.len();
        for (i, entry) in entries.iter().enumerate() {
            let is_last = i + 1 == len;
            self.path.push(i);

            self.rows.push(DisplayRow {
                prefix: self.build_prefix(is_last),
                marker: entry.marker(),
                name: entry.name().to_owned(),
                is_dir: matches!(entry, Entry::Dir { .. }),
                tree_path: self.path.clone(),
            });

            if let Entry::Dir { children, expanded: true, .. } = entry {
                self.ancestors_is_last.push(is_last);
                self.walk(children);
                self.ancestors_is_last.pop();
            }

            self.path.pop();
        }
    }

    fn build_prefix(&self, is_last: bool) -> String {
        if self.ancestors_is_last.is_empty() {
            return String::new();
        }
        let mut prefix = String::new();
        for &ancestor_is_last in &self.ancestors_is_last {
            prefix.push_str(if ancestor_is_last { "    " } else { "│   " });
        }
        prefix.push_str(if is_last { "╰─ " } else { "├─ " });
        prefix
    }
}

pub fn flatten(entries: &[Entry]) -> Vec<DisplayRow> {
    let mut f = Flattener::default();
    f.walk(entries);
    f.rows
}

pub fn entry_at_path<'a>(entries: &'a mut [Entry], tree_path: &[usize]) -> &'a mut Entry {
    let mut current = &mut entries[tree_path[0]];
    for &idx in &tree_path[1..] {
        current = match current {
            Entry::Dir { children, .. } => &mut children[idx],
            _ => unreachable!(),
        };
    }
    current
}
