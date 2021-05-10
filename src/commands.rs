use crate::keymap::parse_key;
use combine::Parser;
use std::{collections::VecDeque, path::Path};
use std::path::PathBuf;
use termion::event::Key;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
  Quit,
  Shell(String, Vec<String>),
  Open(Option<PathBuf>),
  CmdStr(String),
  Echo(String),
  Set(String, String),
  Cd(Option<PathBuf>),
  MapKey(Key, Box<Command>),
  Block(Vec<Command>),
}

pub fn build_cmd(cmd: String, args: Vec<String>) -> Result<Command, String> {
  match cmd.as_str() {
    "quit" => Ok(Command::Quit),
    "open" => Ok(Command::Open(None)),
    "set" => Ok(Command::Set(args[0].clone(), args[1].clone())),
    "echo" => Ok(Command::Echo(args.join(" "))),
    "shell" => Ok(Command::Shell(args.join(" "), vec![])),
    "cd" => Ok(Command::Cd(args.get(0).map(PathBuf::from))),
    "map" => Ok(Command::MapKey(
      parse_key(args[0].as_str()).map_err(|_| "could not parse key")?,
      Box::new(build_cmd(args[1].clone(), args[2..].to_vec())?),
    )),
    _ => Err(format!("unknown command {}", cmd)),
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
    choice,
    Parser, Stream, StreamOnce,
  };

  fn lex<Input, P>(p: P) -> impl Parser<Input, Output = P::Output>
  where
    P: Parser<Input>,
    Input: Stream<Token = char>,
  {
    p.skip(spaces())
  }
  
  fn cmd_str_char<Input>(str_sep: char) -> impl Parser<Input, Output = char>
  where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
  {
    parser(move |input: &mut Input| {
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
        x if x == str_sep => Err(Commit::Peek(Input::Error::empty(input.position()).into())),
        _ => Ok((c, committed)),
      }
    })
  }
  
  fn is_word_char(c: char) -> bool {
    if c.is_whitespace() {
      return false;
    }
    if c == '#' {
      return false;
    }
    if c == ';' {
      return false;
    }
    return true;
  }

  fn arg<Input: Stream<Token = char>>() -> impl Parser<Input, Output = String> {
    let word_char = || satisfy(is_word_char);
    let word = || many1(word_char());
    let double_quotes = || between(char('"'), lex(char('"')), many(cmd_str_char('"')));
    let single_quotes = || between(char('\''), lex(char('\'')), many(cmd_str_char('\'')));
    choice!(
      double_quotes(),
      single_quotes(),
      word()
    )
  }

  pub fn cmd<Input>() -> impl Parser<Input, Output = (String, Vec<String>)>
  where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
  {
    lex(arg()).and(many(lex(arg())))
  }
}

pub fn parse_cmd(input: &str) -> Result<Command, String> {
  match cmd_parser::cmd().parse(input) {
    Err(_) => Err("error parsing command".to_string()),
    Ok(((cmd, args), _)) => build_cmd(cmd, args),
  }
}

pub fn read_config_file(path: &Path) -> Result<Vec<Command>, String> {
  let contents = std::fs::read_to_string(path);
  match contents {
    Ok(contents) => contents.lines().map(|l| parse_cmd(l)).collect(),
    Err(err) => Err(err.to_string()),
  }
}

#[cfg(test)]
mod test {
  use crate::commands::*;
  use combine::StreamOnce;

  fn cmd_parse_test(input: &str) -> Result<(String, Vec<String>), <&str as StreamOnce>::Error> {
    cmd_parser::cmd().parse(input).map(|((cmd, args), rem)| {
      assert!(rem.is_empty());
      (cmd, args)
    })
  }

  #[test]
  fn parse_cmd_quit() {
    let res = parse_cmd("quit");
    assert_eq!(res, Ok(Command::Quit));
  }
  #[test]
  fn parse_cmd_string() {
    let res = cmd_parse_test("cmd 'arg \"1' \"arg '2\"");
    assert_eq!(
      res,
      Ok((
        "cmd".to_string(),
        vec!["arg \"1".to_string(), "arg '2".to_string()]
      ))
    );
  }
  #[test]
  fn parse_cmd_multiple() {
    let res = cmd_parse_test("cmd1 a b; cmd2 c d;; ; ");
    assert_eq!(
      res,
      Ok((
        "cmd".to_string(),
        vec!["arg \"1".to_string(), "arg '2".to_string()]
      ))
    );
  }
}
