use std::{cell::RefCell, rc::Rc};

use ecs::World;

use crate::{Error, ErrorWithRange, Interpreter, Stage, StageResult, ast_grammar};

#[derive(Default)]
pub struct Parser<'w> {
    main_content: &'static str,
    world: Option<&'w World>,
    interpreter: Option<Rc<RefCell<Interpreter>>>,
}

impl<'w> Stage<'w> for Parser<'w> {
    fn init(
        &mut self,
        prev_stage_result: super::StageResult<'w>,
    ) -> Result<(), crate::ErrorWithRange> {
        match prev_stage_result {
            StageResult::PreParse(world, content, interpreter) => {
                self.main_content = Box::leak(Box::new(content));
                self.world = Some(world);
                self.interpreter = Some(interpreter);
            }
            _ => Err(Error::StageError(0, prev_stage_result.into()))
                .map_err(|err| ErrorWithRange { err, range: 0..1 })?,
        }
        Ok(())
    }

    fn run(self, _world: &'w World, _source: String) -> Result<super::StageResult<'w>, crate::ErrorWithRange> {
        let ast = ast_grammar::ProgrammParser::new()
            .parse(self.main_content)
            .map_err(|err| ErrorWithRange {
                err: Error::ParseError(err),
                range: 0..1,
            })?;
        Ok(StageResult::Parsing(
            self.world.expect("must be checked in init method"),
            ast,
            Rc::clone(self.interpreter.as_ref().expect("must be present as checked in init")),
        ))
    }
}

impl<'w> Drop for Parser<'w> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.main_content as *const str as *mut str));
        }
    }
}
