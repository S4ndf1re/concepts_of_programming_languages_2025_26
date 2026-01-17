#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use ecs::World;
    use parser_types::BeautifyError;

    use crate::{Interpreter, Preprocessor, StaticSourceLoader};

    #[test]
    fn test_basic_interpretation() {
        let source = String::from(
            r#"
           fn main() {
            a := 10;
            a += 20;
            assert(a == 30)
           }
           "#,
        );

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
        let source_loader = StaticSourceLoader::from(source.clone());

        let preprocessor = Preprocessor::new(&source_loader);
        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(&source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);
        if let Err(err) = interpreter.borrow_mut().run(&world) {
            err.panic_error(&source)
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
        let source_loader = StaticSourceLoader::from(source.clone());

        let preprocessor = Preprocessor::new(&source_loader);
        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(&source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);
        if let Err(err) = interpreter.borrow_mut().run(&world) {
            err.panic_error(&source)
        }
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
        let source_loader = StaticSourceLoader::from(source.clone());

        let preprocessor = Preprocessor::new(&source_loader);
        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(&source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);
        if let Err(err) = interpreter.borrow_mut().run(&world) {
            err.panic_error(&source)
        }
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
        let source_loader = StaticSourceLoader::from(source.clone());

        let preprocessor = Preprocessor::new(&source_loader);
        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(&source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);
        if let Err(err) = interpreter.borrow_mut().run(&world) {
            err.panic_error(&source)
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

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
        let source_loader = StaticSourceLoader::from(source.clone());

        let preprocessor = Preprocessor::new(&source_loader);
        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(&source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);
        if let Err(err) = interpreter.borrow_mut().run(&world) {
            err.panic_error(&source)
        }
    }

    #[test]
    fn ecs_integration1() {
        let source = r#"
            component Position1d {
                x: int,
            }

            system position_add_one(positions: P, world: W)
                querying P as List with {Position1d},
                         W as World
            {
                // println("Iterating over components");
                for (entt in positions) {
                    comp := entt[0];
                    comp.x += 1;
                    println(comp.x);

                    if (comp.x > 100) {
                        world.stop();
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

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
        let source_loader = StaticSourceLoader::from(source.clone());

        let preprocessor = Preprocessor::new(&source_loader);
        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(&source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);
        if let Err(err) = interpreter.borrow_mut().run(&world) {
            err.panic_error(&source)
        }

        world.run();
    }

    #[test]
    fn ecs_integration2() {
        let source = r#"
            component Position1d {
                x: int,
            }

            system position_add_one(positions: P, world: W)
                querying P as List with {Position1d},
                         W as World
            {
                // println("Iterating over components");
                for (entt in positions) {
                    entt[0].x += 1;
                    println(entt[0].x);

                    if (entt[0].x > 100) {
                        world.stop();
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

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
        let source_loader = StaticSourceLoader::from(source.clone());

        let preprocessor = Preprocessor::new(&source_loader);
        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(&source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);
        if let Err(err) = interpreter.borrow_mut().run(&world) {
            err.panic_error(&source)
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
                assert(list.pop() == 10);
                println("Final");
            }

           "#
        .to_owned();

        let world = World::default();
        let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
        let source_loader = StaticSourceLoader::from(source.clone());

        let preprocessor = Preprocessor::new(&source_loader);
        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(&source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);
        if let Err(err) = interpreter.borrow_mut().run(&world) {
            err.panic_error(&source)
        }

        world.run();
    }
}
