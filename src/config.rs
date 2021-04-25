use crate::App;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub show_hidden: bool,
  pub open_cmd: String,
  pub quit_on_open: bool,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      show_hidden: false,
      open_cmd: String::from("kcr edit \"$1\"; kcr send focus"),
      quit_on_open: false,
    }
  }
}

impl Config {
  pub fn set_opt(&mut self, opt: String, val: String) -> Result<(), String> {
    match opt.as_str() {
      "open_cmd" => {
        self.open_cmd = val;
        Ok(())
      }
      "show_hidden" => {
        self.show_hidden = Self::parse_opt(val)?;
        Ok(())
      }
      "quit_on_open" => {
        self.quit_on_open = Self::parse_opt(val)?;
        Ok(())
      }
      _ => Err(format!("unknown option {}", opt)),
    }
  }

  fn parse_opt<T: std::str::FromStr>(val: String) -> Result<T, String> {
    match val.parse::<T>() {
      Ok(res) => Ok(res),
      Err(_) => Err("Could not parse option value".to_string()),
    }
  }
}
