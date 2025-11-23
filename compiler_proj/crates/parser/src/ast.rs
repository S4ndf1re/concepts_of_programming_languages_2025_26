use std::fmt::Debug;

/// Any symbol, that is not a type definition
pub type Symbol = String;

pub type Module = String;
pub type Header = String;
pub type Alias = String;
pub type DyLibName = String;


#[derive(Debug, Clone)]
pub enum TypeSymbolType {
    Symbol(Symbol),
    List(Box<TypeSymbol>),
    Map(Box<TypeSymbol>, Box<TypeSymbol>),
    Option(Box<TypeSymbol>),
}

/// The symbol that represents any existing type
#[derive(Debug, Clone)]
pub struct TypeSymbol {
    is_weak: bool,
    type_of: TypeSymbolType,
    resolved: bool,
    inferred: bool,
}

impl TypeSymbol {
    pub fn strong(type_of: TypeSymbolType) -> Self {
        Self {
            is_weak: false,
            type_of,
            resolved: false,
            inferred: false,
        }
    }

    pub fn weak(type_of: TypeSymbolType) -> Self {
        Self {
            is_weak: true,
            type_of,
            resolved: false,
            inferred: false,
        }
    }
    pub fn make_weak(mut self) -> Self {
        self.is_weak = true;
        self
    }
}

#[derive(Debug)]
pub enum Query {}


#[derive(Debug)]
pub enum AstTypeDefinition {
    Int,
    Float,
    String,
    Bool,
    Struct(Vec<(Symbol, TypeSymbol)>),
    List(TypeSymbol),
    Map(Symbol, TypeSymbol),
    Function(Vec<(Symbol, TypeSymbol)>, Option<TypeSymbol>),
    System(Vec<(Symbol, Query)>),
    Option(TypeSymbol),
    Result(TypeSymbol, TypeSymbol),
}

#[derive(Debug)]
pub enum AssignmentOperations {
    Identity,
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulo,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum PrefixOperator {
    Not,    // '!'
    Negate, // '-'
}



pub struct StructBody {
    pub functions: Vec<Box<AstNode>>,
    pub attributes: Vec<(Symbol, TypeSymbol)>,
}

#[derive(Debug)]
pub struct MemberAccess { //a.c(e,f).d
    pub member: Symbol,
    pub params: Option<Vec<Box<AstNode>>>,
}

#[derive(Debug)]
pub enum AstNode {
    Import(Module, Option<Alias>),
    ImportNative(Header, DyLibName, Option<Alias>),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Box<AstNode>>),
    Map(Vec<(Box<AstNode>, Box<AstNode>)>),
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
        else_branch: Option<Vec<Box<AstNode>>>
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
