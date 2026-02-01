use crate::form_item::FormItem;

/// Zarządza stanem formularza - focused input, nawigacja
pub struct FormState {
    items: Vec<FormItem>,
    focused_index: usize,
}

impl FormState {
    pub fn new(items: Vec<FormItem>) -> Self {
        let mut state = Self {
            items,
            focused_index: 0,
        };

        // Ustaw pierwszy input jako focused
        state.focus_first();
        state
    }

    pub fn items(&self) -> &[FormItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut [FormItem] {
        &mut self.items
    }

    pub fn focused_index(&self) -> usize {
        self.focused_index
    }

    pub fn focused_item(&self) -> Option<&FormItem> {
        self.items.get(self.focused_index)
    }

    pub fn focused_item_mut(&mut self) -> Option<&mut FormItem> {
        self.items.get_mut(self.focused_index)
    }

    /// Przejdź do następnego inputa
    pub fn focus_next(&mut self) -> bool {
        self.unfocus_current();

        let start_index = self.focused_index;
        loop {
            self.focused_index = (self.focused_index + 1) % self.items.len();

            if self.items[self.focused_index].is_input() {
                self.focus_current();
                return true;
            }

            if self.focused_index == start_index {
                // Nie znaleziono następnego - wróć do obecnego
                self.focus_current();
                return false;
            }
        }
    }

    /// Przejdź do poprzedniego inputa
    pub fn focus_prev(&mut self) -> bool {
        self.unfocus_current();

        let start_index = self.focused_index;
        loop {
            self.focused_index = if self.focused_index == 0 {
                self.items.len() - 1
            } else {
                self.focused_index - 1
            };

            if self.items[self.focused_index].is_input() {
                self.focus_current();
                return true;
            }

            if self.focused_index == start_index {
                self.focus_current();
                return false;
            }
        }
    }

    /// Spróbuj przejść do następnego inputa (używane przy Submit)
    /// Zwraca true jeśli znaleziono następny input
    pub fn try_next(&mut self) -> bool {
        let current = self.focused_index;
        if self.focus_next() && self.focused_index != current {
            true
        } else {
            false
        }
    }

    /// Sprawdź czy obecny input jest poprawny
    pub fn validate_current(&self) -> Result<(), String> {
        self.items[self.focused_index]
            .as_input()
            .map(|input| input.validate())
            .unwrap_or(Ok(()))
    }

    /// Waliduj wszystkie inputy
    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let errors: Vec<String> = self
            .items
            .iter()
            .filter_map(|item| item.as_input())
            .filter_map(|input| input.validate().err())
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Aktualizuj timery błędów
    pub fn update_error_timers(&mut self) -> bool {
        self.items
            .iter_mut()
            .filter_map(|item| item.as_input_mut())
            .map(|input| input.update_error_timer())
            .any(|changed| changed)
    }

    fn focus_first(&mut self) {
        for (i, item) in self.items.iter_mut().enumerate() {
            if let Some(input) = item.as_input_mut() {
                input.set_focused(true);
                self.focused_index = i;
                return;
            }
        }
    }

    fn focus_current(&mut self) {
        if let Some(input) = self.items[self.focused_index].as_input_mut() {
            input.set_focused(true);
        }
    }

    fn unfocus_current(&mut self) {
        if let Some(input) = self.items[self.focused_index].as_input_mut() {
            input.set_focused(false);
        }
    }
}
