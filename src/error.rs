use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValyrianError {
    #[error("🐉 The Maester's scroll contains errors: {0}")] ParseError(String),

    #[error("⚔️ Runtime Terror in the Seven Kingdoms: {0}")] RuntimeError(String),

    #[error("🏰 Variable '{0}' is not known in this realm")] UndefinedVariable(String),

    #[error("🗡️ Function '{0}' has not been declared by the council")] UndefinedFunction(String),

    #[error("🍷 Type mismatch: Expected {expected}, found {found}")] TypeError {
        expected: String,
        found: String,
    },

    #[error("❄️ The Night King has entered your call stack (division by zero)")]
    DivisionByZero,

    #[error("🔥 Dracarys! Your program has been consumed by flames: {0}")] IoError(String),

    #[error("👑 The Iron Throne demands better syntax: {0}")] SyntaxError(String),

    #[error(
        "🧙‍♂️ The Red Priest miscounted the offerings — expected a different number of arguments"
    )]
    ArgumentMismatch,

    #[error(
        "🏹 Arrows must fly true: Invalid operation {op} on {left_type} and {right_type}"
    )] InvalidOperation {
        op: String,
        left_type: String,
        right_type: String,
    },
}

impl From<std::io::Error> for ValyrianError {
    fn from(error: std::io::Error) -> Self {
        ValyrianError::IoError(error.to_string())
    }
}

impl ValyrianError {
    pub fn type_error(expected: &str, found: &str) -> Self {
        ValyrianError::TypeError {
            expected: expected.to_string(),
            found: found.to_string(),
        }
    }

    pub fn invalid_operation(op: &str, left_type: &str, right_type: &str) -> Self {
        ValyrianError::InvalidOperation {
            op: op.to_string(),
            left_type: left_type.to_string(),
            right_type: right_type.to_string(),
        }
    }
}
