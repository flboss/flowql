pub struct Program<'a> {
    statements: Vec<Statement<'a>>,
}

pub enum Statement<'a> {
    LetBinding {
        name: &'a str,
        expr: Expression<'a>,
    },
    Assignment {
        left: Expression<'a>,
        right: Expression<'a>,
    },
    CreatePersistent {
        name: &'a str,
        expr: Expression<'a>,
    },
    SetPersistent {
        name: &'a str,
        expr: Expression<'a>,
    },
    MigratePersistent {
        name: &'a str,
        expr: Expression<'a>,
    },
    DropPersistent {
        name: &'a str,
    },
}

pub enum Expression<'a> {
    Literal(LiteralValue),
    Variable(&'a str),
    Pipeline {
        left: Box<Expression<'a>>,
        operations: Vec<PipelineOp>,
    },
    BinaryOp {
        left: Box<Expression<'a>>,
        op: BinaryOperator,
        right: Box<Expression<'a>>,
    },
    UnaryOp {
        op: UnaryOperator,
        right: Box<Expression<'a>>,
    },
    FunctionCall {
        name: &'a str,
        args: Vec<Expression<'a>>,
    },
}

pub enum LiteralValue {
    Int(u64),
    Float(f64),
    String(String),
}

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

pub enum UnaryOperator {
    Negate,
    LogicalNot,
}

pub enum PipelineOp {}
