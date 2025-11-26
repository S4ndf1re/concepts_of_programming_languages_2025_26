use std::{fmt::Display, iter::zip};

use derivative::Derivative;

use crate::{AstNode, Symbol, TypeSymbol};


#[derive(Derivative)]
#[derivative(Debug, Clone, Hash, Eq)]
pub struct FunctionType {
    pub name: Symbol,
    pub params: Vec<(Symbol, TypeSymbol)>,
    pub return_type: Option<Box<TypeSymbol>>,
    #[derivative(Hash = "ignore")]
    pub execution_body: Vec<Box<AstNode>>,
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
