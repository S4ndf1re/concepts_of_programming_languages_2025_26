pub mod ast;
pub use ast::*;

pub mod errors;
pub use errors::*;

pub mod stages;
pub use stages::*;

pub mod types;
pub use types::*;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub ast_grammar);

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use graphviz_rust::{
        cmd::Format,
        dot_generator::{graph, id},
        dot_structures::{Graph, Id},
        exec_dot,
        printer::{DotPrinter, PrinterContext},
    };

    use crate::{AstNodeType, BeautifyError, ToGraphviz, ast_grammar};

    #[test]
    fn import_test1() {
        let source = r#"import name as abc;
                            import name2;"#;
        let expr = ast_grammar::ProgrammParser::new().parse(source);

        if let Err(expr) = expr {
            expr.panic_error(source);
        } else if let Ok(expr) = expr {
            assert!(expr.len() == 2);

            assert!(matches!(expr[0].type_of, AstNodeType::Import(_, _)));
            assert!(matches!(expr[1].type_of, AstNodeType::Import(_, _)));
        }
    }

    #[test]
    fn import_test2() {
        let source = r#"import native "module" "file";
                            import native "module" "file" as my_module;"#;
        let expr = ast_grammar::ProgrammParser::new().parse(source);

        if let Err(expr) = expr {
            expr.print_error(source);
            panic!("{}", expr);
        } else if let Ok(expr) = expr {
            assert!(expr.len() == 2);

            assert!(matches!(
                expr[0].type_of,
                AstNodeType::ImportNative(_, _, _)
            ));
            assert!(matches!(
                expr[1].type_of,
                AstNodeType::ImportNative(_, _, _)
            ));
        }
    }

    #[test]
    fn import_test3() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"import name as abc;
                            import native "module" "file";
                            import name2 as abc;
                            import native "module" "file" as my_module;"#,
            )
            .unwrap();

        assert!(expr.len() == 4);

        assert!(matches!(expr[0].type_of, AstNodeType::Import(_, _)));
        assert!(matches!(
            expr[1].type_of,
            AstNodeType::ImportNative(_, _, _)
        ));
        assert!(matches!(expr[2].type_of, AstNodeType::Import(_, _)));
        assert!(matches!(
            expr[3].type_of,
            AstNodeType::ImportNative(_, _, _)
        ));
    }

    #[test]
    fn function_definition_test1() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"fn my_function_name(a: int, b: string, c: MyStruct): float {
            }"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0].type_of,
            AstNodeType::TypeDef {
                typename: _,
                typedef: _,
                execution_body: _,
            }
        ));
    }

    #[test]
    fn declaration_test1() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(r#"a := b;"#)
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0].type_of,
            AstNodeType::Declaration {
                new_symbol: _,
                expression: _,
                assumed_type: _,
            }
        ));
    }

    #[test]
    fn branch_test1() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"if (a == b) {
            }"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0].type_of,
            AstNodeType::Branch {
                cond: _,
                body: _,
                else_if_branches: _,
                else_branch: _
            }
        ));
    }

    #[test]
    fn branch_test2() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"c := if (a == b) {
            };"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0].type_of,
            AstNodeType::Declaration {
                new_symbol: _,
                expression: _,
                assumed_type: _,
            }
        ));
    }

    #[test]
    fn branch_test3() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"if (a == b) {
            } else {
                c := a;
            }"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0].type_of,
            AstNodeType::Branch {
                cond: _,
                body: _,
                else_if_branches: _,
                else_branch: Some(_),
            }
        ));
    }

    #[test]
    fn branch_test4() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"if (1 != 2) {
            } else if (1 <= 2 && 1 == 2) {
            } else if (1 > 2) {
            } else {
                c := 1;
            }"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0].type_of,
            AstNodeType::Branch {
                cond: _,
                body: _,
                else_if_branches: _,
                else_branch: Some(_),
            }
        ));

        let format = Format::Png;

        let mut g = graph!(strict di id!());
        expr.to_graphviz(&mut g);

        let dot = g.print(&mut PrinterContext::default());
        let graph_svg = exec_dot(dot.clone(), vec![format.into()]).unwrap();

        let mut file = File::create("parsed_trees/branch.png").unwrap();
        file.write_all(&graph_svg).unwrap();
    }

    #[test]
    fn branch_test5() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                if (a) {};
                "#,
            )
            .is_err();

        assert!(expr);

        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                if if (a) {} else {} {}
                "#,
            )
            .is_err();

        assert!(expr);
    }

    #[test]
    fn returnable_test() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                        a := (a == b) != c != d;
                        a = a >= b;
                        c := a;
                        c := a.c();
                        a.c();
                        "#,
            )
            .unwrap();

        let expr = ast_grammar::ProgrammParser::new().parse(
            r#"
                        a.c()
                        "#,
        );
        assert!(expr.is_err());
    }

    #[test]
    fn loops1() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                        for (;;) {}

                        for (i := 6; i < 8; i+=2) {}

                        for (i = 3 ; i == 9;)  {}

                        for (a in list) {}

                        while (1 == 2) {}
                        "#,
            )
            .unwrap();

        let format = Format::Png;

        let mut g = graph!(strict di id!());
        expr.to_graphviz(&mut g);

        let dot = g.print(&mut PrinterContext::default());
        let graph_svg = exec_dot(dot.clone(), vec![format.into()]).unwrap();

        let mut file = File::create("parsed_trees/loops.png").unwrap();
        file.write_all(&graph_svg).unwrap();
    }

    #[test]
    fn structs1() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                        struct A {
                            a: float,
                            fn my_function_name(a: int, b: string, c: MyStruct): float {
                            }
                            c: String,
                        }
                            "#,
            )
            .unwrap();

        let format = Format::Png;

        let mut g = graph!(strict di id!());
        expr.to_graphviz(&mut g);

        let dot = g.print(&mut PrinterContext::default());
        let graph_svg = exec_dot(dot.clone(), vec![format.into()]).unwrap();

        let mut file = File::create("parsed_trees/struct.png").unwrap();
        file.write_all(&graph_svg).unwrap();
    }

    #[test]
    fn math1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                    a:= a+b;
                    a = a-b;
                    a = a*b;
                    a = a/b;
                    a = a/b+c;
                    a = a*b*c;
                    a = a-b*c;
                    a = a%b;
                            "#,
            )
            .unwrap();
    }

    #[test]
    fn logic1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                    a:= a && b;
                    a = a || b;
                    a = a || a + b;
                            "#,
            )
            .unwrap();
    }

    #[test]
    fn logic2() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                    a:= a < 5 && (b || c);
                    a = (a <= b && b <= c) || a >= b && b>= c;
                            "#,
            )
            .unwrap();

        let format = Format::Png;

        let mut g = graph!(strict di id!());
        expr.to_graphviz(&mut g);

        let dot = g.print(&mut PrinterContext::default());
        let graph_svg = exec_dot(dot.clone(), vec![format.into()]).unwrap();

        let mut file = File::create("parsed_trees/condition.png").unwrap();
        file.write_all(&graph_svg).unwrap();
    }

    #[test]
    fn function_call() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                        a();
                        a.c();
                        a(a.c(d.e.f.g().c(a,e())));
                        a.c();
                        a.c.e();
                        f := a();
                        f();
                    "#,
            )
            .unwrap();

        let format = Format::Png;

        let mut g = graph!(strict di id!());
        expr.to_graphviz(&mut g);

        let dot = g.print(&mut PrinterContext::default());
        let graph_svg = exec_dot(dot.clone(), vec![format.into()]).unwrap();

        let mut file = File::create("parsed_trees/member_access.png").unwrap();
        file.write_all(&graph_svg).unwrap();

        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                        a()();
                    "#,
            )
            .is_err();

        assert!(expr);
    }

    #[test]
    fn unary1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                -a - -a;
                !a;
                a && !b;
                -10;
                -10.0;
                --10.0;
                !(a || b);
                    "#,
            )
            .unwrap();
        // a := !a;
        // b := !b && -a;
    }

    #[test]
    fn special_assigns1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                a := if (b == c) { d; } else { d.f.g(a,b,c).c.d().h; };
                    "#,
            )
            .unwrap();
    }

    #[test]
    fn weak_test1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                let a: weak T = 10;

                fn f(a: weak T, b: T): weak R {
                }

                struct MyStruct {
                    a: weak float,

                    fn f(a: weak T, b: T): weak R {
                    }

                    c: float,
                }
                    "#,
            )
            .unwrap();
    }

    #[test]
    fn weak_test2() {
        let source = r#"
            let a: weak [T] = 10;
            let a: [weak T] = 10;
            let a: weak [weak T] = weak a;
            let b: weak {A -> B} = weak b;
            let b: {weak A -> B} = 10;
            let c: {weak A -> weak B} = 10;
            let c: weak {weak A -> weak B} = 10;
            "#;
        let expr = ast_grammar::ProgrammParser::new().parse(source);

        if let Err(expr) = expr {
            expr.panic_error(source);
        }
    }

    #[test]
    fn list_test1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                let a: weak T = 10;

                fn f(a: weak T, b: T): weak R {
                }

                struct MyStruct {
                    a: weak float,

                    fn f(a: weak T, b: T): weak R {
                    }

                    c: float,
                }
                    "#,
            )
            .unwrap();
    }

    #[test]
    fn return_test1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                fn f(a: int, b: String): int {
                    return a;
                }
                return a < b && b + 7;
                    "#,
            )
            .unwrap();

        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                return if (a == b) {
                };
                    "#,
            )
            .unwrap();
    }

    #[test]
    fn comment_test1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                fn f(a: int, b: String): int { //this is my commenttest
                    return a;
                }
                //here, i return something
                return a < b && b + 7;
                    "#,
            )
            .unwrap();
    }

    #[test]
    fn option_test1() {
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                let a: B? = some(10);
                let a: B? = none;
                a := some(10);
                b := none;
                    "#,
            )
            .unwrap();
    }

    #[test]
    fn result_test1() {
        let source = r#"
                let a: B!C = ok(10);
                let a: B!D = err(10);
                a := ok(10);
                b := err(10);
                    "#;
        let expr = ast_grammar::ProgrammParser::new().parse(source);

        if let Err(err) = expr {
            err.print_error(source);
            panic!("{}", err)
        }
    }

    #[test]
    fn struct_test1() {
        let source = r#"
                    a := B {
                        v: 10,
                        g: 10.0,
                        h: C {},
                    };
                    "#;
        let expr = ast_grammar::ProgrammParser::new().parse(source);

        if let Err(err) = expr {
            err.print_error(source);
            panic!("{}", err)
        }
    }
}
