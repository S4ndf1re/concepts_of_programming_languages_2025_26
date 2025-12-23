use std::{fmt::Debug, ops::Range};

use graphviz_rust::{
    dot_generator::{attr, edge, id, node},
    dot_structures::{Attribute, Edge, EdgeTy, Graph, Id, Node, NodeId, Stmt, Vertex},
};
use rand::distr::{Alphabetic, SampleString};

use crate::TypeSymbol;

/// Any symbol, that is not a type definition
pub type Symbol = String;

pub type Module = String;
pub type Header = String;
pub type Alias = String;
pub type DyLibName = String;

pub trait ToGraphviz {
    fn to_graphviz(&self, graph: &mut Graph) -> Node;
    fn new_id(&self) -> String {
        Alphabetic.sample_string(&mut rand::rng(), 32)
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum GroupSystem {
    Single(Symbol),
    Ordered(Symbol, Symbol),
}

#[derive(Debug, PartialEq, Clone)]
pub enum RegisterType {
    Chain(Vec<Symbol>),
    After(Symbol, Symbol),
    Before(Symbol, Symbol),
}


#[derive(Debug, PartialEq, Clone, Hash)]
pub struct QueryTerm {
    pub components: Vec<Symbol>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum QueryCond {
    Component(Symbol),
    Not(Box<QueryCond>),
    And(Box<QueryCond>, Box<QueryCond>),
    Or(Box<QueryCond>, Box<QueryCond>),
}

impl QueryCond {
    pub fn get_dependent_symbols(&self) -> Vec<&Symbol> {
        match self {
            QueryCond::Component(s) => vec![s],
            QueryCond::Not(cond) => cond.get_dependent_symbols(),
            QueryCond::And(c1, c2) => c1
                .get_dependent_symbols()
                .into_iter()
                .chain(c2.get_dependent_symbols())
                .collect::<Vec<_>>(),
            QueryCond::Or(c1, c2) => c1
                .get_dependent_symbols()
                .into_iter()
                .chain(c2.get_dependent_symbols())
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum QueryType {
    List {
        select: QueryTerm,
        condition: Option<QueryCond>,
    },
    Single {
        select: QueryTerm,
        condition: Option<QueryCond>,
    },
    World,
    Resource(Symbol),
    EventReader(Symbol),
    EventWriter(Symbol),
}

impl QueryType {
    pub fn get_dependent_symbols(&self) -> Vec<&Symbol> {
        let mut result = Vec::new();
        match self {
            QueryType::List { select, condition } => {
                for symbol in &select.components {
                    result.push(symbol);
                }
                if let Some(cond) = condition {
                    result.extend(cond.get_dependent_symbols());
                }
            }
            QueryType::Single { select, condition } => {
                for symbol in &select.components {
                    result.push(symbol);
                }
                if let Some(cond) = condition {
                    result.extend(cond.get_dependent_symbols());
                }
            }
            QueryType::Resource(res) => result.push(res),
            QueryType::EventReader(evt) | QueryType::EventWriter(evt) => result.push(evt),
            _ => (),
        }

        result
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Query {
    pub symbol: Symbol,
    pub type_of: QueryType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstTypeDefinition {
    Int,
    Float,
    String,
    Bool,
    Struct(Vec<(Symbol, TypeSymbol)>),
    Component(Vec<(Symbol, TypeSymbol)>),
    List(TypeSymbol),
    Map(TypeSymbol, TypeSymbol),
    Function(Vec<(Symbol, TypeSymbol)>, Option<TypeSymbol>),
    System(Vec<(Symbol, Symbol)>, Option<Vec<Query>>),
    Option(TypeSymbol),
    Result(TypeSymbol, TypeSymbol),
}

impl ToGraphviz for AstTypeDefinition {
    fn to_graphviz(&self, graph: &mut Graph) -> Node {
        let mut n = node!(self.new_id());
        let mut edges = Vec::new();

        let attrs = match self {
            AstTypeDefinition::Int => vec![attr!("label", "int")],
            AstTypeDefinition::Float => vec![attr!("label", "float")],
            AstTypeDefinition::String => vec![attr!("label", "string")],
            AstTypeDefinition::Bool => vec![attr!("label", "bool")],
            AstTypeDefinition::Struct(items) => {
                for item in items {
                    let member = node!(self.new_id(); attr!("label", item.0));
                    graph.add_stmt(Stmt::Node(member.clone()));
                    edges.push(edge!(n.id.clone() => member.id.clone()));

                    let type_of = item.1.to_graphviz(graph);
                    edges.push(edge!(member.id.clone() => type_of.id.clone()));
                }

                vec![attr!("label", "struct")]
            }
            AstTypeDefinition::Component(items) => {
                for item in items {
                    let member = node!(self.new_id(); attr!("label", item.0));
                    graph.add_stmt(Stmt::Node(member.clone()));
                    edges.push(edge!(n.id.clone() => member.id.clone()));

                    let type_of = item.1.to_graphviz(graph);
                    edges.push(edge!(member.id.clone() => type_of.id.clone()));
                }

                vec![attr!("label", "component")]
            }
            AstTypeDefinition::List(type_symbol) => {
                let type_of = type_symbol.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => type_of.id.clone()));
                vec![attr!("label", "list")]
            }
            AstTypeDefinition::Map(key, val) => {
                let type_of = key.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => type_of.id.clone()));

                let type_of = val.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => type_of.id.clone()));
                vec![attr!("label", "map")]
            }
            AstTypeDefinition::Function(items, return_type) => {
                for item in items {
                    let member = node!(self.new_id(); attr!("label", item.0));
                    graph.add_stmt(Stmt::Node(member.clone()));
                    edges.push(edge!(n.id.clone() => member.id.clone()));

                    let type_of = item.1.to_graphviz(graph);
                    edges.push(edge!(member.id.clone() => type_of.id.clone()));
                }

                if let Some(ret_type) = return_type {
                    let member = node!(self.new_id(); attr!("label", "return"));
                    graph.add_stmt(Stmt::Node(member.clone()));
                    edges.push(edge!(n.id.clone() => member.id.clone()));

                    let type_of = ret_type.to_graphviz(graph);
                    edges.push(edge!(member.id.clone() => type_of.id.clone()));
                }

                vec![attr!("label", "function")]
            }
            AstTypeDefinition::System(_items, _query) => {
                // TODO: implemented yet
                vec![attr!("label", "system")]
            }
            AstTypeDefinition::Option(type_symbol) => {
                let type_of = type_symbol.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => type_of.id.clone()));
                vec![attr!("label", "option")]
            }
            AstTypeDefinition::Result(ok_val, err_val) => {
                let type_of = ok_val.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => type_of.id.clone()));

                let type_of = err_val.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => type_of.id.clone()));

                vec![attr!("label", "result")]
            }
        };

        n.attributes = attrs;
        graph.add_stmt(Stmt::Node(n.clone()));
        for e in edges {
            graph.add_stmt(Stmt::Edge(e));
        }

        n
    }
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

pub struct ComponentBody {
    pub attributes: Vec<(Symbol, TypeSymbol)>,
}

#[derive(Debug, Clone)]
pub enum MemberAccessType {
    Symbol,
    Function(Vec<Box<AstNode>>),
    Struct(Vec<(Symbol, Box<AstNode>)>),
}

#[derive(Debug, Clone)]
pub struct MemberAccess {
    //a.c(e,f).d
    pub member: Symbol,
    pub type_of: MemberAccessType,
    pub range: Range<usize>,
}

impl ToGraphviz for MemberAccess {
    fn to_graphviz(&self, graph: &mut Graph) -> Node {
        let mut n = node!(self.new_id());
        let mut edges = Vec::new();

        let attrs = match &self.type_of {
            MemberAccessType::Symbol => {
                vec![attr!("label", &format!("\"Symbol({})\"", self.member))]
            }
            MemberAccessType::Function(ast_nodes) => {
                let param_node = node!(self.new_id(); attr!("label", "parameters"));
                graph.add_stmt(Stmt::Node(param_node.clone()));
                edges.push(edge!(n.id.clone() => param_node.id.clone()));

                for node in ast_nodes {
                    let node = node.to_graphviz(graph);
                    edges.push(edge!(param_node.id.clone() => node.id.clone()));
                }

                vec![attr!("label", &format!("\"Function({})\"", self.member))]
            }
            MemberAccessType::Struct(items) => {
                let param_node = node!(self.new_id(); attr!("label", "attributes"));
                edges.push(edge!(n.id.clone() => param_node.id.clone()));

                for node in items {
                    let attr_node = node!(self.new_id(); attr!("label", node.0));
                    edges.push(edge!(param_node.id.clone() => attr_node.id.clone()));

                    let node = node.1.to_graphviz(graph);
                    edges.push(edge!(attr_node.id.clone() => node.id.clone()));
                }

                vec![attr!("label", &format!("\"Struct({})\"", self.member))]
            }
        };

        n.attributes = attrs;

        graph.add_stmt(Stmt::Node(n.clone()));
        for e in edges {
            graph.add_stmt(Stmt::Edge(e));
        }
        n
    }
}

impl ToGraphviz for Vec<MemberAccess> {
    fn to_graphviz(&self, graph: &mut Graph) -> Node {
        let mut first_node = None;
        let mut last_node: Option<Node> = None;
        let mut edges = Vec::new();

        for ma in self {
            let node = ma.to_graphviz(graph);

            if first_node.is_none() {
                first_node = Some(node.clone());
            }

            if let Some(last_node) = &last_node {
                let binding_node = node!(self.new_id(); attr!("label", "dot"));
                graph.add_stmt(Stmt::Node(binding_node.clone()));
                edges.push(edge!(last_node.id.clone() => binding_node.id.clone()));
                edges.push(edge!(binding_node.id.clone() => node.id.clone()));
            }

            last_node = Some(node.clone());
        }

        for e in edges {
            graph.add_stmt(Stmt::Edge(e));
        }

        if let Some(first) = first_node {
            first
        } else {
            panic!("unsupported conversion")
        }
    }
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
    Declaration {
        new_symbol: Symbol,
        expression: Box<AstNode>,
        assumed_type: Option<TypeSymbol>,
    },
    EntityDeclaration {
        new_symbol: Symbol,
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
    InfixCall(Box<AstNode>, InfixOperator, Box<AstNode>),
    PrefixCall(PrefixOperator, Box<AstNode>),
    MemberCall {
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
    GroupDef {
        systems: Vec<GroupSystem>,
    },
    Register {
        /// Can be both groups and systems
        schedule_entity: RegisterType,
    },
    EntityDef {
        name: Symbol,
        default_components: Option<Vec<AstNode>>,
    },
    Weak(Box<AstNode>),
    // TODO: Break statement in loops,
}

impl ToGraphviz for AstNode {
    fn to_graphviz(&self, graph: &mut Graph) -> Node {
        let mut n = node!(self.new_id());

        let mut edges = Vec::new();

        let attrs = match &self.type_of {
            AstNodeType::Import(_, _) => vec![attr!("label", "\"import\"")],
            AstNodeType::ImportNative(_, _, _) => vec![attr!("label", "\"import_native\"")],
            AstNodeType::Int(i) => vec![attr!("label", &format!("\"int({i})\""))],
            AstNodeType::Float(f) => vec![attr!("label", &format!("\"float({f})\""))],
            AstNodeType::String(s) => {
                vec![attr!("label", &format!("\"string({s})\""))]
            }
            AstNodeType::Bool(b) => vec![attr!("label", &format!("\"bool({b})\""))],
            AstNodeType::List(ast_nodes) => {
                for node in ast_nodes {
                    let n_child = node.to_graphviz(graph);
                    edges.push(edge!(n.id.clone() => n_child.id.clone()));
                }
                vec![attr!("label", "list")]
            }
            AstNodeType::Map(items) => {
                for (k, v) in items {
                    let n_key = node!(self.new_id(); attr!("label", "entry"));
                    graph.add_stmt(Stmt::Node(n_key.clone()));

                    let n_k = k.to_graphviz(graph);
                    let n_v = v.to_graphviz(graph);

                    edges.push(edge!(n.id.clone() => n_key.id.clone()));
                    edges.push(edge!(n_key.id.clone() => n_k.id.clone()));
                    edges.push(edge!(n_key.id.clone() => n_v.id.clone()));
                }
                vec![attr!("label", "map")]
            }
            AstNodeType::Option(ast_node) => {
                if let Some(ast_node) = ast_node {
                    let n_child = ast_node.to_graphviz(graph);
                    edges.push(edge!(n.id.clone() => n_child.id.clone()));
                    vec![attr!("label", "option_some")]
                } else {
                    vec![attr!("label", "option_none")]
                }
            }
            AstNodeType::Result(ast_node) => {
                if let Ok(ast_node) = ast_node {
                    let n_child = ast_node.to_graphviz(graph);
                    edges.push(edge!(n.id.clone() => n_child.id.clone()));
                    vec![attr!("label", "result_ok")]
                } else {
                    let n_child = ast_node.as_ref().err().unwrap().to_graphviz(graph);
                    edges.push(edge!(n.id.clone() => n_child.id.clone()));
                    vec![attr!("label", "result_err")]
                }
            }
            AstNodeType::Declaration {
                new_symbol,
                expression: ast_node,
                assumed_type: _,
            } => {
                let n_child = ast_node.as_ref().to_graphviz(graph);
                edges.push(edge!(n.id.clone() => n_child.id.clone()));

                vec![attr!(
                    "label",
                    &format!("\"declaration(new_symbol: {new_symbol})\"")
                )]
            }
            AstNodeType::EntityDeclaration { new_symbol } => vec![attr!(
                "label",
                &format!("\"entity_declaration(new_entity: {new_symbol})\"")
            )],
            AstNodeType::AssignmentOp {
                recipient,
                operation,
                expression: ast_node,
            } => {
                let n_child = ast_node.as_ref().to_graphviz(graph);
                edges.push(edge!(n.id.clone() => n_child.id.clone()));

                vec![attr!(
                    "label",
                    &format!("\"assignment(op: {operation:?}, recipient: {recipient})\"")
                )]
            }
            AstNodeType::TypeDef {
                typename,
                typedef,
                execution_body,
            } => {
                let type_node = typedef.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => type_node.id.clone()));

                let exec_node = node!(self.new_id(); attr!("label", "execution_body"));
                graph.add_stmt(Stmt::Node(exec_node.clone()));
                edges.push(edge!(n.id.clone() => exec_node.id.clone()));

                for stmt in execution_body {
                    let stmt_node = stmt.to_graphviz(graph);
                    edges.push(edge!(exec_node.id.clone() => stmt_node.id.clone()));
                }

                vec![attr!(
                    "label",
                    &format!("\"typedef(typename: {typename})\"")
                )]
            }
            AstNodeType::InfixCall(left, infix_operator, right) => {
                let n_child = left.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => n_child.id.clone()));

                let n_child = right.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => n_child.id.clone()));

                vec![attr!(
                    "label",
                    &format!("\"infix_call(op: {infix_operator:?})\"")
                )]
            }
            AstNodeType::PrefixCall(prefix_operator, ast_node) => {
                let n_child = ast_node.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => n_child.id.clone()));

                vec![attr!(
                    "label",
                    &format!("\"prefix_call(op: {prefix_operator:?})\"")
                )]
            }
            AstNodeType::MemberCall { calls } => {
                let node = calls.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => node.id.clone()));
                vec![attr!("label", "member_call")]
            }
            AstNodeType::Branch {
                cond,
                body,
                else_if_branches,
                else_branch,
            } => {
                let if_node = node!(self.new_id(); attr!("label", "if"));
                graph.add_stmt(Stmt::Node(if_node.clone()));
                edges.push(edge!(n.id.clone() => if_node.id.clone()));

                let cond_node = node!(self.new_id(); attr!("label", "condition"));
                graph.add_stmt(Stmt::Node(cond_node.clone()));
                edges.push(edge!(if_node.id.clone() => cond_node.id.clone()));

                let n_child = cond.to_graphviz(graph);
                edges.push(edge!(cond_node.id.clone() => n_child.id.clone()));

                let body_node = node!(self.new_id(); attr!("label", "body"));
                graph.add_stmt(Stmt::Node(body_node.clone()));
                edges.push(edge!(if_node.id.clone() => body_node.id.clone()));

                for expr in body {
                    let expr_node = expr.to_graphviz(graph);
                    edges.push(edge!(body_node.id.clone() => expr_node.id.clone()));
                }

                for elif in else_if_branches {
                    let elif_node = node!(self.new_id(); attr!("label", "else_if"));
                    graph.add_stmt(Stmt::Node(elif_node.clone()));
                    edges.push(edge!(n.id.clone() => elif_node.id.clone()));

                    let cond_node = node!(self.new_id(); attr!("label", "condition"));
                    graph.add_stmt(Stmt::Node(cond_node.clone()));
                    edges.push(edge!(elif_node.id.clone() => cond_node.id.clone()));

                    let n_child = elif.0.to_graphviz(graph);
                    edges.push(edge!(cond_node.id.clone() => n_child.id.clone()));

                    let body_node = node!(self.new_id(); attr!("label", "body"));
                    graph.add_stmt(Stmt::Node(body_node.clone()));
                    edges.push(edge!(elif_node.id.clone() => body_node.id.clone()));

                    for expr in &elif.1 {
                        let expr_node = expr.to_graphviz(graph);
                        edges.push(edge!(body_node.id.clone() => expr_node.id.clone()));
                    }
                }

                if let Some(el) = else_branch {
                    let el_node = node!(self.new_id(); attr!("label", "else"));
                    graph.add_stmt(Stmt::Node(el_node.clone()));
                    edges.push(edge!(n.id.clone() => el_node.id.clone()));

                    for expr in el {
                        let expr_node = expr.to_graphviz(graph);
                        edges.push(edge!(el_node.id.clone() => expr_node.id.clone()));
                    }
                }

                vec![attr!("label", "branch")]
            }
            AstNodeType::While { cond, body } => {
                let cond_node = node!(self.new_id(); attr!("label", "condition"));
                graph.add_stmt(Stmt::Node(cond_node.clone()));
                edges.push(edge!(n.id.clone() => cond_node.id.clone()));

                let n_child = cond.to_graphviz(graph);
                edges.push(edge!(cond_node.id.clone() => n_child.id.clone()));

                let body_node = node!(self.new_id(); attr!("label", "body"));
                graph.add_stmt(Stmt::Node(body_node.clone()));
                edges.push(edge!(n.id.clone() => body_node.id.clone()));

                for expr in body {
                    let expr_node = expr.to_graphviz(graph);
                    edges.push(edge!(body_node.id.clone() => expr_node.id.clone()));
                }

                vec![attr!("label", "while")]
            }
            AstNodeType::ForEach {
                recipient,
                iterable,
                body,
            } => {
                let body_node = node!(self.new_id(); attr!("label", "iterable"));
                graph.add_stmt(Stmt::Node(body_node.clone()));
                edges.push(edge!(n.id.clone() => body_node.id.clone()));

                let iter_node = iterable.to_graphviz(graph);
                edges.push(edge!(body_node.id.clone() => iter_node.id.clone()));

                let body_node = node!(self.new_id(); attr!("label", "body"));
                graph.add_stmt(Stmt::Node(body_node.clone()));
                edges.push(edge!(n.id.clone() => body_node.id.clone()));

                for expr in body {
                    let expr_node = expr.to_graphviz(graph);
                    edges.push(edge!(body_node.id.clone() => expr_node.id.clone()));
                }

                vec![attr!(
                    "label",
                    &format!("\"foreach(recipient: {recipient})\"")
                )]
            }
            AstNodeType::For {
                declaration,
                condition,
                assignment,
                body,
            } => {
                if let Some(decl) = declaration {
                    let decl_node = node!(self.new_id(); attr!("label", "declaration"));
                    graph.add_stmt(Stmt::Node(decl_node.clone()));
                    edges.push(edge!(n.id.clone() => decl_node.id.clone()));

                    let decl = decl.to_graphviz(graph);
                    edges.push(edge!(decl_node.id.clone() => decl.id.clone()));
                }

                if let Some(cond) = condition {
                    let cond_node = node!(self.new_id(); attr!("label", "condition"));
                    graph.add_stmt(Stmt::Node(cond_node.clone()));
                    edges.push(edge!(n.id.clone() => cond_node.id.clone()));

                    let cond = cond.to_graphviz(graph);
                    edges.push(edge!(cond_node.id.clone() => cond.id.clone()));
                }

                if let Some(assign) = assignment {
                    let assign_node = node!(self.new_id(); attr!("label", "assignment"));
                    graph.add_stmt(Stmt::Node(assign_node.clone()));
                    edges.push(edge!(n.id.clone() => assign_node.id.clone()));

                    let assign = assign.to_graphviz(graph);
                    edges.push(edge!(assign_node.id.clone() => assign.id.clone()));
                }

                let body_node = node!(self.new_id(); attr!("label", "body"));
                graph.add_stmt(Stmt::Node(body_node.clone()));
                edges.push(edge!(n.id.clone() => body_node.id.clone()));

                for expr in body {
                    let expr_node = expr.to_graphviz(graph);
                    edges.push(edge!(body_node.id.clone() => expr_node.id.clone()));
                }

                vec![attr!("label", "for")]
            }
            AstNodeType::ReturnStatement { return_value } => {
                let expr_node = return_value.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => expr_node.id.clone()));
                vec![attr!("label", "return")]
            }
            AstNodeType::Weak(ast_node) => {
                let expr_node = ast_node.to_graphviz(graph);
                edges.push(edge!(n.id.clone() => expr_node.id.clone()));
                vec![attr!("label", "weak")]
            },
            _ => vec![attr!("label", "groupDef")]
,
        };

        n.attributes = attrs;

        graph.add_stmt(Stmt::Node(n.clone()));
        for edge in edges {
            graph.add_stmt(Stmt::Edge(edge));
        }
        n
    }
}

impl ToGraphviz for Vec<AstNode> {
    fn to_graphviz(&self, graph: &mut Graph) -> Node {
        let mut n = node!(self.new_id());
        n.attributes = vec![attr!("label", "Expressions")];

        let mut edges = Vec::new();

        for child in self {
            let n1 = child.to_graphviz(graph);
            edges.push(edge!(n.id.clone() => n1.id.clone()));
        }

        graph.add_stmt(Stmt::Node(n.clone()));
        for edge in edges {
            graph.add_stmt(Stmt::Edge(edge));
        }
        n
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
                result.push('\r');
                i += 1;
            } else if s[i + 1] == 't' {
                result.push('\t');
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
