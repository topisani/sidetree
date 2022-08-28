use tui::widgets::ListState;

pub mod event;

pub struct StatefulList<T> {
  pub state: ListState,
  pub items: Vec<T>,
}

impl<T> StatefulList<T> {
  pub fn new() -> StatefulList<T> {
    StatefulList {
      state: ListState::default(),
      items: Vec::new(),
    }
  }

  #[allow(dead_code)]
  pub fn with_items(items: Vec<T>) -> StatefulList<T> {
    StatefulList {
      state: ListState::default(),
      items,
    }
  }

  pub fn nth(&mut self, n: usize) {
    self.state.select(Some((n).min(self.items.len() - 1)));
  }

  pub fn next(&mut self) {
    if let Some(i) = self.state.selected() {
      self.state.select(Some((i + 1).min(self.items.len() - 1)));
    } else {
      self.state.select(Some(0))
    }
  }

  pub fn previous(&mut self) {
    if let Some(i) = self.state.selected() {
      self.state.select(Some(i.saturating_sub(1)));
    } else {
      self.state.select(Some(0))
    }
  }

  pub fn index(&self) -> Option<usize> {
    self.state.selected()
  }

  pub fn selected(&self) -> Option<&T> {
    self.index().and_then(move |i| self.items.get(i))
  }

  #[allow(dead_code)]
  pub fn selected_mut(&mut self) -> Option<&mut T> {
    self.index().and_then(move |i| self.items.get_mut(i))
  }

  pub fn select_index(&mut self, index: usize) {
    self.state.select(Some(index));
  }
}
