use std::{fmt::Debug, ops::Range};

use crate::TypeSymbol;

/// Any symbol, that is not a type definition
pub type Symbol = String;

pub type Module = String;
pub type Header = String;
pub type Alias = String;
pub type DyLibName = String;


#[derive(Debug, PartialEq, Clone)]
pub enum Query {}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Clone)]
pub enum AssignmentOperations {
    Identity,
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulo,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum PrefixOperator {
    Not,    // '!'
    Negate, // '-'
}

pub struct StructBody {
    pub functions: Vec<Box<AstNode>>,
    pub attributes: Vec<(Symbol, TypeSymbol)>,
}

#[derive(Debug, Clone)]
pub struct MemberAccess {
    //a.c(e,f).d
    pub member: Symbol,
    pub params: Option<Vec<Box<AstNode>>>,
    pub range: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub range: Range<usize>,
    pub type_of: AstNodeType,
}

impl AstNode {
    pub fn new(range: Range<usize>, type_of: AstNodeType) -> Self {
        Self { range, type_of }
    }
}

#[derive(Debug, Clone)]
pub enum AstNodeType {
    Import(Module, Option<Alias>),
    ImportNative(Header, DyLibName, Option<Alias>),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Box<AstNode>>),
    Map(Vec<(Box<AstNode>, Box<AstNode>)>),
    Option(Option<Box<AstNode>>),
    Result(Result<Box<AstNode>, Box<AstNode>>),
    StructInitializer {
        name: Symbol,
        values: Vec<(Symbol, Box<AstNode>)>,
    },
    Declaration {
        new_symbol: Symbol,
        expression: Box<AstNode>,
        assumed_type: Option<TypeSymbol>,
    },
    AssignmentOp {
        recipient: Symbol,
        operation: AssignmentOperations,
        expression: Box<AstNode>,
    },
    TypeDef {
        typename: Symbol,
        typedef: AstTypeDefinition,
        execution_body: Vec<Box<AstNode>>,
    },
    FunctionCall {
        function_name: Symbol,
        params: Vec<Box<AstNode>>,
    },
    InfixCall(Box<AstNode>, InfixOperator, Box<AstNode>),
    PrefixCall(PrefixOperator, Box<AstNode>),
    MemberCall {
        parent: Symbol,
        calls: Vec<MemberAccess>,
    },
    Branch {
        cond: Box<AstNode>,
        body: Vec<Box<AstNode>>,
        else_if_branches: Vec<(Box<AstNode>, Vec<Box<AstNode>>)>,
        else_branch: Option<Vec<Box<AstNode>>>,
    },
    While {
        cond: Box<AstNode>,
        body: Vec<Box<AstNode>>,
    },
    ForEach {
        recipient: Symbol,
        iterable: Box<AstNode>,
        body: Vec<Box<AstNode>>,
    },
    For {
        declaration: Option<Box<AstNode>>,
        condition: Option<Box<AstNode>>,
        assignment: Option<Box<AstNode>>,
        body: Vec<Box<AstNode>>,
    },
    ReturnStatement {
        return_value: Box<AstNode>,
    },
    Symbol(Symbol),
    Weak(Box<AstNode>),
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

pub fn apply_string_escapes(s: &str) -> String {
    let mut result = String::new();
    let s = s.chars().collect::<Vec<char>>();

    let mut i = 0;
    while i < s.len() {
        if s[i] == '\\' {
            if s[i + 1] == '\\' {
                result.push('\\');
                i += 1;
            } else if s[i + 1] == '"' {
                result.push('\"');
                i += 1;
            } else if s[i + 1] == 'n' {
                result.push('\n');
                i += 1;
            } else if s[i + 1] == 'r' {
                result.push('\n');
                i += 1;
            } else if s[i + 1] == 't' {
                result.push('\n');
                i += 1;
            }

            result.push('\\');
        } else {
            result.push(s[i]);
        }
        i += 1;
    }

    result
}
