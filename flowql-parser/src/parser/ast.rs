#[derive(Default, Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            statements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    LetBinding { name: String, expr: Expression },
    CreatePersistent { name: String, expr: Expression },
    SetPersistent { name: String, expr: Expression },
    MigratePersistent { name: String, expr: Expression },
    DropPersistent { name: String },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(LiteralValue),
    Variable(String),
    Pipeline {
        left: Box<Expression>,
        operations: Vec<PipelineOp>,
    },
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int(u64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    LogicalOr,
    LogicalAnd,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    LogicalNot,
}

#[derive(Debug, Clone)]
pub enum PipelineOp {
    Select,
    Filter,
}

#[derive(Debug, Clone)]
pub struct SelectOperation {}
pub struct FilterOperation {}
