use core::cell::RefCell;

pub trait AppLog {
    fn log(&self, message: String);
}

#[derive(Default)]
pub struct NullLog {}

impl AppLog for NullLog {
    fn log(&self, _message: String) {}
}

#[derive(Default)]
pub struct StringLog {
    messages: RefCell<String>,
}

impl AppLog for StringLog {
    fn log(&self, message: String) {
        let updated_messages = format!("{}{} ", self.messages.borrow(), message);
        *self.messages.borrow_mut() = updated_messages;
    }
}

impl StringLog {
    pub fn _get_all(&self) -> String {
        self.messages.borrow().clone()
    }
}
