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
    FileTreeState {
      root_entry: TreeEntry::new(path),
      lines: StatefulList::new(),
    }
  }

  /// Rescan the file system and rebuild the list
  pub fn update(&mut self) {
    self.root_entry.expanded = true;
    self.root_entry.update();
    self.rebuild_list();
  }

  pub fn select_next(&mut self) {
    self.lines.next()
  }
  pub fn select_prev(&mut self) {
    self.lines.previous()
  }

  /// Currently selected entry
  pub fn entry(&mut self) -> Option<&mut TreeEntry> {
    self.root_entry.find_mut(self.lines.selected()?)
  }

  /// Rebuild the list from the file tree.
  /// Does not rescan the filesystem
  fn rebuild_list(&mut self) {
    self.lines.items = self.root_entry.build_lines_rec(0).collect();
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
