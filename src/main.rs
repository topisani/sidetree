#![feature(try_trait)]
#![feature(generators, generator_trait)]
mod app;
mod commands;
mod config;
mod file_tree;
mod keymap;
mod prompt;
mod tree_entry;
mod util;

use crate::app::App;
use crate::commands::Command;
use crate::tree_entry::*;
use std::fs::File;

use clap::Clap;
use std::error::Error;
use std::io;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::util::event::{Event, Events};

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(
  version = "0.1.0",
  author = "Tobias Pisani <topisani@hamsterpoison.com>"
)]
struct Opts {
  /// Sets a custom config file. Could have been an Option<T> with no default too
  #[clap(short, long)]
  config: Option<String>,
}

fn default_conf_file() -> String {
  let xdg = xdg::BaseDirectories::with_prefix("sidetree").unwrap();
  let conf_file = xdg
    .place_config_file("sidetreerc")
    .expect("Cannot create config directory");
  if !conf_file.exists() {
    File::create(&conf_file).expect("Cannot create config file");
  }
  conf_file.to_str().map(|s| s.to_string()).unwrap()
}

fn main() -> Result<(), Box<dyn Error>> {
  let opts = Opts::parse();

  // Terminal initialization
  let stdout = io::stdout().into_raw_mode()?;
  let stdout = MouseTerminal::from(stdout);
  let stdout = AlternateScreen::from(stdout);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut events = Events::new();
  let mut app = App::new();
  let conf_file = opts.config.unwrap_or_else(default_conf_file);
  app.run_script_file(conf_file.as_str())?;
  loop {
    terminal.draw(|f| {
      app.draw(f);
    })?;

    if let Event::Input(input) = events.next()? {
      app.on_key(input);
      if app.statusline.has_focus() {
        events.disable_exit_key();
      } else {
        events.enable_exit_key();
      }
    }
    app.tick();
    if app.exit {
      break;
    }
  }

  Ok(())
}
