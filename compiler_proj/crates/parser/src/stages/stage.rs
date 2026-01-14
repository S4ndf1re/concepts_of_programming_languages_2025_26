use std::{cell::RefCell, rc::Rc};

use ecs::World;
use parser_types::{AstNode, ErrorWithRange, Scope};

use crate::{Interpreter, Parser, Preprocessor};

#[allow(clippy::large_enum_variant)]
pub enum Stages<'w> {
    Parser(Parser<'w>),
    Preprocessor(Preprocessor<'w>),
    Interpreter(Rc<RefCell<Interpreter>>),
}

pub enum StageResult<'w> {
    PreParse(&'w World, String, Rc<RefCell<Interpreter>>),
    Parsing(&'w World, Vec<AstNode>, Rc<RefCell<Interpreter>>),
    Preprocessor(Scope, Vec<AstNode>),
    Interpretation,
}

impl<'w> From<StageResult<'w>> for usize {
    fn from(value: StageResult) -> Self {
        match value {
            StageResult::PreParse(_, _, _) => 0,
            StageResult::Parsing(_, _, _) => 1,
            StageResult::Preprocessor(_, _) => 2,
            StageResult::Interpretation => 3,
        }
    }
}

pub trait Stage<'w> {
    fn init(&mut self, prev_stage_result: StageResult<'w>) -> Result<(), ErrorWithRange>;
    fn run(self, world: &'w World, source: String) -> Result<StageResult<'w>, ErrorWithRange>;
}

pub fn run_stages<'w>(
    stages: Vec<Stages<'w>>,
    mut state: StageResult<'w>,
    world: &'w World,
    source: String,
) -> Result<StageResult<'w>, ErrorWithRange> {
    for stage in stages {
        match stage {
            Stages::Parser(mut p) => {
                p.init(state)?;
                state = p.run(world, source.clone())?;
            }
            Stages::Preprocessor(mut p) => {
                p.init(state)?;
                state = p.run(world, source.clone())?;
            }
            Stages::Interpreter(i) => {
                let mut interpreter = Rc::clone(&i);
                interpreter.init(state)?;
                state = interpreter.run(world, source.clone())?;
            }
        }
    }

    Ok(state)
}
