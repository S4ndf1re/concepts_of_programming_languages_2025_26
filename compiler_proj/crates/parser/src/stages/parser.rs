use crate::{Error, Stage, StageResult, ast_grammar};




#[derive(Default)]
pub struct Parser {
    main_content: String
}


impl Stage for Parser {
    fn init(&mut self, prev_stage_result: super::StageResult) -> Result<(), crate::Error> {
        match prev_stage_result {
            StageResult::PreParse(content) => self.main_content = content,
            _ => Err(Error::StageError(0, prev_stage_result.into()))?
        }
        Ok(())
    }

    fn run(self) -> Result<super::StageResult, crate::Error> {
        let ast = ast_grammar::ProgrammParser::new().parse(&self.main_content).unwrap();
        Ok(StageResult::Parsing(ast))
    }
}
