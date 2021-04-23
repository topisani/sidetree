use crate::commands::CommandQueue;
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
  fn on_submit(&mut self, cmds: &mut CommandQueue, input: &str);
  fn on_cancel(&mut self, cmds: &mut CommandQueue);
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
  pub fn on_key(&mut self, cmds: &mut CommandQueue, key: Key) -> bool {
    match key {
      Key::Char('\n') => {
        self.submit(cmds);
        true
      }
      Key::Char(c) => {
        self.input.push(c);
        false
      }
      Key::Backspace => {
        self.input.pop();
        false
      }
      Key::Esc => {
        self.cancel(cmds);
        true
      }
      _ => false,
    }
  }

  pub fn submit(&mut self, cmds: &mut CommandQueue) {
    self.prompt.on_submit(cmds, self.input.as_str());
    self.input.clear();
  }
  pub fn cancel(&mut self, cmds: &mut CommandQueue) {
    self.prompt.on_cancel(cmds);
    self.input.clear();
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
    if let Some(p) = &self.prompt_state {
      true
    } else {
      false
    }
  }

  /// Handle a key
  /// Return true if the tree should be updated
  pub fn on_key(&mut self, cmds: &mut CommandQueue, key: Key) -> bool {
    if let Some(p) = &mut self.prompt_state {
      if p.on_key(cmds, key) {
        self.prompt_state = None;
        return true;
      }
    }
    false
  }

  pub fn prompt(&mut self, prompt: Box<dyn Prompt>) {
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
