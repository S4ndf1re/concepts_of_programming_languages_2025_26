use std::fmt::Display;

use annotate_snippets::{AnnotationKind, Level, Patch, Renderer, Snippet, renderer::DecorStyle};
use lalrpop_util::ParseError;
use thiserror::Error;

use crate::{Symbol, ast_grammar};

#[derive(Clone, Debug, Error)]
pub struct ErrorWithRange {
    pub err: Error,
    pub range: std::ops::Range<usize>,
}

impl std::fmt::Display for ErrorWithRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.err, self.range.start, self.range.end)
    }
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("operation {operation} unsupported: {type_of}")]
    OperationUnsupported { operation: String, type_of: String },
    #[error("cannot deref a weak reference. convert to strong first")]
    CantDerefWeak,
    #[error("variable {0} already declared")]
    VariableAlreadyDeclared(Symbol),
    #[error("value {0} and type {1} do not match")]
    ValueAndTypeDoNotMatch(String, String),
    #[error("type {0} is already registered")]
    TypeAlreadyExists(String),
    #[error("type {0} does not exist")]
    TypeDoesNotExist(String),
    #[error("stage error, expected stage {0}, got stage {1}")]
    StageError(usize, usize),
    #[error("main function not found, can't start execution")]
    MainNotFound,
    #[error("expected {0} to be of type {1}, but received {2}")]
    WrongType(Symbol, String, String),
    #[error("Type cannot be deducted, missing type")]
    TypeDeductionError,
    #[error("missing return in {0}")]
    MissingReturn(Symbol),
    #[error("expected value for parameter {0}")]
    ExpectedValue(Symbol),
    #[error("cannot downcast to weak as reference value is not strong")]
    CantDowncastToWeak,
    #[error("cannot upgrade value to strong reference counted value")]
    CantUpgradeToStrong,
    #[error("Interpreter value cannot be empty for assignment or decleration")]
    CantBeEmpty,
    #[error("symbol {0} could not get resolved")]
    SymbolNotFound(Symbol),
    #[error("cant cast as type {0}")]
    CantCastAsType(Symbol),
    #[error("parser error")]
    ParseError(ParseError<usize, ast_grammar::Token<'static>, &'static str>),
}

pub trait BeautifyError: Display {
    fn print_error(&self, source: &str);
    fn panic_error(&self, source: &str) {
        self.print_error(source);
        panic!("{}", self)
    }
}

impl<T: Display, E: Display> BeautifyError for ParseError<usize, T, E> {
    fn print_error(&self, source: &str) {
        match self {
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
                let report = &[
                    Level::ERROR
                        .primary_title(format!("unexpected token {}", token.1))
                        .element(
                            Snippet::source(source).annotation(
                                AnnotationKind::Primary
                                    .span(token.0..token.2)
                                    .label(format!("expected, {}", expected.join(", "))),
                            ),
                        ),
                    Level::HELP.secondary_title("Possible fix").element(
                        Snippet::source(source).patch(Patch::new(
                            token.0..token.0 + 1,
                            format!("try inserting one of [{}] here", expected.join(", ")),
                        )),
                    ),
                ];

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
}

impl BeautifyError for ErrorWithRange {
    fn print_error(&self, source: &str) {
        match &self.err {
            Error::OperationUnsupported {
                operation: _,
                type_of,
            } => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(type_of),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::SymbolNotFound(_symbol) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label("unknown"),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::CantBeEmpty => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label("must not be empty"),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::CantCastAsType(type_of) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("can't cast to {}", type_of)),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::CantDerefWeak => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label("is weak, and cannot be dereferenced"),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::CantDowncastToWeak => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label("can't be downcast to weak"),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::CantUpgradeToStrong => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label("can't upgrade to strong"),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::ExpectedValue(value) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("expected {value}")),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::MainNotFound => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("no main function")),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::MissingReturn(func) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("missing return")),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::ParseError(err) => {
                err.print_error(source);
            }
            Error::StageError(should, is) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("expected stage {should}, got stage {is}")),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::TypeAlreadyExists(type_of) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("{type_of} already exists")),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::TypeDeductionError => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label("wrong type deducted"),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::TypeDoesNotExist(type_of) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label("does not exist"),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::ValueAndTypeDoNotMatch(type_of, value_of) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("{type_of} != value_of")),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::VariableAlreadyDeclared(var) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("already declared")),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            Error::WrongType(_, expected, _) => {
                let report = &[Level::ERROR
                    .primary_title(format!("{}", &self.err))
                    .element(
                        Snippet::source(source).annotation(
                            AnnotationKind::Primary
                                .span(self.range.clone())
                                .label(format!("should be of type {expected}")),
                        ),
                    )];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
        }
    }

    fn panic_error(&self, source: &str) {
        self.print_error(source);
        panic!("{}", self.err)
    }
}
