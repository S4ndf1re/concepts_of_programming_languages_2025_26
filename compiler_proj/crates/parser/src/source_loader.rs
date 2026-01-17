use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use parser_types::Error;

pub trait SourceLoader {
    /// load the main file of a source object
    fn load_main_file(&self) -> Result<&'static str, Error>;
    /// Load a single file by name
    fn load_file(&self, filename: &Path) -> Result<&'static str, Error>;

    fn empty_string(&self) -> &'static str {
        ""
    }
}

pub struct StaticSourceLoader {
    content: &'static String,
}

impl SourceLoader for StaticSourceLoader {
    fn load_main_file(&self) -> Result<&'static str, Error> {
        Ok(self.content)
    }

    fn load_file(&self, _filename: &Path) -> Result<&'static str, Error> {
        Err(Error::IoError("File not found".to_owned()))
    }
}

impl From<String> for StaticSourceLoader {
    fn from(content: String) -> Self {
        let content_ref = Box::into_raw(Box::new(content));
        unsafe {
            Self {
                content: &*content_ref,
            }
        }
    }
}

impl Drop for StaticSourceLoader {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.content as *const String as *mut String));
        }
    }
}

pub struct FileSourceLoader {
    root: PathBuf,
    main_file: PathBuf,
    buffered: Rc<RefCell<HashMap<String, &'static String>>>,
}

impl FileSourceLoader {
    pub fn new(root: PathBuf, main_file: PathBuf) -> Self {
        Self {
            root,
            main_file,
            buffered: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn add_buffered(&self, path: String, value: &'static String) {
        self.buffered.borrow_mut().insert(path, value);
    }

    fn resolve_buffered(&self, path: &str) -> Option<&'static String> {
        self.buffered.borrow().get(path).copied()
    }
}

impl SourceLoader for FileSourceLoader {
    fn load_main_file(&self) -> Result<&'static str, Error> {
        let mut complete_main_path = PathBuf::new();
        complete_main_path.push(&self.root);
        complete_main_path.push(&self.main_file);
        let main_path = complete_main_path.to_string_lossy().into_owned();

        if let Some(value) = self.resolve_buffered(&main_path) {
            return Ok(value);
        }

        let file = File::open(complete_main_path.as_path());
        let Ok(file) = file else {
            return Err(Error::IoError(file.unwrap_err().to_string()));
        };

        let content = io::read_to_string(file);
        let Ok(content) = content else {
            return Err(Error::IoError(content.unwrap_err().to_string()));
        };

        let content_ref = Box::into_raw(Box::new(content));
        unsafe {
            let static_str = &*content_ref;
            self.add_buffered(main_path, static_str);
            Ok(static_str)
        }
    }

    fn load_file(&self, filename: &Path) -> Result<&'static str, Error> {
        let mut complete_path = PathBuf::new();
        complete_path.push(&self.root);
        complete_path.push(filename);
        let complete_path_str = complete_path.to_string_lossy().into_owned();

        if let Some(value) = self.resolve_buffered(&complete_path_str) {
            return Ok(value);
        }

        let file = File::open(complete_path.as_path());
        let Ok(file) = file else {
            return Err(Error::IoError(file.unwrap_err().to_string()));
        };

        let content = io::read_to_string(file);
        let Ok(content) = content else {
            return Err(Error::IoError(content.unwrap_err().to_string()));
        };

        let content_ref = Box::into_raw(Box::new(content));
        unsafe {
            let static_str = &*content_ref;
            self.add_buffered(complete_path_str, static_str);
            Ok(static_str)
        }
    }
}

impl Drop for FileSourceLoader {
    fn drop(&mut self) {
        for file in self.buffered.borrow().iter() {
            unsafe {
                drop(Box::from_raw(*file.1 as *const String as *mut String));
            }
        }
    }
}
