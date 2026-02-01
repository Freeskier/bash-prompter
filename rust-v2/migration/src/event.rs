#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Exit,
    Submit,
    NextInput,
    PrevInput,
    DeleteWord,        // Ctrl+Backspace
    DeleteWordForward, // Ctrl+Delete
}
