use std::fmt::{Debug, Write};

#[macro_export]
macro_rules! err_args {
    ($($arg:expr),* $(,)?) => {
        &[$($arg),*]
    };
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    TypeInferenceNeeded = 1,
    InvalidToken = 2,
    UnexpectedEof = 3,
    DuplicateDefinition = 4,
    InvalidType = 5,
    VariableCannotBeVoid = 6,
    ExpectedToken = 7,
    Unknown = 255,
}

impl ErrorKind {
    const MESSAGES: &'static [&'static str] = &[
        "unknown error",
        "type annotations needed - cannot infer type",
        "invalid token - expected {}, found {}",
        "unexpected end of file",
        "duplicate definition of {}",
        "invalid type - expected {}, found {}",
        "variables cannot have type 'void'",
        "invalid token - expected {}",
    ];

    pub const fn template(&self) -> &'static str {
        let idx = *self as usize;
        if idx < Self::MESSAGES.len() {
            Self::MESSAGES[idx]
        } else {
            Self::MESSAGES[0]
        }
    }

    pub fn format_message(&self, args: &[impl Debug]) -> String {
        let template = self.template();
        let mut result = String::new();
        
        let mut parts = template.split("{}").peekable();
        let mut i = 0;

        while let Some(part) = parts.next() {
            result.push_str(part);
            if parts.peek().is_some() && i < args.len() {
                write!(result, "{:?}", args[i]).unwrap();
                i += 1;
            }
        }

        result
    }
}
