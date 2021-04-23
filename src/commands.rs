use serde::{Deserialize, Serialize};

use crate::FileTreeState;
use crate::PathBuf;
use crate::StatusLine;
use combine::Parser;
use std::collections::VecDeque;

pub struct CommandQueue {
  queue: VecDeque<Command>,
}

impl CommandQueue {
  pub fn new() -> CommandQueue {
    CommandQueue {
      queue: VecDeque::new(),
    }
  }

  pub fn push(&mut self, cmd: Command) {
    self.queue.push_back(cmd);
  }

  pub fn pop(&mut self) -> Option<Command> {
    self.queue.pop_front()
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  show_hidden: bool,
  open_cmd: String,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      show_hidden: false,
      open_cmd: String::from("kcr edit \"$1\"; kcr send focus"),
    }
  }
}

pub enum InputMode {
  Normal,
  Prompt,
}

pub struct App {
  pub config: Config,
  pub tree: FileTreeState,
  pub exit: bool,
  pub statusline: StatusLine,
  pub queued_commands: CommandQueue,
}

impl App {
  pub fn new() -> App {
    App {
      config: Config::default(),
      tree: FileTreeState::new(PathBuf::from(".")),
      exit: false,
      statusline: StatusLine::new(),
      queued_commands: CommandQueue::new(),
    }
  }

  pub fn run_queued_commands(&mut self) {
    while let Some(cmd) = self.queued_commands.pop() {
      self.run_command(cmd)
    }
  }
}

pub enum Command {
  Quit,
  Shell(String, Vec<String>),
  Open(Option<PathBuf>),
  CmdStr(String),
  Echo(String),
  Set(String, String),
}

impl App {
  fn error(&mut self, msg: &str) {
    self.statusline.info.error(msg)
  }

  fn set_opt(&mut self, opt: String, val: String) {
    match opt.as_str() {
        "open_cmd" => self.config.open_cmd = val,
        _ => self.error(format!("unknown option {}", opt).as_str()),
      }
  }
  
  pub fn run_command(&mut self, cmd: Command) {
    use Command::*;
    match cmd {
      Quit => {
        self.exit = true;
      }
      Shell(cmd, args) => {
        run_shell(self, cmd.as_str(), args.iter().map(|x| x.as_str()));
      }
      Open(path) => {
        let cmd = self.config.open_cmd.clone();
        let path = path.as_ref().unwrap_or_else(|| &self.tree.entry().path);
        let path = path.clone();
        run_shell(self, cmd.as_str(), path.to_str().iter().map(|x| *x))
      }
      CmdStr(cmd) => match parse_cmd(&cmd) {
        Ok(cmd) => self.run_command(cmd),
        Err(msg) => self.error(msg.as_str()),
      },
      Set(opt, val) => self.set_opt(opt, val),
      Echo(msg) => {
        self.statusline.info.info(msg.as_str());
      }
    }
  }
}

pub fn build_cmd(cmd: String, args: Vec<String>) -> Result<Command, String> {
  match cmd.as_str() {
    "quit" => Ok(Command::Quit),
    "open" => Ok(Command::Open(None)),
    "set" => Ok(Command::Set(args[0].clone(), args[1].clone())),
    "echo" => Ok(Command::Echo(args.join(" "))),
    _ => Err(format!("unknown command {}", cmd)),
  }
}

pub fn run_shell<'a, I: Iterator<Item = &'a str>>(app: &mut App, cmd: &str, args: I) {
  let output = std::process::Command::new("sh")
    .arg("-c")
    .arg(cmd)
    .arg("--")
    .args(args)
    .output();
  match output {
    Err(err) => {
      app.statusline.info.error(&err.to_string());
    }
    Ok(output) => {
      if !output.status.success() {
        app
          .statusline
          .info
          .error(format!("Command failed with {}", output.status).as_str())
      }
    }
  }
}

mod cmd_parser {
  use combine::{
    error::{Commit, ParseError},
    parser::{
      char::{char, spaces},
      function::parser,
      repeat::{many, many1},
      sequence::between,
      token::{any, satisfy, satisfy_map},
    },
    Parser, Stream, StreamOnce,
  };

  fn lex<Input, P>(p: P) -> impl Parser<Input, Output = P::Output>
  where
    P: Parser<Input>,
    Input: Stream<Token = char>,
    <Input as StreamOnce>::Error: ParseError<
      <Input as StreamOnce>::Token,
      <Input as StreamOnce>::Range,
      <Input as StreamOnce>::Position,
    >,
  {
    p.skip(spaces())
  }
  fn cmd_str_char<Input>() -> impl Parser<Input, Output = char>
  where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
  {
    parser(|input: &mut Input| {
      let (c, committed) = any().parse_lazy(input).into_result()?;
      let mut back_slash_char = satisfy_map(|c| {
        Some(match c {
          '"' => '"',
          '\'' => '\'',
          '\\' => '\\',
          '/' => '/',
          'b' => '\u{0008}',
          'f' => '\u{000c}',
          'n' => '\n',
          'r' => '\r',
          't' => '\t',
          _ => return None,
        })
      });
      match c {
        '\\' => committed.combine(|_| back_slash_char.parse_stream(input).into_result()),
        '"' => Err(Commit::Peek(Input::Error::empty(input.position()).into())),
        _ => Ok((c, committed)),
      }
    })
  }
  fn is_word_char(c: char) -> bool {
    if c.is_whitespace() {
      return false;
    }
    return true;
  }
  pub fn cmd<Input>() -> impl Parser<Input, Output = (String, Vec<String>)>
  where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
  {
    let word_char = satisfy(is_word_char);
    let word = many1(satisfy(is_word_char));
    let cmd_arg = between(char('"'), lex(char('"')), many(cmd_str_char()))
      .or(between(char('\''), lex(char('\'')), many(cmd_str_char())))
      .or(many1(word_char));

    lex(word).and(many(lex(cmd_arg)))
  }
}

pub fn parse_cmd(input: &str) -> Result<Command, String> {
  match cmd_parser::cmd().parse(input) {
    Err(_) => Err("error parsing command".to_string()),
    Ok(((cmd, args), _)) => build_cmd(cmd, args),
  }
}
