use crate::cache::Cache;
use crate::commands::parse_cmds;
use crate::commands::read_config_file;
use crate::commands::Command;
use crate::config::Config;
use crate::file_tree::{FileTree, FileTreeState};
use crate::keymap::KeyMap;
use crate::prompt::Prompt;
use crate::prompt::StatusLine;
use termion::event::MouseButton;
use tui::backend::Backend;

use std::path::{Path, PathBuf};
use termion::event::{Key, MouseEvent};
use tui::layout::{Constraint, Direction, Layout};
use tui::Frame;

pub struct App<'a> {
  pub config: Config,
  pub tree: FileTreeState,
  pub exit: bool,
  pub statusline: StatusLine<'a>,
  pub keymap: KeyMap,
}

impl<'a> App<'a> {
  pub fn new(cache: Cache) -> App<'a> {
    let mut res = App {
      config: Config::default(),
      tree: FileTreeState::new(PathBuf::from(".")),
      exit: false,
      statusline: StatusLine::new(),
      keymap: KeyMap::new(),
    };
    res.read_cache(cache);
    res.tree.update(&res.config);
    res
  }
}

impl<'a> App<'a> {
  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
      .split(f.size());

    f.render_stateful_widget(FileTree::new(&self.config), chunks[0], &mut self.tree);
    self.statusline.draw(f, chunks[1]);
  }

  pub fn read_cache(&mut self, cache: Cache) {
    self.tree.extend_expanded_paths(cache.expanded_paths);
    self.tree.update(&self.config);
    self.tree.select_path(&cache.selected_path);
  }

  pub fn get_cache(&self) -> Cache {
    Cache {
      expanded_paths: self.tree.expanded_paths.clone(),
      selected_path: self.tree.entry().path.clone(),
    }
  }

  pub fn update(&mut self) {
    self.tree.update(&self.config);
  }

  pub fn tick(&mut self) {
    self.update();
  }

  pub fn on_mouse(&mut self, me: MouseEvent) -> Option<()> {
    if self.statusline.has_focus() {
      return Some(());
    }
    if let MouseEvent::Press(button, _x, y) = me {
      match button {
        MouseButton::Left | MouseButton::Right => {
          let line = (y - 1) as usize;
          if self.tree.selected_idx() == Some(line) {
            let entry = self.tree.entry().clone();
            if entry.is_dir {
              self.tree.toggle_expanded(&entry.path);
            } else {
              self.run_command(&Command::Open(None))
            }
          } else {
            self.tree.select_nth(line);
          }
        }
        MouseButton::WheelDown => {
          self.tree.select_next();
        }
        MouseButton::WheelUp => {
          self.tree.select_prev();
        }
        _ => {}
      }
    };
    Some(())
  }

  pub fn on_key(&mut self, k: Key) -> Option<()> {
    if self.statusline.has_focus() {
      let (update, cmd) = self.statusline.on_key(k);
      if let Some(cmd) = cmd {
        self.run_command(&cmd);
      }
      if update {
        self.update();
      }
      return Some(());
    }
    if let Some(cmd) = self.keymap.get_mapping(k) {
      self.run_command(&cmd);
      return Some(());
    }
    match k {
      Key::Char('q') => {
        self.exit = true;
      }
      Key::Char('j') | Key::Down => {
        self.tree.select_next();
      }
      Key::Char('k') | Key::Up => {
        self.tree.select_prev();
      }
      Key::Char('\n') => {
        let entry = self.tree.entry().clone();
        if entry.is_dir {
          self.tree.toggle_expanded(&entry.path);
        } else {
          self.run_command(&Command::Open(None))
        }
      }
      Key::Char('l') | Key::Right => {
        let entry = self.tree.entry().clone();
        if entry.is_dir {
          if !entry.is_expanded() {
            self.tree.expand(&entry.path);
          } else {
            self.tree.select_next();
          }
        }
      }
      Key::Char('h') | Key::Left => {
        let entry = self.tree.entry().clone();
        if entry.is_expanded() {
          self.tree.collapse(&entry.path);
        } else {
          self.tree.select_up();
        }
      }
      Key::Char('!') => {
        self.statusline.prompt(Box::new(ShellPrompt {}));
      }
      Key::Char(':') => {
        self.statusline.prompt(Box::new(CmdPrompt {}));
      }
      Key::Alt('l') => {
        self.run_command(&Command::Cd(None));
      }
      Key::Char('.') => {
        self.config.show_hidden = !self.config.show_hidden;
      }
      _ => {}
    }
    Some(())
  }

  pub fn run_commands(&mut self, cmds: &Vec<Command>) {
    for c in cmds {
      self.run_command(c);
    }
  }

  pub fn run_command(&mut self, cmd: &Command) {
    use Command::*;
    match cmd {
      Quit => {
        self.quit();
      }
      Shell(cmd) => {
        self.run_shell(cmd.as_str());
      }
      Open(path) => {
        let cmd = self.config.open_cmd.clone();
        let path = path.as_ref().unwrap_or_else(|| &self.tree.entry().path);
        let _path = path.clone();
        self.run_shell(cmd.as_str());
        if self.config.quit_on_open {
          self.quit();
        }
      }
      CmdStr(cmd) => match parse_cmds(cmd) {
        Ok(cmds) => self.run_commands(&cmds),
        Err(msg) => self.error(msg.as_str()),
      },
      Set(opt, val) => {
        if let Err(e) = self.config.set_opt(opt, val) {
          self.statusline.info.error(e.as_str());
        }
      }
      Echo(msg) => {
        self.statusline.info.info(msg.as_str());
      }
      Cd(path) => {
        let path = path.as_ref().unwrap_or_else(|| &self.tree.entry().path);
        let path = path.clone();
        match std::env::set_current_dir(path.as_path()) {
          Ok(()) => self
            .tree
            .change_root(&self.config, std::env::current_dir().unwrap()),
          Err(err) => self.error(err.to_string().as_str()),
        }
      }
      MapKey(key, cmd) => {
        self.keymap.add_mapping(*key, (**cmd).clone());
      }
      Rename(name) => {
        if let Some(name) = name {
          let src = &self.tree.entry().path;
          let mut dst = src.clone();
          dst.set_file_name(name);
          // TODO: Error handling
          if !dst.exists() {
            std::fs::rename(src, dst).unwrap();
          }
        } else {
          self.statusline.prompt(Box::new(RenamePrompt {
            old_name: self
              .tree
              .entry()
              .path
              .file_name()
              .unwrap()
              .to_string_lossy()
              .into(),
          }));
        }
      }
      NewFile(name) => {
        if let Some(name) = name {
          let mut path = self.tree.current_dir();
          path.push(name);
          // TODO: Error handling
          if !path.exists() {
            if name.ends_with('/') {
              std::fs::create_dir_all(path).unwrap();
            } else {
              std::fs::write(path, "").unwrap();
            }
          }
        } else {
          self.statusline.prompt(Box::new(NewFilePrompt {}));
        }
      }
      NewDir(name) => {
        if let Some(name) = name {
          let mut path = self.tree.current_dir();
          path.push(name);
          // TODO: Error handling
          if !path.exists() {
            std::fs::create_dir_all(path).unwrap();
          }
        } else {
          self.statusline.prompt(Box::new(NewDirPrompt {}));
        }
      }

      Delete { prompt } => {
        if !prompt {
          let path = &self.tree.entry().path;
          // TODO: Error handling
          if path.is_dir() {
            std::fs::remove_dir_all(path).unwrap();
          } else {
            std::fs::remove_file(path).unwrap();
          }
        } else {
          self.statusline.prompt(Box::new(DeletePrompt {}));
        }
      }
    }
    self.update();
  }
  pub fn error(&mut self, msg: &str) {
    self.statusline.info.error(msg)
  }
  fn quit(&mut self) {
    self.exit = true;
  }

  pub fn run_script_file(&mut self, path: &Path) -> Result<(), String> {
    let cmds = read_config_file(path)?;
    self.run_commands(&cmds);
    Ok(())
  }

  fn run_shell(&mut self, cmd: &str) {
    let output = std::process::Command::new("sh")
      .arg("-c")
      .arg(cmd)
      .arg("--")
      .arg(self.tree.entry().path.to_str().unwrap_or(""))
      .env(
        "sidetree_root",
        self.tree.root_entry.path.to_str().unwrap_or(""),
      )
      .env(
        "sidetree_entry",
        self.tree.entry().path.to_str().unwrap_or(""),
      )
      .env(
        "sidetree_dir",
        self.tree.current_dir().to_str().unwrap_or(""),
      )
      .output();
    match output {
      Err(err) => {
        self.statusline.info.error(&err.to_string());
      }
      Ok(output) => {
        if !output.status.success() {
          self
            .statusline
            .info
            .error(format!("Command failed with {}", output.status).as_str())
        }
      }
    }
  }
}

pub struct ShellPrompt {}

impl Prompt for ShellPrompt {
  fn prompt_text(&self) -> &str {
    "!"
  }
  fn on_submit(&mut self, text: &str) -> Option<Command> {
    Some(Command::Shell(text.to_string()))
  }
  fn on_cancel(&mut self) -> Option<Command> {
    None
  }
}

pub struct CmdPrompt {}

impl Prompt for CmdPrompt {
  fn prompt_text(&self) -> &str {
    ":"
  }
  fn on_submit(&mut self, text: &str) -> Option<Command> {
    Some(Command::CmdStr(text.to_string()))
  }
}

pub struct RenamePrompt {
  old_name: String,
}

impl Prompt for RenamePrompt {
  fn prompt_text(&self) -> &str {
    "Rename>"
  }

  fn on_submit(&mut self, input: &str) -> Option<Command> {
    Some(Command::Rename(Some(input.into())))
  }

  fn init_text(&self) -> String {
    self.old_name.clone()
  }
}

pub struct NewFilePrompt {}

impl Prompt for NewFilePrompt {
  fn prompt_text(&self) -> &str {
    "mk>"
  }

  fn on_submit(&mut self, input: &str) -> Option<Command> {
    Some(Command::NewFile(Some(input.into())))
  }
}

pub struct NewDirPrompt {}

impl Prompt for NewDirPrompt {
  fn prompt_text(&self) -> &str {
    "New dir>"
  }

  fn on_submit(&mut self, input: &str) -> Option<Command> {
    Some(Command::NewDir(Some(input.into())))
  }
}

pub struct DeletePrompt {}

impl Prompt for DeletePrompt {
  fn prompt_text(&self) -> &str {
    "delete? [y/N]>"
  }

  fn on_submit(&mut self, input: &str) -> Option<Command> {
    if input == "y" || input == "Y" {
      Some(Command::Delete { prompt: false })
    } else {
      None
    }
  }
}
