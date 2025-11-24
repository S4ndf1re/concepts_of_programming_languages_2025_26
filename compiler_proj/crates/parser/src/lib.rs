pub mod ast;
pub use ast::*;

pub mod lexer;
pub use lexer::*;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub ast_grammar);

#[cfg(test)]
mod tests {
    use crate::{AstNode, ast_grammar};

    #[test]
    fn import_test1() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"import name as abc;
                            import name2;"#,
            )
            .unwrap();

        assert!(expr.len() == 2);

        assert!(matches!(expr[0], AstNode::Import(_, _)));
        assert!(matches!(expr[1], AstNode::Import(_, _)));
    }

    #[test]
    fn import_test2() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"import native "module" "file";
                            import native "module" "file" as my_module;"#,
            )
            .unwrap();

        assert!(expr.len() == 2);

        assert!(matches!(expr[0], AstNode::ImportNative(_, _, _)));
        assert!(matches!(expr[1], AstNode::ImportNative(_, _, _)));
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

        assert!(matches!(expr[0], AstNode::Import(_, _)));
        assert!(matches!(expr[1], AstNode::ImportNative(_, _, _)));
        assert!(matches!(expr[2], AstNode::Import(_, _)));
        assert!(matches!(expr[3], AstNode::ImportNative(_, _, _)));
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
            expr[0],
            AstNode::TypeDef {
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
            expr[0],
            AstNode::Declaration {
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
                r#"if a == b {
            }"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0],
            AstNode::Branch {
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
                r#"c := if a == b {
            };"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0],
            AstNode::Declaration {
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
                r#"if a == b {
            } else {
                c := a;
            }"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0],
            AstNode::Branch {
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
                r#"if a != b {
            } else if a <= c {
            } else if c > d {
            } else {
                c := a;
            }"#,
            )
            .unwrap();

        assert!(expr.len() == 1);

        assert!(matches!(
            expr[0],
            AstNode::Branch {
                cond: _,
                body: _,
                else_if_branches: _,
                else_branch: Some(_),
            }
        ));
    }

    #[test]
    fn branch_test5() {
        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                if a {};
                "#,
            )
            .is_err();

        assert!(expr);

        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                if if a {} else {} {}
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
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                        for ;; {}

                        for i := 6; i < 8; i+=2 {}

                        for i = 3 ; i == 9;  {}

                        for a in list {}

                        while 1 == 2 {}
                        "#,
            )
            .unwrap();
    }

    #[test]
    fn structs1() {
        let _expr = ast_grammar::ProgrammParser::new()
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
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                    a:= a < 5 && (b || c);
                    a = (a <= b && b <= c) || a >= b && b>= c;
                            "#,
            )
            .unwrap();
    }

    #[test]
    fn function_call() {
        let _expr = ast_grammar::ProgrammParser::new()
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
                a := if b == c { d; } else { d.f.g(a,b,c).c.d().h; };
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
        let _expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                let a: weak [T] = 10;
                let a: [weak T] = 10;
                let a: weak [weak T] = 10;
                let b: weak {A -> B} = 10;
                let b: {weak A -> B} = 10;
                let c: {weak A -> weak B} = 10;
                let c: weak {weak A -> weak B} = 10;
                "#,
            )
            .unwrap();
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

        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                return if a == b {
                };
                    "#,
            )
            .is_err();
    }
}
