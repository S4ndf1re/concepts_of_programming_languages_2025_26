#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use ecs::World;

    use crate::{
        BeautifyError, Interpreter, Parser, Preprocessor, StageResult, Stages, ast_grammar,
        run_stages,
    };

    #[test]
    fn test_basic_interpretation() {
        let source = String::from(
            r#"
           fn main() {
            a := 10;
            a += 20;
           }
           "#,
        );

        let ast = ast_grammar::ProgrammParser::new().parse(&source).unwrap();

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let stages = vec![
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::Parsing(&world, ast, Rc::clone(&interpreter));

        let _ = run_stages(stages, state, &world, source).unwrap();
    }

    #[test]
    fn test_basic_interpretation2() {
        let source = String::from(r#"
           fn main() {
            a = 10;
            a += 20;
            println(a);
           }
           "#);

        let ast = ast_grammar::ProgrammParser::new().parse(&source).unwrap();
        let world = World::default();

        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
        let stages = vec![
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::Parsing(&world, ast, Rc::clone(&interpreter));

        let result = run_stages(stages, state, &world, source.clone());

        if let Err(err) = result {
            err.print_error(&source);
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

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::PreParse(&world, source.clone(), Rc::clone(&interpreter));

        let _ = run_stages(stages, state, &world, source).unwrap();
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

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::PreParse(&world, source.clone(), Rc::clone(&interpreter));

        let _ = run_stages(stages, state, &world, source).unwrap();
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

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::PreParse(&world, source.clone(), Rc::clone(&interpreter));

        let _ = run_stages(stages, state, &world, source).unwrap();
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

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::PreParse(&world, source.clone(), Rc::clone(&interpreter));

        let result = run_stages(stages, state, &world, source);
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

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::PreParse(&world, source.clone(), Rc::clone(&interpreter));

        let result = run_stages(stages, state, &world, source);
        if let Err(occured_error) = result {
            occured_error.panic_error(&source_safe);
        }
    }

    #[test]
    fn ecs_integration1() {
        let source = r#"
            component Position1d {
                x: int,
            }

            system position_add_one(positions: P)
            querying P as List with {Position1d} {
                // println("Iterating over components");
                for (entt in positions) {
                    comp := entt[0];
                    comp.x += 1;
                    println(comp.x);

                    if (comp.x > 100) {
                        stop();
                    }
                }
            }


            register position_add_one;

            fn main() {
               create entity e1;

              e1 += Position1d {
                x: 0,
              };
            }

           "#
        .to_owned();

        let source_safe = source.clone();

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::PreParse(&world, source.clone(), Rc::clone(&interpreter));

        let result = run_stages(stages, state, &world, source);
        if let Err(occured_error) = result {
            occured_error.panic_error(&source_safe);
        }

        world.run();
    }

    #[test]
    fn ecs_integration2() {
        let source = r#"
            component Position1d {
                x: int,
            }

            system position_add_one(positions: P)
            querying P as List with {Position1d} {
                // println("Iterating over components");
                for (entt in positions) {
                    entt[0].x += 1;
                    println(entt[0].x);

                    if (entt[0].x > 100) {
                        stop();
                    }
                }
            }


            register position_add_one;

            fn main() {
               create entity e1;

              e1 += Position1d {
                x: 0,
              };
            }

           "#
        .to_owned();

        let source_safe = source.clone();

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::PreParse(&world, source.clone(), Rc::clone(&interpreter));

        let result = run_stages(stages, state, &world, source);
        if let Err(occured_error) = result {
            occured_error.panic_error(&source_safe);
        }

        world.run();
    }

    #[test]
    fn builtin_struct() {
        let source = r#"
            fn main() {
                list := BuiltinList{};
                println("Pre push");
                list.push(10);
                println("Post push");
                println(list.pop());
                println("Final");
            }

           "#
        .to_owned();

        let source_safe = source.clone();

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Rc::clone(&interpreter)),
        ];

        let state = StageResult::PreParse(&world, source.clone(), Rc::clone(&interpreter));

        let result = run_stages(stages, state, &world, source);
        if let Err(occured_error) = result {
            occured_error.panic_error(&source_safe);
        }

        world.run();
    }
}
