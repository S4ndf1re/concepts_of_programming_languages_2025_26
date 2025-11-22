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
                expression: _
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
                expression: _
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
    fn returnable_test() {
        let expr = ast_grammar::ProgrammParser::new()
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

        let expr = ast_grammar::ProgrammParser::new()
            .parse(
                r#"
                        a.c()
                        "#,
            );
        assert!(expr.is_err());
    }
}
