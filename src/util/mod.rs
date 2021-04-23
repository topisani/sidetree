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

  pub fn with_items(items: Vec<T>) -> StatefulList<T> {
    StatefulList {
      state: ListState::default(),
      items,
    }
  }

  pub fn next(&mut self) {
    if let Some(i) = self.state.selected() {
      if i + 1 < self.items.len() {
        self.state.select(Some(i + 1));
      }
    } else {
      self.state.select(Some(0))
    }
  }

  pub fn previous(&mut self) {
    if let Some(i) = self.state.selected() {
      if i > 0 {
        self.state.select(Some(i - 1));
      }
    } else {
      self.state.select(Some(0))
    }
  }

  pub fn unselect(&mut self) {
    self.state.select(None);
  }

  pub fn index(&self) -> Option<usize> {
    self.state.selected()
  }

  pub fn selected(&self) -> Option<&T> {
    self
      .index()
      .and_then(move |i| self.items.get(i))
  }
  pub fn selected_mut(&mut self) -> Option<&mut T> {
    self
      .index()
      .and_then(move |i| self.items.get_mut(i))
  }
}
