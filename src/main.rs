#![feature(try_trait)]
mod commands;
mod file_tree;
mod prompt;
mod tree_entry;
mod util;

use crate::file_tree::{FileTree, FileTreeState};
use crate::prompt::InfoBox;
use crate::prompt::Prompt;
use crate::prompt::StatusLine;
use crate::tree_entry::*;
use std::process::Command;

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
  pub statusline: StatusLine,
}

impl App {
  pub fn new() -> App {
    App {
      tree: FileTreeState::new(PathBuf::from(".")),
      exit: false,
      statusline: StatusLine::new(),
    }
  }
  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
      .split(f.size());

    f.render_stateful_widget(FileTree::new(), chunks[0], &mut self.tree);
    self.statusline.draw(f, chunks[1]);
  }

  pub fn update(&mut self) {
    self.tree.update();
  }

  pub fn on_key(&mut self, k: Key) -> Option<()> {
    if self.statusline.has_focus() {
      self.statusline.on_key(k);
      self.tree.update();
      return Some(());
    }
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
            run_shell(
              &mut self.statusline.info,
              format!("kcr edit '{}'; kcr send focus", entry.path.to_str().unwrap()),
            )
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
      Key::Char('!') => {
        if let Some(entry) = self.tree.entry() {
          self.statusline.prompt(Box::new(ShellPrompt { path: entry.path.clone() }));
        }
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

pub fn run_shell(info: &mut InfoBox, cmd: String) {
  let output = Command::new("sh").arg("-c").arg(cmd).output();
  match output {
    Err(err) => {
      info.error(&err.to_string());
    }
    Ok(output) => {
      if !output.status.success() {
        info.error(format!("Command failed with exit code {}", output.status).as_str())
      }
    }
  }
}

pub struct ShellPrompt {
  path: PathBuf,
}

impl Prompt for ShellPrompt {
  fn prompt_text(&self) -> &str {
    "!"
  }
  fn on_submit(&mut self, info: &mut InfoBox, text: &str) {
    let output = Command::new("sh")
      .arg("-c")
      .arg(text)
      .arg("--")
      .arg(self.path.to_str().unwrap_or(""))
      .output();
    match output {
      Err(err) => {
        info.error(&err.to_string());
      }
      Ok(output) => {
        if !output.status.success() {
          info.error(format!("Command failed with exit code {}", output.status).as_str())
        }
      }
    }
  }
  fn on_cancel(&mut self, _: &mut InfoBox) {}
}
