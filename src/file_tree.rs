use crate::util::StatefulList;
use crate::{config::Config, icons};
use path_absolutize::Absolutize;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tui::{
  buffer::Buffer,
  layout::Rect,
  style::{Color, Modifier, Style},
  widgets::List,
  widgets::ListItem,
  widgets::StatefulWidget,
};

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ExpandedPaths {
  //#[serde(flatten)]
  expanded_paths: HashSet<PathBuf>,
}

impl ExpandedPaths {
  pub fn extend(&mut self, x: &ExpandedPaths) {
    self
      .expanded_paths
      .extend(x.expanded_paths.iter().map(|x| x.clone()));
  }

  pub fn toggle_expanded(&mut self, path: &Path) {
    if !self.expanded_paths.remove(path) {
      self.expand(path);
    }
  }
  pub fn collapse(&mut self, path: &Path) {
    self.expanded_paths.remove(path);
  }

  pub fn expand(&mut self, path: &Path) {
    self.expanded_paths.insert(PathBuf::from(path));
  }

  pub fn is_expanded(&self, path: &Path) -> bool {
    self.expanded_paths.contains(path)
  }
}

pub struct FileTreeState {
  pub root_entry: TreeEntry,
  pub expanded_paths: ExpandedPaths,
  lines: StatefulList<TreeEntryLine>,
}

impl FileTreeState {
  pub fn new(path: PathBuf) -> FileTreeState {
    let mut res = FileTreeState {
      root_entry: TreeEntry::new(path),
      lines: StatefulList::new(),
      expanded_paths: ExpandedPaths::default(),
    };
    res.expanded_paths.expand(&res.root_entry.path);
    res.lines.state.select(Some(0));
    res
  }

  pub fn extend_expanded_paths(&mut self, exp: ExpandedPaths) {
    self.expanded_paths.extend(&exp);
  }

  pub fn toggle_expanded(&mut self, path: &Path) {
    self.expanded_paths.toggle_expanded(path)
  }
  pub fn collapse(&mut self, path: &Path) {
    self.expanded_paths.collapse(path)
  }

  pub fn expand(&mut self, path: &Path) {
    self.expanded_paths.expand(path)
  }

  pub fn is_expanded(&self, path: &Path) -> bool {
    self.expanded_paths.is_expanded(path)
  }

  pub fn change_root(&mut self, cfg: &Config, path: PathBuf) {
    self.root_entry = TreeEntry::new(path);
    self.root_entry.expanded = true;
    self.update(cfg);
  }

  /// Rescan the file system and rebuild the list
  pub fn update(&mut self, cfg: &Config) {
    let selected = self.line().map(|x| x.path.clone());
    self.root_entry.update(&self.expanded_paths);
    self.rebuild_list(cfg);
    if let Some(x) = selected {
      self.select_path(&x);
    }
  }

  pub fn select_next(&mut self) {
    self.lines.next()
  }
  pub fn select_prev(&mut self) {
    self.lines.previous()
  }

  pub fn select_path(&mut self, path: &Path) {
    let path = path.absolutize().expect("Error absolutizing path");
    if let Some(idx) = self.lines.items.iter().position(|line| line.path == path) {
      self.lines.select_index(idx);
    }
  }

  /// Expand parents to reveal <path>
  pub fn expand_to_path(&mut self, path: &Path) {
    let path = path.absolutize().expect("Error absolutizing path");
    for anc in path.ancestors().skip(1) {
      if !anc.starts_with(&self.root_entry.path) {
        break;
      }
      self.expand(anc);
    }
  }

  /// Select the next entry up
  pub fn select_up(&mut self) -> Option<()> {
    let level = self.lines.selected()?.level;
    while self.lines.index()? != 0 {
      self.select_prev();
      if self.lines.selected()?.level < level {
        break;
      }
    }
    Some(())
  }

  /// Currently selected entry
  pub fn entry(&self) -> &TreeEntry {
    self
      .lines
      .selected()
      .and_then(|x| self.root_entry.find(x))
      .unwrap_or(&self.root_entry)
  }

  /// Currently selected line
  fn line(&self) -> Option<&TreeEntryLine> {
    self.lines.selected()
  }

  /// Currently selected entry
  pub fn entry_mut(&mut self) -> &mut TreeEntry {
    let root = &mut self.root_entry;
    if let Some(line) = self.lines.selected_mut() {
      if let Some(entry) = root.find_mut(line) {
        return entry;
      } else {
        panic!()
      }
    } else {
      return root;
    }
  }

  /// Rebuild the list from the file tree.
  /// Does not rescan the filesystem
  fn rebuild_list(&mut self, cfg: &Config) {
    self.lines.items = self.root_entry.build_lines_rec(cfg, 0).collect();
  }
}

pub struct FileTree {}

impl FileTree {
  pub fn new() -> FileTree {
    FileTree {}
  }
}

impl StatefulWidget for FileTree {
  type State = FileTreeState;

  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    let items: Vec<ListItem> = state
      .lines
      .items
      .iter()
      .map(|tel| ListItem::new("  ".repeat(tel.level) + tel.line.as_str()))
      .collect();
    let list = List::new(items)
      .style(Style::default().fg(Color::White))
      .highlight_style(
        Style::default()
          .bg(Color::LightBlue)
          .add_modifier(Modifier::BOLD),
      );
    list.render(area, buf, &mut state.lines.state);
  }
}

#[derive(Clone)]
pub struct TreeEntry {
  pub path: PathBuf,
  pub is_dir: bool,
  pub is_link: bool,
  pub children: Vec<TreeEntry>,
  expanded: bool,
}

/// A line in the FileTree widget.
/// Identified by `path` which is used to locate the matching
pub struct TreeEntryLine {
  pub path: PathBuf,
  pub line: String,
  pub level: usize,
}

impl TreeEntry {
  fn new(path: PathBuf) -> TreeEntry {
    let path = path
      .as_path()
      .absolutize()
      .map(PathBuf::from)
      .unwrap_or(path);
    let md = path.metadata();
    let is_link = path.as_path().read_link().is_ok();
    TreeEntry {
      path,
      is_dir: md.map(|m| m.is_dir()).unwrap_or(false),
      is_link,
      children: vec![],
      expanded: false,
    }
  }

  fn update(&mut self, expanded: &ExpandedPaths) {
    self.expanded = expanded.is_expanded(&self.path);
    if self.expanded {
      self.read_fs()
    }
    for child in &mut self.children {
      child.update(expanded)
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

  fn should_show_item(&self, conf: &Config, _level: usize) -> bool {
    if !conf.show_hidden
      && self
        .path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|x| x.starts_with("."))
        .unwrap_or(false)
    {
      return false;
    }
    return true;
  }

  // https://www.nerdfonts.com/cheat-sheet
  fn icon(&self, conf: &Config) -> char {
    if conf.file_icons {
      icons::icon_for_file(self.path.as_path())
    } else {
      if self.is_dir {
        if self.expanded {
          ''
        } else {
          if self.is_link {
            ''
          } else {
            ''
          }
        }
      } else {
        if self.is_link {
          ''
        } else {
          ''
        }
      }
    }
  }

  pub fn build_line(&self, conf: &Config, level: usize) -> Option<TreeEntryLine> {
    if !self.should_show_item(conf, level) {
      return None;
    }
    self.path.file_name().and_then(|s| s.to_str()).map(|name| {
      let prefix = {
        let icon = self.icon(conf);
        let arrow = if self.is_dir {
          if self.expanded {
            '▾'
          } else {
            '▸'
          }
        } else {
          ' '
        };
        format!("{} {} ", arrow, icon)
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
    conf: &'a Config,
    level: usize,
  ) -> Box<dyn Iterator<Item = TreeEntryLine> + 'a> {
    let self_line = std::iter::once(self).filter_map(move |s| s.build_line(conf, level));
    if self.expanded {
      Box::new(
        self_line.chain(
          self
            .children
            .iter()
            .map(move |n| n.build_lines_rec(conf, level + 1))
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

  /// Get the cached variable of whether this entry is expanded.
  pub fn is_expanded(&self) -> bool {
    self.expanded
  }
}
