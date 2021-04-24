use crate::App;
use crate::commands::Config;
use crate::util::StatefulList;
use crate::TreeEntry;
use crate::TreeEntryLine;
use std::path::PathBuf;
use tui::{
  buffer::Buffer,
  layout::Rect,
  style::{Color, Modifier, Style},
  widgets::List,
  widgets::ListItem,
  widgets::StatefulWidget,
};

pub struct FileTreeState {
  pub root_entry: TreeEntry,
  lines: StatefulList<TreeEntryLine>,
}

impl FileTreeState {
  pub fn new(path: PathBuf) -> FileTreeState {
    let mut res = FileTreeState {
      root_entry: TreeEntry::new(path),
      lines: StatefulList::new(),
    };
    res.lines.state.select(Some(0));
    res
  }

  /// Rescan the file system and rebuild the list
  pub fn update(&mut self, cfg: &Config) {
    self.root_entry.expanded = true;
    self.root_entry.update();
    self.rebuild_list(cfg);
  }

  pub fn select_next(&mut self) {
    self.lines.next()
  }
  pub fn select_prev(&mut self) {
    self.lines.previous()
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
      return root
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
