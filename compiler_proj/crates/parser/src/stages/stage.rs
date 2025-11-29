use crate::{AstNode, Error, Interpreter, Parser, Preprocessor, Scope};

pub enum Stages {
    Parser(Parser),
    Preprocessor(Preprocessor),
    Interpreter(Interpreter),
}

pub enum StageResult {
    PreParse(String),
    Parsing(Vec<AstNode>),
    Stage0(Scope, Vec<AstNode>),
    Stage1,
}

impl From<StageResult> for usize {
    fn from(value: StageResult) -> Self {
        match value {
            StageResult::PreParse(_) => 0,
            StageResult::Parsing(_) => 1,
            StageResult::Stage0(_, _) => 2,
            StageResult::Stage1 => 3,
        }
    }
}

pub trait Stage {
    fn init(&mut self, prev_stage_result: StageResult) -> Result<(), Error>;
    fn run(self) -> Result<StageResult, Error>;
}

pub fn run_stages(stages: Vec<Stages>, mut state: StageResult) -> Result<StageResult, Error> {
    for stage in stages {
        match stage {
            Stages::Parser(mut p) =>{
               p.init(state)?;
               state = p.run()?;
            }
            Stages::Preprocessor(mut p) => {
                p.init(state)?;
                state = p.run()?;
            }
            Stages::Interpreter(mut i) => {
                i.init(state)?;
                state = i.run()?;
            }
        }
    }

    Ok(state)
}
