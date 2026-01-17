use parser_types::{AstNode, BeautifyError, ast_grammar};

pub fn parse_content(source: &str) -> Vec<AstNode> {
    let ast = ast_grammar::ProgrammParser::new().parse(source);

    if let Err(err) = ast {
        err.panic_error(source);
    } else {
        ast.unwrap()
    }
}
