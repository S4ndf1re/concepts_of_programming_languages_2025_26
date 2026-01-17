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
    use parser_types::BeautifyError;

    use crate::{Interpreter, StaticSourceLoader, preprocess};

    #[test]
    fn test_preprocessing() {
        let source = String::from(
            r#"fn abc(a: B, c: int, d: float) {}
                                        struct B {
                                            a: float,
                                        }
                                "#,
        );

        let world = World::default();

        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let loader = StaticSourceLoader::from(source.clone());

        if let Err(expr) = preprocess(&loader, interpreter, &world) {
            expr.panic_error(&source);
        }
    }
}
