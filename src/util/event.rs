use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::{Event as TEvent, Key, MouseEvent};
use termion::input::TermRead;

pub enum Event<A, B> {
  Key(A),
  Mouse(B),
  Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
  rx: mpsc::Receiver<Event<Key, MouseEvent>>,
  _input_handle: thread::JoinHandle<()>,
  _tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
  pub exit_key: Key,
  pub tick_rate: Duration,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      exit_key: Key::Char('q'),
      tick_rate: Duration::from_millis(250),
    }
  }
}

impl Events {
  pub fn new() -> Events {
    Events::with_config(Config::default())
  }

  pub fn with_config(config: Config) -> Events {
    let (tx, rx) = mpsc::channel();
    let _input_handle = {
      let tx = tx.clone();
      thread::spawn(move || {
        let stdin = io::stdin();
        for evt in stdin.events() {
          if let Ok(res) = evt {
            match res {
              TEvent::Key(key) => {
                if let Err(err) = tx.send(Event::Key(key)) {
                  eprintln!("{}", err);
                  return;
                }
              },
              TEvent::Mouse(mouse) => {
                if let Err(err) = tx.send(Event::Mouse(mouse)) {
                  eprintln!("{}", err);
                  return;
                }
              },
              _ => ()
            }
          }
        }
      })
    };
    let _tick_handle = {
      thread::spawn(move || loop {
        if tx.send(Event::Tick).is_err() {
          break;
        }
        thread::sleep(config.tick_rate);
      })
    };
    Events {
      rx,
      _input_handle,
      _tick_handle,
    }
  }

  pub fn next(&self) -> Result<Event<Key, MouseEvent>, mpsc::RecvError> {
    self.rx.recv()
  }
}
