#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ErrorTypes {
    FileImportAborted,
    ImageLoadError,
}

#[derive(Debug)]
pub struct Error {
    pub id: ErrorTypes,
    pub msg: String,
}

impl Error {
    pub fn new(id: ErrorTypes, msg: impl Into<String>) -> Self {
        Self {
            id,
            msg: msg.into(),
        }
    }
    pub fn msg(&self) -> String {
        self.msg.clone()
    }
}
