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

#[cfg(test)]
mod tests {
    use crate::{BeautifyError, Preprocessor, Stage, StageResult, ast_grammar};

    #[test]
    fn test_preprocessing() {
        let source = r#"fn abc(a: B, c: int, d: float) {}
                                struct B {
                                    a: float,
                                }
                        "#;
        let expr = ast_grammar::ProgrammParser::new().parse(source);

        if let Err(expr) = expr {
            expr.panic_error(source);
        } else {
            let expr = expr.unwrap();

            let s0 = StageResult::Parsing(expr);

            let mut processor = Preprocessor::new().unwrap();
            processor.init(s0).unwrap();
            processor.run().unwrap();
        }
    }
}
