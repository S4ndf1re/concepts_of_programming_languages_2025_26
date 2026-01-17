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

    use crate::{Interpreter, Preprocessor, StaticSourceLoader};

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

        let preprocessor = Preprocessor::new(&loader);
        if let Err(expr) = preprocessor.preprocess(interpreter, &world) {
            expr.panic_error(&source);
        }
    }
}
