use std::path::PathBuf;

pub struct TreeEntry {
  pub path: PathBuf,
  pub is_dir: bool,
  pub expanded: bool,
  pub children: Vec<TreeEntry>,
}

/// A line in the FileTree widget.
/// Identified by `path` which is used to locate the matching
pub struct TreeEntryLine {
  pub path: PathBuf,
  pub line: String,
  pub level: usize,
}

impl TreeEntry {
  pub fn new(path: PathBuf) -> TreeEntry {
    let path = std::fs::canonicalize(path.as_path()).unwrap_or(path);
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
  pub fn collapse(&mut self) {
    self.expanded = false;
  }
  pub fn expand(&mut self) {
    self.expanded = true;
  }

  pub fn update(&mut self) {
    if self.expanded {
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
          .filter_map(|p| {
            p.map(|p| p.path())
              .map(|p| {
                self
                  .children
                  .iter()
                  .position(|e| e.path == p)
                  .map(|i| self.children.remove(i))
                  .unwrap_or_else(|| TreeEntry::new(p))
              })
              .ok()
          })
          .collect()
      })
      .unwrap_or(vec![]);
    self.children.sort_by(|a, b| a.path.cmp(&b.path));
    self.children.sort_by(|a, b| b.is_dir.cmp(&a.is_dir));
  }
  pub fn build_line(&self, level: usize) -> Option<TreeEntryLine> {
    self.path.file_name().and_then(|s| s.to_str()).map(|name| {
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
      TreeEntryLine {
        path: self.path.clone(),
        line: format!("{} {}", prefix, name),
        level,
      }
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
  pub fn find(&self, e: &TreeEntryLine) -> Option<&TreeEntry> {
    if e.path == self.path {
      return Some(self);
    }
    for child in &self.children {
      let res = child.find(e);
      if res.is_some() {
        return res;
      }
    }
    return None;
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
