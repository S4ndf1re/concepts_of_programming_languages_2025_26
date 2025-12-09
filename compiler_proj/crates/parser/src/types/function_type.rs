use std::{cell::RefCell, fmt::Display, iter::zip, rc::Rc};

use derivative::Derivative;

use crate::{AstNode, Error, IsReturn, Scope, Symbol, TypeSymbol};

pub type BuildinCallback = fn(scope: Rc<RefCell<Scope>>) -> Result<IsReturn, Error>;

#[derive(Debug, Clone)]
pub enum FunctionExecutionStrategy {
    Buildin(BuildinCallback),
    Interpreted(Vec<Box<AstNode>>),
}

#[derive(Derivative)]
#[derivative(Debug, Clone, Hash, Eq)]
pub struct FunctionType {
    pub name: Symbol,
    pub params: Vec<(Symbol, TypeSymbol)>,
    pub return_type: Option<Box<TypeSymbol>>,
    #[derivative(Hash = "ignore")]
    pub execution_body: FunctionExecutionStrategy,
}

impl PartialEq for FunctionType {
    fn eq(&self, other: &Self) -> bool {
        let mut equals = true;

        for (p1, p2) in zip(&self.params, &other.params) {
            equals = equals && p1.1 == p2.1;
        }

        equals = equals && self.return_type == other.return_type;

        equals
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ret) = &self.return_type {
            write!(
                f,
                "fn {}({}): {}",
                self.name,
                self.params
                    .iter()
                    .map(|p| format!("{}: {}", p.0, p.1))
                    .collect::<Vec<String>>()
                    .join(", "),
                ret,
            )
        } else {
            write!(
                f,
                "fn {}({})",
                self.name,
                self.params
                    .iter()
                    .map(|p| format!("{}: {}", p.0, p.1))
                    .collect::<Vec<String>>()
                    .join(", "),
            )
        }
    }
}
