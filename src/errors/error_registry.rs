use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

use crate::utilities::positions_and_ranges::CustomRange;

/// Should not be directly used.
pub struct Error {
    pub message: String,
    pub range: CustomRange,
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub code_number: i32,
}

impl Error {
    pub fn new(
        message: String,
        range: CustomRange,
        severity: DiagnosticSeverity,
        code: String,
        code_number: i32,
    ) -> Self {
        Self {
            message,
            range,
            severity,
            code,
            code_number,
        }
    }
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            range: self.range.to_range(),
            severity: Some(self.severity),
            code: Some(NumberOrString::String(format!(
                "{}: {}",
                self.code_number, self.code
            ))),
            source: Some(String::from("Mythic Language Server")),
            message: self.message.clone(),
            related_information: None,
            tags: None,
            data: None,
            code_description: None,
        }
    }
}

/// A macro to make it easier to create new error types.
macro_rules! error_struct {
    ($name:ident, $code_number:expr, $code:expr, $message:expr) => {
        pub struct $name {
            pub range: CustomRange,
            pub message: String,
        }

        impl $name {
            pub fn new(range: CustomRange) -> Self {
                Self {
                    range,
                    message: String::from($message),
                }
            }
            pub fn to_error(&self) -> Error {
                Error::new(
                    self.message.clone(),
                    self.range,
                    DiagnosticSeverity::ERROR,
                    String::from($code),
                    $code_number,
                )
            }
        }
    };
    // This is the same as the above, but the mesasge takes in a function.
    // the function parameters get added to the new function.
    // e.g. error_struct!(MyError, 1, "MY_ERROR", |a, b| format!("{} {}", a, b));
    //      MyError::new(range, "Hello", "World");
    // e.g. error_struct!(MyError, 2, "MY_ERROR", |a, b| {
    //          let c = a + b;
    //          format!("{} {}", a, b)
    //      });
    //      MyError::new(range, "Hello", "world")
    ($name:ident, $code_number:expr, $code:expr, $message:expr, $($param:ident),+) => {
        pub struct $name {
            pub range: CustomRange,
            pub message: String,
        }

        impl $name {
            pub fn new(range: CustomRange, $($param: String),+) -> Self {
                Self {
                    range,
                    message: $message($($param),+),
                }
            }
            pub fn to_error(&self) -> Error {
                Error::new(
                    self.message.clone(),
                    self.range,
                    DiagnosticSeverity::ERROR,
                    String::from($code),
                    $code_number,
                )
            }
        }
    };

    ($name:ident, $code_number:expr, $code:expr) => {
        pub struct $name {
            pub range: CustomRange,
            pub message: String,
        }

        impl $name {
            pub fn new(range: CustomRange, message: String) -> Self {
                Self {
                    range,
                    message,
                }
            }
            pub fn to_error(&self) -> Error {
                Error::new(
                    self.message.clone(),
                    self.range,
                    DiagnosticSeverity::ERROR,
                    String::from($code),
                    $code_number,
                )
            }
        }
    };
}

error_struct!(SyntaxError, 0, "syntax_error");
error_struct!(
    TargeterAlreadyDefinedError,
    1,
    "targeter_already_defined_error",
    "The targeter is already defined for this skill line!"
);
error_struct!(
    TriggerAlreadyDefinedError,
    2,
    "trigger_already_defined_error",
    "The trigger is already defined for this skill line!"
);
error_struct!(
    InvalidConfigurationFileStructureError,
    3,
    "invalid_configuration_file_structure_error",
    |got, expected| format!(
        "Invalid configuration file structure. Expected {}, got {}",
        expected, got
    ),
    got,
    expected
);

