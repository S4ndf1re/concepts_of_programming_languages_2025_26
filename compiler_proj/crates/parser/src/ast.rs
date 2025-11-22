use std::fmt::Debug;

/// Any symbol, that is not a type definition
pub type Symbol = String;

pub type Module = String;
pub type Header = String;
pub type Alias = String;
pub type DyLibName = String;

/// The symbol that represents any existing type
pub struct TypeSymbol {
    name: String,
    resolved: bool,
    inferred: bool,
}

impl TypeSymbol {
    pub fn new(name: String) -> Self {
        Self {
            name,
            resolved: false,
            inferred: false,
        }
    }
}

pub enum Query {}

pub enum AstTypeDefinition {
    Int,
    Float,
    String,
    Bool,
    Struct(Vec<(Symbol, TypeSymbol)>),
    List(TypeSymbol),
    Map(TypeSymbol, TypeSymbol),
    Function(Vec<(Symbol, TypeSymbol)>, Option<TypeSymbol>),
    System(Vec<(Symbol, Query)>),
    Option(TypeSymbol),
    Result(TypeSymbol, TypeSymbol),
}

pub enum AssignmentOperations {
    Identity,
    Plus,
    Minus,
    Divide,
    Multiply,
    Modulo,
}

pub enum InfixOperator {
    // Computation
    Plus,
    Minus,
    Divide,
    Multiply,
    Modulo,
    // Comparison
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    // Logical
    And,
    Or,
}

pub enum PrefixOperator {
    Not,    // '!'
    Negate, // '-'
}

pub enum AstNode {
    Import(Module, Option<Alias>),
    ImportNative(Header, DyLibName, Option<Alias>),
    Declaration {
        new_symbol: Symbol,
        expression: Box<AstNode>,
    },
    AssignmentOp {
        recipient: Symbol,
        operation: AssignmentOperations,
        expression: Box<AstNode>,
    },
    TypeDef {
        typename: Symbol,
        typedef: AstTypeDefinition,
        execution_body: Option<Box<AstNode>>,
    },
    FunctionCall {
        params: Vec<Box<AstNode>>,
    },
    InfixCall(Box<AstNode>, InfixOperator, Box<AstNode>),
    PrefixCall(PrefixOperator, Box<AstNode>),
    MethodCall {
        caller: Symbol,
        method: Symbol,
        params: Vec<Box<AstNode>>,
    },

    Branch {
        cond: Box<AstNode>, 
        body: Option<Box<AstNode>>,
        else_if_branches: Vec<(Box<AstNode>, Option<Box<AstNode>>)>,
        else_branch: Option<Box<AstNode>>
    },
    Symbol(Symbol),

}

pub enum Expr {
    Number(i32),
    Op(Box<Expr>, OpCode, Box<Expr>),
}

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Number(i) => write!(f, "{i}"),
            Expr::Op(l, op, r) => write!(f, "({:?} {:?} {:?})", l, op, r),
        }
    }
}

pub enum OpCode {
    Mul,
    Div,
    Add,
    Sub,
}

impl Debug for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Mul => write!(f, "*"),
            OpCode::Div => write!(f, "/"),
            OpCode::Add => write!(f, "+"),
            OpCode::Sub => write!(f, "-"),
        }
    }
}
