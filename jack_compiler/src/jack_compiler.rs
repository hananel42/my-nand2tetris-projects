use std::env;
use std::fs::{read_to_string, File};
use std::io::Read;
use std::path::Path;
use crate::compilation_engine::{CompilationEngine, CompilationError, CompilationErrorType};
use crate::tokenizer::JackTokenizer;
use crate::vm_writer::VMWriter;

pub fn compile_file(path: impl AsRef<Path>) -> Result<(), CompilationError> {
    let path = path.as_ref();
    let name = path.file_stem().ok_or_else(|| {
        return CompilationError {
            error_type: CompilationErrorType::IOError,
            message: "File not found".to_string(),
            token: None,
            line: -1,
            char: -1,
        }
    })?.to_str().ok_or_else(|| {
        return CompilationError {
            error_type: CompilationErrorType::IOError,
            message: "Illegal file name".to_string(),
            token: None,
            line: -1,
            char: -1,
        }
    })?.to_string();
    let mut file = File::open(path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let mut engine = CompilationEngine::new(
        name.clone(),
        JackTokenizer::new(content.chars()),
        VMWriter::new(format!("{}.vm", name))?,
    );

    engine.compile()?;
    Ok(())
}


pub fn compile_directory(path: impl AsRef<Path>) -> Result<(), CompilationError> {

    let path = path.as_ref();
    env::set_current_dir(path)?;
    for entry in path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "jack") {
            compile_file(path)?;
        }
    }
    Ok(())
}