#![feature(try_trait)]
mod util;

use crate::util::StatefulList;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Text;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::{Block, Borders};
use tui::Frame;
use tui::Terminal;

use crate::util::event::{Event, Events};

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

  pub fn update(&mut self) {
    if self.expanded && self.children.is_empty() {
      self.build_children()
    }
    for child in &mut self.children {
      child.update()
    }
  }

  pub fn build_children(&mut self) {
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

  pub fn toggle_expanded(&mut self) {
    self.expanded = !self.expanded;
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

pub struct App {
  pub root_entry: TreeEntry,
  pub entries: StatefulList<TreeEntryLine>,
  pub exit: bool,
}

impl App {
  pub fn new() -> App {
    App {
      root_entry: TreeEntry::new(PathBuf::from("./")),
      entries: StatefulList::new(),
      exit: false,
    }
  }

  pub fn update(&mut self) {
    self.root_entry.expanded = true;
    self.root_entry.update();
    self.rebuild_list();
  }

  pub fn rebuild_list(&mut self) {
    self.entries.items = self.root_entry.build_lines_rec(0).collect();
  }

  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
      .split(f.size());
    let items: Vec<ListItem> = self
      .entries
      .items
      .iter()
      .map(|tel| ListItem::new("  ".repeat(tel.level) + tel.line.as_str()))
      .collect();
    let list = List::new(items)
      .style(Style::default().fg(Color::White))
      .highlight_style(
        Style::default()
          .bg(Color::LightGreen)
          .add_modifier(Modifier::BOLD),
      );
    f.render_stateful_widget(list, chunks[0], &mut self.entries.state);
  }

  pub fn on_key(&mut self, k: Key) {
    match k {
      Key::Char('q') => {
        self.exit = true;
      }
      Key::Char('j') => {
        self.entries.next();
      }
      Key::Char('k') => {
        self.entries.previous();
      }
      Key::Char('l') | Key::Char('\n') => {
        if let Some(entry) = self.selected_mut() {
          entry.toggle_expanded();
        }
      }
      _ => {}
    }
    self.update();
  }

  pub fn selected_mut(&mut self) -> Option<&mut TreeEntry> {
    self.root_entry.find_mut(self.entries.selected()?)
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  // Terminal initialization
  let stdout = io::stdout().into_raw_mode()?;
  let stdout = MouseTerminal::from(stdout);
  let stdout = AlternateScreen::from(stdout);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let events = Events::new();
  let mut app = App::new();
  app.update();
  loop {
    if app.exit {
      break;
    }
    terminal.draw(|f| {
      app.draw(f);
    })?;

    if let Event::Input(input) = events.next()? {
      // app.update();
      app.on_key(input);
    }
  }

  Ok(())
}
