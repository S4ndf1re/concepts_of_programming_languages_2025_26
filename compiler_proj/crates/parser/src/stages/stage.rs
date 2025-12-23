use crate::{AstNode, ErrorWithRange, Interpreter, Parser, Preprocessor, Scope};

pub enum Stages {
    Parser(Parser),
    Preprocessor(Preprocessor),
    Interpreter(Interpreter),
}

pub enum StageResult {
    PreParse(String),
    Parsing(Vec<AstNode>),
    Preprocessor(Scope, Vec<AstNode>),
    Interpretation,
}

impl From<StageResult> for usize {
    fn from(value: StageResult) -> Self {
        match value {
            StageResult::PreParse(_) => 0,
            StageResult::Parsing(_) => 1,
            StageResult::Preprocessor(_, _) => 2,
            StageResult::Interpretation => 3,
        }
    }
}

pub trait Stage {
    fn init(&mut self, prev_stage_result: StageResult) -> Result<(), ErrorWithRange>;
    fn run(self) -> Result<StageResult, ErrorWithRange>;
}

pub fn run_stages(
    stages: Vec<Stages>,
    mut state: StageResult,
) -> Result<StageResult, ErrorWithRange> {
    for stage in stages {
        match stage {
            Stages::Parser(mut p) => {
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
