use std::{cell::RefCell, fmt::{Display, Error}, iter::zip, rc::Rc, thread::Scope};

use derivative::Derivative;

use crate::{AstNode, Query, Symbol};


pub type BuildinSystemCallback = fn(scope: Rc<RefCell<Scope>>) -> Result<(), Error>;

#[derive(Debug, Clone)]
pub enum SystemExecutionStrategy {
    Buildin(BuildinSystemCallback),
    Interpreted(Vec<Box<AstNode>>),
}

#[derive(Derivative)]
#[derivative(Debug, Clone, Hash, Eq)]
pub struct SystemType {
    pub name: Symbol,
    pub params: Vec<(Symbol, Symbol)>,
    pub queries: Option<Vec<Query>>,
    #[derivative(Hash = "ignore")]
    pub execution_body: SystemExecutionStrategy,
}

impl PartialEq for SystemType {
    fn eq(&self, other: &Self) -> bool {
        let mut equals = true;

        for (p1, p2) in zip(&self.params, &other.params) {
            equals = equals && p1.1 == p2.1;
        }

        equals
    }
}

impl Display for SystemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "system {}({})",
            self.name,
            self.params
                .iter()
                .map(|p| format!("{}: {}", p.0, p.1))
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}
