use std::path::Path;
use std::path::PathBuf;
use termion::event::Key;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::text::Span;
use tui::text::Spans;
use tui::text::Text;
use tui::widgets::Paragraph;
use tui::widgets::StatefulWidget;
use tui::widgets::Widget;

pub trait Prompt {
  fn prompt_text(&self) -> &str;
  fn on_submit(&mut self, info: &mut InfoBox, input: &str);
  fn on_cancel(&mut self, info: &mut InfoBox);
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
  pub fn on_key(&mut self, info: &mut InfoBox, key: Key) -> bool {
    match key {
      Key::Char('\n') => {
        self.submit(info);
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
        self.cancel(info);
        true
      }
      _ => {
        false
      }
    }
  }

  pub fn submit(&mut self, info: &mut InfoBox) {
    self.prompt.on_submit(info, self.input.as_str());
    self.input.clear();
  }
  pub fn cancel(&mut self, info: &mut InfoBox) {
    self.prompt.on_cancel(info);
    self.input.clear();
  }
}

struct PromptWidget;

impl StatefulWidget for PromptWidget {
  type State = PromptState;

  fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
    let text = vec![Spans::from(vec![
      Span::raw(state.prompt.prompt_text()),
      Span::raw(state.input.as_str()),
    ])];
    let input = Paragraph::new(text);
    input.render(rect, buf);
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
  pub fn on_key(&mut self, key: Key) {
    if let Some(p) = &mut self.prompt_state {
      if p.on_key(&mut self.info, key) {
        self.prompt_state = None
      }
    }
  }

  pub fn prompt(&mut self, prompt: Box<dyn Prompt>) {
    self.prompt_state = Some(PromptState::new(prompt));
  }
}

pub struct StatusLineWidget;

impl StatefulWidget for StatusLineWidget {
  type State = StatusLine;

  fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
    if let Some(prompt) = &mut state.prompt_state {
      PromptWidget {}.render(rect, buf, prompt)
    } else {
      let text = vec![Spans::from(vec![Span::raw(state.info.info_msg.as_str())])];
      let input = Paragraph::new(text);
      input.render(rect, buf);
    }
  }
}
