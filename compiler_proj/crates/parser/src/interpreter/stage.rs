use crate::{AstNode, Error, Scope};


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
