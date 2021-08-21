use combine::parser::EasyParser;
use tui::style::{Color, Modifier, Style};
use config_macros::ConfParsable;

// Config definition
#[derive(Default, ConfParsable)]
pub struct Config {
  pub show_hidden: bool,
  pub open_cmd: String,
  pub quit_on_open: bool,
  pub file_icons: bool,
  pub icon_style: Style,
  pub dir_name_style: Style,
  pub file_name_style: Style,
  pub highlight_style: Style,
}

impl Config {
  pub fn set_opt(&mut self, name: &str, val: &str) -> Result<(), String> {
    self.get_child_mut(name)?.set_opt(val)
  }
  #[allow(dead_code)]
  pub fn get_opt(&self, name: &str) -> Result<String, String> {
    Ok(self.get_child(name)?.get_opt())
  }
}

// Lib functions

trait ConfOpt {
  fn set_opt(&mut self, val: &str) -> Result<(), String>;
  fn get_opt(&self) -> String;
}

trait ConfTree {
  fn get_child(&self, name: &str) -> Result<&dyn ConfOpt, String>;
  fn get_child_mut(&mut self, name: &str) -> Result<&mut dyn ConfOpt, String>;
}

fn parse_opt<T: std::str::FromStr>(val: &str) -> Result<T, String> {
  match val.parse::<T>() {
    Ok(res) => Ok(res),
    Err(_) => Err("Could not parse option value".to_string()),
  }
}

impl ConfOpt for bool {
  fn set_opt(&mut self, val: &str) -> Result<(), String> {
    *self = parse_opt(val)?;
    Ok(())
  }
  fn get_opt(&self) -> String {
    if *self {
      "true".to_string()
    } else {
      "false".to_string()
    }
  }
}

impl ConfOpt for i32 {
  fn set_opt(&mut self, val: &str) -> Result<(), String> {
    *self = parse_opt(val)?;
    Ok(())
  }
  fn get_opt(&self) -> String {
    self.to_string()
  }
}

impl ConfOpt for String {
  fn set_opt(&mut self, val: &str) -> Result<(), String> {
    *self = val.to_string();
    Ok(())
  }
  fn get_opt(&self) -> String {
    self.clone()
  }
}

impl ConfOpt for Style {
  fn set_opt(&mut self, val: &str) -> Result<(), String> {
    *self = parse_style(val)?;
    Ok(())
  }
  fn get_opt(&self) -> String {
    let col_to_str = |c| match c {
      Color::Reset => "reset".to_string(),
      Color::Black => "black".to_string(),
      Color::Red => "red".to_string(),
      Color::Green => "green".to_string(),
      Color::Yellow => "yellow".to_string(),
      Color::Blue => "blue".to_string(),
      Color::Magenta => "magenta".to_string(),
      Color::Cyan => "cyan".to_string(),
      Color::Gray => "gray".to_string(),
      Color::DarkGray => "darkgray".to_string(),
      Color::LightRed => "lightred".to_string(),
      Color::LightGreen => "lightgreen".to_string(),
      Color::LightYellow => "lightyellow".to_string(),
      Color::LightBlue => "lightblue".to_string(),
      Color::LightMagenta => "lightmagenta".to_string(),
      Color::LightCyan => "lightcyan".to_string(),
      Color::White => "white".to_string(),
      Color::Rgb(r, g, b) => format!("rgb:{:02X}{:02X}{:02X}", r, g, b),
      Color::Indexed(i) => format!("color{}", i),
    };
    let mod_to_char = |m: Modifier| {
      let mut r = String::new();
      if m.contains(Modifier::BOLD) {
        r.push('b');
      }
      if m.contains(Modifier::DIM) {
        r.push('d');
      }
      if m.contains(Modifier::ITALIC) {
        r.push('i');
      }
      if m.contains(Modifier::UNDERLINED) {
        r.push('u');
      }
      if m.contains(Modifier::SLOW_BLINK) {
        r.push('B');
      }
      if m.contains(Modifier::REVERSED) {
        r.push('r');
      }
      r
    };
    let fg = self.fg.map(col_to_str);
    let bg = self.bg.map(col_to_str);
    let am = mod_to_char(self.add_modifier);
    let sm = mod_to_char(self.sub_modifier);
    let mut r = String::new();
    fg.map(|fg| r += fg.as_str());
    bg.map(|bg| {
      r += ",";
      r += bg.as_str()
    });
    if !am.is_empty() {
      r += "+";
      r += am.as_str();
    }
    if !sm.is_empty() {
      r += "-";
      r += sm.as_str();
    }
    r
  }
}

mod style_parser {
  use combine::parser::char::*;
  use combine::parser::EasyParser;
  use combine::*;
  use tui::style::Color;
  use tui::style::Modifier;
  use tui::style::Style;

  pub fn color<'a>() -> impl EasyParser<&'a str, Output = Color> {
    let hex_byte = || {
      count_min_max(2, 2, hex_digit()).and_then(|x: String| {
        u8::from_str_radix(x.as_str(), 16).map_err(|_| error::UnexpectedParse::Unexpected)
      })
    };
    let rgb = || {
      string("rgb:")
        .with(hex_byte().and(hex_byte()).and(hex_byte()))
        .map(|((r, g), b)| Color::Rgb(r, g, b))
    };
    let indexed = || {
      string("color")
        .with(many1(digit()))
        .and_then(|x: String| {
          u8::from_str_radix(x.as_str(), 10).map_err(|_| error::UnexpectedParse::Unexpected)
        })
        .map(|i| Color::Indexed(i))
    };
    let named_color = || {
      many1(letter()).and_then(|x: String| match x.as_str() {
        "reset" => Ok(Color::Reset),
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" => Ok(Color::Gray),
        "darkgray" => Ok(Color::DarkGray),
        "lightred" => Ok(Color::LightRed),
        "lightgreen" => Ok(Color::LightGreen),
        "lightyellow" => Ok(Color::LightYellow),
        "lightblue" => Ok(Color::LightBlue),
        "lightmagenta" => Ok(Color::LightMagenta),
        "lightcyan" => Ok(Color::LightCyan),
        "white" => Ok(Color::White),
        &_ => Err(error::UnexpectedParse::Unexpected),
      })
    };
    choice!(attempt(rgb()), attempt(indexed()), attempt(named_color()))
  }

  fn modifier<'a>() -> impl EasyParser<&'a str, Output = Modifier> {
    satisfy_map(|x: char| match x {
      'b' => Some(Modifier::BOLD),
      'd' => Some(Modifier::DIM),
      'i' => Some(Modifier::ITALIC),
      'u' => Some(Modifier::UNDERLINED),
      'B' => Some(Modifier::SLOW_BLINK),
      'r' => Some(Modifier::REVERSED),
      _ => None,
    })
  }

  fn modifiers<'a>() -> impl EasyParser<&'a str, Output = Modifier> {
    many1(modifier()).map(|x: Vec<_>| {
      let mut r = Modifier::empty();
      for m in x {
        r = r | m;
      }
      r
    })
  }

  pub fn style<'a>() -> impl EasyParser<&'a str, Output = Style> {
    optional(color())
      .and(optional(attempt(token(',').with(color()))))
      .and(optional(attempt(token('+').with(modifiers()))))
      .and(optional(attempt(token('-').with(modifiers()))))
      .map(|(((fg, bg), add_mods), sub_mods)| {
        let mut res = Style::default();
        fg.map(|fg| res = res.fg(fg));
        bg.map(|bg| res = res.bg(bg));
        add_mods.map(|m| res = res.add_modifier(m));
        sub_mods.map(|m| res = res.remove_modifier(m));
        res
      })
  }
}

pub fn parse_style(input: &str) -> Result<Style, String> {
  match style_parser::style().easy_parse(input) {
    Err(e) => Err(format!("error parsing style: {}", e)),
    Ok((style, _)) => Ok(style),
  }
}

#[allow(dead_code)]
pub fn parse_color(input: &str) -> Result<Color, String> {
  match style_parser::color().easy_parse(input) {
    Err(e) => Err(format!("error parsing color: {}", e)),
    Ok((color, _)) => Ok(color),
  }
}

#[cfg(test)]
mod tests {
  use crate::config::*;
  use tui::style::Color::*;
  use tui::style::Modifier;
  use tui::style::Style;

  #[test]
  fn style_parsing() {
    assert!(parse_color(",").is_err());
    assert_eq!(parse_style(""), Ok(Style::default()));
    assert_eq!(parse_style("blue"), Ok(Style::default().fg(Blue)));
    assert_eq!(
      parse_style("rgb:1234AF"),
      Ok(Style::default().fg(Rgb(0x12, 0x34, 0xAF)))
    );
    assert_eq!(
      parse_style("blue,black"),
      Ok(Style::default().fg(Blue).bg(Black))
    );
    assert_eq!(parse_style(",red"), Ok(Style::default().bg(Red)));
    assert_eq!(
      parse_style("+ib"),
      Ok(
        Style::default()
          .add_modifier(Modifier::ITALIC)
          .add_modifier(Modifier::BOLD)
      )
    );
    assert_eq!(
      parse_style("+ib-u"),
      Ok(
        Style::default()
          .add_modifier(Modifier::ITALIC)
          .add_modifier(Modifier::BOLD)
          .remove_modifier(Modifier::UNDERLINED)
      )
    );
    assert_eq!(
      parse_style("yellow,rgb:0011FF+b-ui"),
      Ok(
        Style::default()
          .fg(Yellow)
          .bg(Rgb(0x00, 0x11, 0xFF))
          .add_modifier(Modifier::BOLD)
          .remove_modifier(Modifier::UNDERLINED)
          .remove_modifier(Modifier::ITALIC)
      )
    );
    assert_eq!(
      Style::default()
        .fg(Indexed(1))
        .bg(Rgb(0x00, 0x11, 0xFF))
        .add_modifier(Modifier::BOLD)
        .remove_modifier(Modifier::UNDERLINED)
        .remove_modifier(Modifier::ITALIC)
        .get_opt()
        .as_str(),
      "color1,rgb:0011FF+b-iu"
    );
  }
}
