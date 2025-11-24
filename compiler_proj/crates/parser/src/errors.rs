use std::fmt::Display;

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet, renderer::DecorStyle};
use lalrpop_util::ParseError;

pub fn print_error<T: Display, E: Display>(source: &str, err: &ParseError<usize, T, E>) {
    match err {
        ParseError::UnrecognizedEof { location, expected } => {
            let report = &[Level::ERROR.primary_title("unexpected eof").element(
                Snippet::source(source).annotation(
                    AnnotationKind::Primary
                        .span(*location..*location + 1)
                        .label(format!("expected, {}", expected.join(", "))),
                ),
            )];

            let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
            println!("{}", renderer.render(report));
        }
        ParseError::UnrecognizedToken { token, expected } => {
            let report = &[Level::ERROR
                .primary_title(format!("unexpected token {}", token.1))
                .element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(token.0..token.2)
                            .label(format!("expected, {}", expected.join(", "))),
                    ),
                )];

            let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
            println!("{}", renderer.render(report));
        }
        ParseError::InvalidToken { location } => {
            let report = &[Level::ERROR.primary_title("invalid token").element(
                Snippet::source(source).annotation(
                    AnnotationKind::Primary
                        .span(*location..*location + 1)
                        .label("token does not exist"),
                ),
            )];

            let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
            println!("{}", renderer.render(report));
        }
        ParseError::ExtraToken { token } => {
            let report = &[Level::ERROR
                .primary_title("unexpected extra token")
                .element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(token.0..token.2)
                            .label(format!("token {} is unexpected", token.1)),
                    ),
                )];

            let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
            println!("{}", renderer.render(report));
        }
        ParseError::User { error } => {
            println!("User error: {}", error);
        }
    }
}
