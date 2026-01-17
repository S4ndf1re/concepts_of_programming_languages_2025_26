use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use clap::Parser;
use ecs::World;
use parser::{FileSourceLoader, Interpreter, Preprocessor, SourceLoader, StaticSourceLoader};
use parser_types::BeautifyError;

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

    let world = World::default();
    let interpreter = Rc::new(RefCell::new(Interpreter::new("main".to_owned())));
    let source_loader = FileSourceLoader::new(file_prefix, filename);
    // let source_loader = StaticSourceLoader::from("".to_string());
    let main_file_source = source_loader.load_main_file().unwrap();

    {
        let preprocessor = Preprocessor::new(&source_loader);

        let preprocessed = preprocessor.preprocess(Rc::clone(&interpreter), &world);
        let Ok((ast, global_scope)) = preprocessed else {
            preprocessed.unwrap_err().panic_error(main_file_source);
        };

        interpreter
            .borrow_mut()
            .initialize_pre_run(ast, global_scope);

        if let Err(err) = interpreter.borrow_mut().run(&world, main_file_source) {
            err.panic_error(main_file_source)
        }
    }
}
