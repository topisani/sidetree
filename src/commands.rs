use std::path::PathBuf;
use crate::keymap::parse_key;
use combine::Parser;
use std::collections::VecDeque;
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

#[derive(Debug, Clone)]
pub enum Command {
  Quit,
  Shell(String, Vec<String>),
  Open(Option<PathBuf>),
  CmdStr(String),
  Echo(String),
  Set(String, String),
  Cd(Option<PathBuf>),
  MapKey(Key, Box<Command>),
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
    return true;
  }
  pub fn cmd<Input>() -> impl Parser<Input, Output = (String, Vec<String>)>
  where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
  {
    let word_char = satisfy(is_word_char);
    let word = many1(satisfy(is_word_char));
    let cmd_arg = between(char('"'), lex(char('"')), many(cmd_str_char('"')))
      .or(between(
        char('\''),
        lex(char('\'')),
        many(cmd_str_char('\'')),
      ))
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

pub fn read_config_file(path: &str) -> Result<Vec<Command>, String> {
  let contents = std::fs::read_to_string(path);
  match contents {
    Ok(contents) => contents.lines().map(|l| parse_cmd(l)).collect(),
    Err(err) => Err(err.to_string()),
  }
}
