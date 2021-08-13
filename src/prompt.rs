use crate::commands::Command;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::Paragraph;
use tui::Frame;
use unicode_width::UnicodeWidthStr;

pub trait Prompt {
  fn prompt_text(&self) -> &str;
  fn on_submit(&mut self, input: &str) -> Option<Command>;
  fn on_cancel(&mut self) -> Option<Command>;
}

struct PromptState {
  pub prompt: Box<dyn Prompt>,
  input: String,
}

impl PromptState {
  pub fn new(prompt: Box<dyn Prompt>) -> PromptState {
    PromptState {
      prompt: prompt,
      input: String::new(),
    }
  }
  /// Returns true if the prompt should be exited
  pub fn on_key(&mut self, key: Key) -> (bool, Option<Command>) {
    match key {
      Key::Char('\n') => {
        (true, self.submit())
      }
      Key::Char(c) => {
        self.input.push(c);
        (false, None)
      }
      Key::Backspace => {
        self.input.pop();
        (false, None)
      }
      Key::Esc => {
        (true, self.cancel())
      }
      _ => (false, None),
    }
  }

  pub fn submit(&mut self) -> Option<Command> {
    let cmd = self.prompt.on_submit(self.input.as_str());
    self.input.clear();
    cmd
  }
  pub fn cancel(&mut self) -> Option<Command>{
    let cmd = self.prompt.on_cancel();
    self.input.clear();
    cmd
  }

  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
    let text = vec![Spans::from(vec![
      Span::raw(self.prompt.prompt_text()),
      Span::raw(self.input.as_str()),
    ])];
    let input = Paragraph::new(text);
    f.render_widget(input, rect);
    f.set_cursor(rect.x + self.input.width() as u16 + 1, rect.y);
  }
}

pub struct InfoBox {
  info_msg: String,
}

impl InfoBox {
  pub fn new() -> InfoBox {
    InfoBox {
      info_msg: String::new(),
    }
  }
  pub fn info(&mut self, msg: &str) {
    self.info_msg = String::from(msg);
  }
  pub fn error(&mut self, msg: &str) {
    self.info_msg = String::from(msg);
  }
  pub fn clear(&mut self) {
    self.info_msg.clear();
  }
}

pub struct StatusLine {
  prompt_state: Option<PromptState>,
  pub info: InfoBox,
}

impl StatusLine {
  pub fn new() -> StatusLine {
    StatusLine {
      prompt_state: None,
      info: InfoBox::new(),
    }
  }
  /// Whether the statusline should get key events
  pub fn has_focus(&self) -> bool {
    if let Some(_) = &self.prompt_state {
      true
    } else {
      false
    }
  }

  /// Handle a key
  /// Return true if the tree should be updated
  pub fn on_key(&mut self, key: Key) -> (bool, Option<Command>) {
    if let Some(p) = &mut self.prompt_state {
      let (exit, cmd) = p.on_key(key);
      if exit {
        self.prompt_state = None;
        return (true, cmd);
      }
      return (false, cmd);
    }
    return (false, None);
  }

  pub fn prompt(&mut self, prompt: Box<dyn Prompt>) {
    self.info.clear();
    self.prompt_state = Some(PromptState::new(prompt));
  }

  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
    if let Some(prompt) = &mut self.prompt_state {
      prompt.draw(f, rect);
    } else {
      let text = vec![Spans::from(vec![Span::raw(self.info.info_msg.as_str())])];
      let input = Paragraph::new(text);
      f.render_widget(input, rect);
    }
  }
}
