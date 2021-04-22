mod commands;
mod file_tree;
mod tree_entry;
mod util;

use std::process::Command;
use crate::file_tree::{FileTree, FileTreeState};
use crate::tree_entry::*;

use std::error::Error;
use std::io;
use std::path::PathBuf;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Frame;
use tui::Terminal;

use crate::util::event::{Event, Events};

pub struct App {
  pub tree: FileTreeState,
  pub exit: bool,
}

impl App {
  pub fn new() -> App {
    App {
      tree: FileTreeState::new(PathBuf::from(".")),
      exit: false,
    }
  }
  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
      .split(f.size());

    f.render_stateful_widget(FileTree::new(), chunks[0], &mut self.tree);
  }

  pub fn update(&mut self) {
    self.tree.update();
  }

  pub fn on_key(&mut self, k: Key) -> Option<()> {
    match k {
      Key::Char('q') => {
        self.exit = true;
      }
      Key::Char('j') => {
        self.tree.select_next();
      }
      Key::Char('k') => {
        self.tree.select_prev();
      }
      Key::Char('\n') => {
        if let Some(entry) = self.tree.entry() {
          if entry.is_dir {
            entry.toggle_expanded();
          } else {
            Command::new("sh").arg("-c").arg(format!("kcr edit {}", entry.path.to_str().unwrap())).output().expect("Failed to run");
          }
        }
      }
      Key::Char('l') => {
        if let Some(entry) = self.tree.entry() {
          if entry.is_dir {
            if !entry.expanded {
              entry.expand();
            } else {
              self.tree.select_next();
            }
          }
        };
      }
      Key::Char('h') => {
        if let Some(entry) = self.tree.entry() {
          if entry.expanded {
            entry.collapse();
          } else {
            self.tree.select_up();
          }
        };
      }
      _ => {}
    }
    self.tree.update();
    Some(())
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
