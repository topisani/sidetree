#![feature(generators, generator_trait)]
mod app;
mod cache;
mod commands;
mod config;
mod file_tree;
mod icons;
mod keymap;
mod prompt;
mod util;

use crate::commands::Command;
use crate::{app::App, cache::Cache};
use std::{fs::File, path::PathBuf};

use clap::Parser;
use commands::parse_cmds;
use std::error::Error;
use std::io;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::util::event::{Event, Events};

extern crate combine;

#[derive(Parser)]
#[clap(
  version = env!("CARGO_PKG_VERSION"),
  author = env!("CARGO_PKG_AUTHORS"),
)]
/// An interactive file tree meant to be used as a side panel for terminal text editors
struct Opts {
  /// Set a config file to use. By default uses $XDG_CONFIG_DIR/sidetree/sidetreerc
  #[clap(short, long)]
  config: Option<PathBuf>,

  /// Unless this is set, expanded paths and current selection will be saved in
  /// $XDG_CACHE_DIR/sidetree/sidetreecache.toml
  #[clap(long)]
  no_cache: bool,

  /// Preselect a path. Will expand all directories up to the path
  #[clap(short, long)]
  select: Option<PathBuf>,

  /// Commands to run on startup
  #[clap(short, long)]
  exec: Option<String>,
}

const DEFAULT_CONFIG: &'static str = include_str!("../sidetreerc");

fn default_conf_file() -> PathBuf {
  let xdg = xdg::BaseDirectories::with_prefix("sidetree").unwrap();
  let conf_file = xdg
    .place_config_file("sidetreerc")
    .expect("Cannot create config directory");
  if !conf_file.exists() {
    File::create(&conf_file).expect("Cannot create config file");
    std::fs::write(
      &conf_file,
      DEFAULT_CONFIG
    )
    .expect("Couldn't write default config file");
  }
  conf_file
}

fn main() -> Result<(), Box<dyn Error>> {
  let opts = Opts::parse();

  // Terminal initialization
  let stdout = io::stdout().into_raw_mode()?;
  let stdout = MouseTerminal::from(stdout);
  let stdout = AlternateScreen::from(stdout);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let events = Events::new();

  let cache = if !opts.no_cache {
    Cache::from_file(&Cache::default_file_path()).expect("Failed to read cache file")
  } else {
    Cache::default()
  };

  let mut app = App::new(cache);
  let conf_file = opts.config.unwrap_or_else(default_conf_file);

  app.run_script_file(&conf_file)?;
  if opts.exec.is_some() {
    app.run_commands(&parse_cmds(&opts.exec.unwrap())?)
  }

  if let Some(path) = opts.select {
    app.tree.expand_to_path(&path);
    app.tree.update(&app.config);
    app.tree.select_path(&path);
  }

  loop {
    terminal.draw(|f| {
      app.draw(f);
    })?;

    if let Event::Input(input) = events.next()? {
      app.on_key(input);
    }
    app.tick();
    if app.exit {
      break;
    }
  }

  if !opts.no_cache {
    app.get_cache().write_file(&Cache::default_file_path())
  }

  Ok(())
}
