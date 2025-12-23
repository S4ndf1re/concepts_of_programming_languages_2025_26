use crate::{Error, ErrorWithRange, Stage, StageResult, ast_grammar};

#[derive(Default)]
pub struct Parser {
    main_content: &'static str,
}

impl Stage for Parser {
    fn init(&mut self, prev_stage_result: super::StageResult) -> Result<(), crate::ErrorWithRange> {
        match prev_stage_result {
            StageResult::PreParse(content) => self.main_content = Box::leak(Box::new(content)),
            _ => Err(Error::StageError(0, prev_stage_result.into()))
                .map_err(|err| ErrorWithRange { err, range: 0..1 })?,
        }
        Ok(())
    }

    fn run(self) -> Result<super::StageResult, crate::ErrorWithRange> {
        let ast = ast_grammar::ProgrammParser::new()
            .parse(&self.main_content)
            .map_err(|err| ErrorWithRange {
                err: Error::ParseError(err),
                range: 0..1,
            })?;
        Ok(StageResult::Parsing(ast))
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.main_content as *const str as *mut str);
        }
    }
}
