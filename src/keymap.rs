use combine::error::StreamError;
use crate::Command;
use combine::parser::char::char;
use combine::parser::char::string;
use combine::parser::char::letter;
use combine::*;
use std::collections::HashMap;
use std::error::Error;
use termion::event::Key;

pub struct KeyMap {
  keys: HashMap<Key, Command>,
}

impl KeyMap {
  pub fn new() -> KeyMap {
    KeyMap {
      keys: HashMap::new(),
    }
  }

  pub fn add_mapping(&mut self, k: Key, c: Command) {
    self.keys.insert(k, c);
  }

  pub fn get_mapping(&self, k: Key) -> Option<Command> {
    self.keys.get(&k).cloned()
  }
}

pub fn parse_key<'a>(input: &'a str) -> Result<Key, easy::ParseError<&'a str>> {
  let char_key = || many1(none_of(">".chars())).and_then(|word: String| match word.as_str() {
    "return" => Ok('\n'),
    "ret" => Ok('\n'),
    "semicolon" => Ok(';'),
    "gt" => Ok('>'),
    "lt" => Ok('<'),
    "percent" => Ok('%'),
    "space" => Ok(' '),
    "tab" => Ok('\t'),
    c if c.len() == 1 => Ok(c.chars().next().unwrap()),
    &_ => Err(error::UnexpectedParse::Unexpected)
  });
  let modifier = || {
    optional(choice!(
      attempt(string("a-").map(|_| -> fn(char) -> Key { |c| Key::Alt(c) })),
      attempt(string("c-").map(|_| -> fn(char) -> Key { |c| Key::Ctrl(c) }))
    ))
    .map(|x| x.unwrap_or(|c| Key::Char(c)))
  };
  let non_mod = || many1(letter()).and_then(|word: String| match word.as_str() {
    "esc" => Ok(Key::Esc),
    "backtab" => Ok(Key::BackTab),
    "backspace" => Ok(Key::Backspace),
    "del" => Ok(Key::Delete),
    "home" => Ok(Key::Home),
    "end" => Ok(Key::End),
    "up" => Ok(Key::Up),
    "down" => Ok(Key::Down),
    "left" => Ok(Key::Left),
    "right" => Ok(Key::Right),
    "insert" => Ok(Key::Insert),
    "pageup" => Ok(Key::PageUp),
    "pagedown" => Ok(Key::PageDown),
    &_ => Err(error::UnexpectedParse::Unexpected)
  });
  let short = || char_key().map(|c| Key::Char(c));
  let long = || {
    between(
      char('<'),
      char('>'),
      attempt(modifier().and(char_key()).map(|(f, c)| f(c))).or(non_mod()),
    )
  };
  let parser = long().or(short());

  parser.skip(eof()).easy_parse(input).map(|(k, _)| k)
}

#[cfg(test)]
mod tests {
  use crate::keymap::parse_key;
  use combine::Parser;
  use termion::event::Key;

  #[test]
  fn key_parsing() {
    assert_eq!(parse_key("a"), Ok(Key::Char('a')));
    assert_eq!(parse_key("<a>"), Ok(Key::Char('a')));
    assert_eq!(parse_key("<a-a>"), Ok(Key::Alt('a')));
    assert_eq!(parse_key("<c-b>"), Ok(Key::Ctrl('b')));
    assert_eq!(parse_key("<return>"), Ok(Key::Char('\n')));
    assert_eq!(parse_key("<esc>"), Ok(Key::Esc));
  }
}
