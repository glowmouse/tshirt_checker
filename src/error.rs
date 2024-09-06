//
// A simple error class for any image import problems
//

// Types of errors
//
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ErrorTypes {
    FileImportAborted,
    ImageLoadError,
}

// Error class (error id and a user facing message describing what went wrong)
//
#[derive(Debug)]
pub struct Error {
    err_id: ErrorTypes,
    err_msg: String,
}

impl Error {
    // Constructor
    pub fn new(id: ErrorTypes, msg: impl Into<String>) -> Self {
        Self {
            err_id: id,
            err_msg: msg.into(),
        }
    }
    // ID getter
    pub fn id(&self) -> ErrorTypes {
        self.err_id
    }
    // User facing message that describes what went wrong.
    pub fn msg(&self) -> String {
        self.err_msg.clone()
    }
}
