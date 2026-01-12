pub mod stage;
pub use stage::*;

pub mod parser;
pub use parser::*;

pub mod preprocessor;
pub use preprocessor::*;

pub mod interpreter;
pub use interpreter::*;

pub mod buildin;
pub use buildin::*;

mod interpreter_test;

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use ecs::World;

    use crate::{BeautifyError, Interpreter, Preprocessor, Stage, StageResult, ast_grammar};

    #[test]
    fn test_preprocessing() {
        let source = String::from(
            r#"fn abc(a: B, c: int, d: float) {}
                                        struct B {
                                            a: float,
                                        }
                                "#,
        );

        let expr = ast_grammar::ProgrammParser::new().parse(&source);

        if let Err(expr) = expr {
            expr.panic_error(&source);
        } else {
            let expr = expr.unwrap();

            let world = World::default();
            let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
            let s0 = StageResult::Parsing(&world, expr, interpreter);

            let mut processor = Preprocessor::new().unwrap();
            processor.init(s0).unwrap();
            processor.run(&world, source).unwrap();
        }
    }
}
