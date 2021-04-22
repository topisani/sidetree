#![feature(try_trait)]
use std::path::PathBuf;

pub struct TreeEntry {
  pub path: PathBuf,
  pub is_dir: bool,
  pub expanded: bool,
  pub children: Vec<TreeEntry>,
}

pub struct TreeEntryLine {
  pub path: PathBuf,
  pub line: String,
  pub level: usize,
}

impl TreeEntry {
  pub fn new(path: PathBuf) -> TreeEntry {
    let md = path.metadata();
    TreeEntry {
      path: path,
      is_dir: md.map(|m| m.is_dir()).unwrap_or(false),
      expanded: false,
      children: vec![],
    }
  }

  pub fn toggle_expanded(&mut self) {
    self.expanded = !self.expanded;
  }

  pub fn update(&mut self) {
    if self.expanded && self.children.is_empty() {
      self.read_fs()
    }
    for child in &mut self.children {
      child.update()
    }
  }

  pub fn read_fs(&mut self) {
    self.children = std::fs::read_dir(&self.path)
      .map(|paths| {
        paths
          .filter_map(|p| p.map(|p| p.path()).map(TreeEntry::new).ok())
          .collect()
      })
      .unwrap_or(vec![]);
    self.children.sort_by(|a, b| a.path.cmp(&b.path));
    self.children.sort_by(|a, b| b.is_dir.cmp(&a.is_dir));
  }
  pub fn build_line(&self, level: usize) -> Option<TreeEntryLine> {
    self
      .path
      .file_name()
      .and_then(|s| s.to_str())
      .and_then(|name| {
        let prefix = {
          if self.is_dir {
            if self.expanded {
              "▾ 🗁 "
            } else {
              "▸ 🗀 "
            }
          } else {
            "  🖺 "
          }
        };
        Some(TreeEntryLine {
          path: self.path.clone(),
          line: format!("{} {}", prefix, name),
          level,
        })
      })
  }

  pub fn build_lines_rec<'a>(
    &'a self,
    level: usize,
  ) -> Box<dyn Iterator<Item = TreeEntryLine> + 'a> {
    let self_line = std::iter::once(self).filter_map(move |s| s.build_line(level));
    if self.expanded {
      Box::new(
        self_line.chain(
          self
            .children
            .iter()
            .map(move |n| n.build_lines_rec(level + 1))
            .flatten(),
        ),
      )
    } else {
      Box::new(self_line)
    }
  }

  /// Find the tree entry corresponding to a `TreeEntryLine`
  pub fn find_mut(&mut self, e: &TreeEntryLine) -> Option<&mut TreeEntry> {
    if e.path == self.path {
      return Some(self);
    }
    for child in &mut self.children {
      let res = child.find_mut(e);
      if res.is_some() {
        return res;
      }
    }
    return None;
  }
}
