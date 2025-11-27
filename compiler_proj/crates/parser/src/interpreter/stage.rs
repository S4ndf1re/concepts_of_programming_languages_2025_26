use crate::{AstNode, Error, Interpreter, Preprocessor, Scope};

pub enum Stages {
    Preprocessor(Preprocessor),
    Interpreter(Interpreter),
}

pub enum StageResult {
    Stage0(Vec<AstNode>),
    Stage1(Scope, Vec<AstNode>),
    Stage2,
}

impl From<StageResult> for usize {
    fn from(value: StageResult) -> Self {
        match value {
            StageResult::Stage0(_) => 0,
            StageResult::Stage1(_, _) => 1,
            StageResult::Stage2 => 2,
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
