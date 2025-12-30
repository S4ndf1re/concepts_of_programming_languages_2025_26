
#[cfg(test)]
mod tests {
    use crate::{
        BeautifyError, Interpreter, Parser, Preprocessor, StageResult, Stages, ast_grammar,
        run_stages,
    };

    #[test]
    fn test_basic_interpretation() {
        let source = r#"
           fn main() {
            a := 10;
            a += 20;
           }
           "#;

        let ast = ast_grammar::ProgrammParser::new().parse(source).unwrap();

        let stages = vec![
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::Parsing(ast);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn test_basic_interpretation2() {
        let source = r#"
           fn main() {
            a = 10;
            a += 20;
            println(a);
           }
           "#;

        let ast = ast_grammar::ProgrammParser::new().parse(source).unwrap();

        let stages = vec![
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::Parsing(ast);

        let result = run_stages(stages, state);

        if let Err(err) = result {
            err.print_error(source);
        }
    }

    #[test]
    fn function_definition_and_returning() {
        let source = r#"

           fn test(a: int): int {
            return a + 10;
           }

           fn main() {
            a := 10;
            println(test(a));
           }
           "#
        .to_owned();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn loop1() {
        let source = r#"
           fn main() {
            a := 10;
            while (a > 0) {
                a -= 1;
            }
           }
           "#
        .to_owned();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn loop2() {
        let source = r#"
           fn main() {
                for (a := 10; a > 0; a -= 1) {
                }
           }
           "#
        .to_owned();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn loop3() {
        let source = r#"
           fn main() {
                res := 0;
                for (a in [10, 20, 30, 40]) {
                    res += a;
                }
                assert(res == true);
           }
           "#
        .to_owned();

        let source_safe = source.clone();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let result = run_stages(stages, state);
        if let Err(occured_error) = result {
            occured_error.print_error(&source_safe);
        }
    }

    #[test]
    fn member_call() {
        let source = r#"

            struct Abc {
                a: int,
                fn print(self) {
                    println(self.a);
                }
            }

           fn main() {
            abc := Abc {
                a: 10,
            };

            abc.print();
           }
           "#
        .to_owned();

        let source_safe = source.clone();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let result = run_stages(stages, state);
        if let Err(occured_error) = result {
            occured_error.panic_error(&source_safe);
        }
    }
}