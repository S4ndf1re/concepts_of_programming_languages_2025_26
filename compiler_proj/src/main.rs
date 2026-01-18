use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use clap::Parser;
use ecs::World;
use parser::{
    BuiltinFunctionDescription, FileSourceLoader, Interpreter, Preprocessor, SourceLoader,
};
use parser_types::{BeautifyError, InterpreterValue};

pub mod raylib_builtin;
pub use raylib_builtin::*;

/// Run a simple file as an interpreted script
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct FileArgs {
    /// Filename of the file to run
    #[arg(index = 1)]
    mainfile: String,
}

fn main() {
    let args = FileArgs::parse();

    let path = Path::new(&args.mainfile);
    let filename = PathBuf::from(path.file_name().unwrap());
    let file_prefix = PathBuf::from(path.parent().unwrap());
    dbg!(&filename);
    dbg!(&file_prefix);

    let world = World::default();
    let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
    let source_loader = FileSourceLoader::new(file_prefix, filename);
    // let source_loader = StaticSourceLoader::from("".to_string());
    let main_file_source = source_loader.load_main_file().unwrap();
    dbg!(main_file_source);

    {
        let preprocessor = Preprocessor::new(&source_loader);

        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(main_file_source);
        };

        preprocessor
            .register_builtin_function(BuiltinFunctionDescription {
                name: "raylib_init".to_owned(),
                callback: raylib_init,
                params: vec![],
                return_type: None,
            })
            .unwrap();

        preprocessor
            .register_builtin_component(Position2d {
                scope: Rc::clone(&global_scope),
                x: InterpreterValue::Int(0),
                y: InterpreterValue::Int(0),
            })
            .unwrap();

        preprocessor
            .register_builtin_component(Colorable {
                scope: Rc::clone(&global_scope),
                r: InterpreterValue::Int(0),
                g: InterpreterValue::Int(0),
                b: InterpreterValue::Int(0),
            })
            .unwrap();

        preprocessor
            .register_builtin_component(RectangleShape {
                scope: Rc::clone(&global_scope),
                w: InterpreterValue::Int(0),
                h: InterpreterValue::Int(0),
            })
            .unwrap();

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);

        if let Err(err) = interpreter.borrow_mut().run(&world, main_file_source) {
            err.panic_error(main_file_source)
        }

        world.add_system(raylib_system);
        world.run();
    }
}
