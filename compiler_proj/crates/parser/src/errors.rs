use std::fmt::Display;

use annotate_snippets::{AnnotationKind, Level, Patch, Renderer, Snippet, renderer::DecorStyle};
use lalrpop_util::ParseError;
use thiserror::Error;

use crate::Symbol;

pub struct ErrorWithRange{
    pub(crate) err: Error,
    pub(crate) range: std::ops::Range<usize>
}


#[derive(Error, Debug)]
pub enum Error {
     #[error("variable {a} already declared")]
    OperationUnsupported{
        a: String,
        b:String,
        c:String
    },
    #[error("cannot deref a weak reference. convert to strong first")]
    CantDerefWeak,
    #[error("variable  already declared")]
    VariableAlreadyDeclared(Symbol),
    #[error("value and type  do not match")]
    ValueAndTypeDoNotMatch(String, String),
    #[error("type  is already registered")]
    TypeAlreadyExists(String),
    #[error("type  does not exist")]
    TypeDoesNotExist(String),
    #[error("stage error, expected stage , got stage ")]
    StageError(usize, usize),
    #[error("main function not found, can't start execution")]
    MainNotFound,
    #[error("expected  to be of type , but received ")]
    WrongType(Symbol, String, String),
    #[error("Type cannot be deducted, missing type")]
    TypeDeductionError,
    #[error("missing return in ")]
    MissingReturn(Symbol),
    #[error("expected value for parameter ")]
    ExpectedValue(Symbol),
    #[error("cannot downcast to weak as reference value is not strong")]
    CantDowncastToWeak,
    #[error("cannot upgrade value to strong reference counted value")]
    CantUpgradeToStrong,
    #[error("Interpreter value cannot be empty for assignment or decleration")]
    CantBeEmpty,
    #[error("symbol could not get resolved")]
    SymbolNotFound(Symbol),
    #[error("cant cast as type ")]
    CantCastAsType(Symbol),


    // #[error("variable {0} already declared")]
    // VariableAlreadyDeclared(Symbol),
    // #[error("value {0} and type {1} do not match")]
    // ValueAndTypeDoNotMatch(String, String),
    // #[error("type {0} is already registered")]
    // TypeAlreadyExists(String),
    // #[error("type {0} does not exist")]
    // TypeDoesNotExist(String),
    // #[error("stage error, expected stage {0}, got stage {1}")]
    // StageError(usize, usize),
    // #[error("main function not found, can't start execution")]
    // MainNotFound,
    // #[error("expected {0} to be of type {1}, but received {2}")]
    // WrongType(Symbol, String, String),
    // #[error("Type cannot be deducted, missing type")]
    // TypeDeductionError,
    // #[error("missing return in {0}")]
    // MissingReturn(Symbol),
    // #[error("expected value for parameter {0}")]
    // ExpectedValue(Symbol),
    // #[error("cannot downcast to weak as reference value is not strong")]
    // CantDowncastToWeak,
    // #[error("cannot upgrade value to strong reference counted value")]
    // CantUpgradeToStrong,
    // #[error("cannot deref a weak reference. convert to strong first")]
    // CantDerefWeak,
    // #[error("Interpreter value cannot be empty for assignment or decleration")]
    // CantBeEmpty,
    // #[error("Operation {0} not supported for applied types: {1} and {2}")]
    // OperationUnsupported(String, String, String),
    // #[error("symbol could not get resolved")]
    // SymbolNotFound(Symbol),
    // #[error("cant cast as type {0}")]
    // CantCastAsType(Symbol),
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

impl BeautifyError for Error{
    fn print_error(&self, source: &str) {
        match self{
            Error::OperationUnsupported{a,b,c}=> {
                                let report = &[
                    Level::ERROR
                        .primary_title(format!("Operation {a} not supported for applied types: {b} and {c}"))
                        .element(
                            Snippet::source(source).annotation(
                                AnnotationKind::Primary
                                    .span(1..4)
                                    .label(format!("expected, {}", b)),
                            ),
                        ),

                ];

                let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
                println!("{}", renderer.render(report));
            }
            _ => ()
        }
    }
}
