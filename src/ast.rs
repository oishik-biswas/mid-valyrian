use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]

pub enum Statement {
    VariableDeclaration {
        name: String,
        data_type: DataType,
        value: Expression,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    Conditional {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    ForLoop {
        count: i64,
        body: Vec<Statement>,
    },
    WhileLoop {
        condition: Expression,
        body: Vec<Statement>,
    },
    Return(Option<Expression>),
    Speak(Expression),
    MainBlock(Vec<Statement>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Scroll,  // String
    Blade,   // i64
    Wine,    // f64
    Vow,     // bool
    Sigil,   // char
    Void,    // No return
}


#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Input(String),
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Greater,
    Less,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Return(Value),
}


#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(char),
    Void,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Boolean(b) => write!(f, "{}", if *b { "aye" } else { "nay" }),
            Value::Char(c) => write!(f, "{}", c),
            Value::Void => write!(f, "void"),
        }
    }
}

impl DataType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "scroll" => Some(DataType::Scroll),
            "blade" => Some(DataType::Blade),
            "wine" => Some(DataType::Wine),
            "vow" => Some(DataType::Vow),
            "sigil" => Some(DataType::Sigil),
            "void" => Some(DataType::Void),
            _ => None,
        }
    }
}

impl BinaryOperator {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => Some(BinaryOperator::Add),
            "-" => Some(BinaryOperator::Subtract),
            "*" => Some(BinaryOperator::Multiply),
            "/" => Some(BinaryOperator::Divide),
            ">" => Some(BinaryOperator::Greater),
            "<" => Some(BinaryOperator::Less),
            "==" => Some(BinaryOperator::Equal),
            "!=" => Some(BinaryOperator::NotEqual),
            _ => None,
        }
    }
}