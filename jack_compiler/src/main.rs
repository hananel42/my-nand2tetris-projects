mod compilation_engine;
mod symbols_table;
mod tokenizer;
mod vm_writer;
mod jack_compiler;

use std::env;
use std::time::Instant;
use crate::vm_writer::VMWriter;
use compilation_engine::CompilationEngine;
use tokenizer::JackTokenizer;

fn main() {
    match env::args().nth(1) {
        Some(arg) => {
            let time =Instant::now();
            if arg.ends_with(".jack") {
                if let Err(e) = jack_compiler::compile_file(arg) {
                    eprintln!("Error compiling file:\n {}", e);
                    std::process::exit(1);
                }
            } else {
                if let Err(e) = jack_compiler::compile_directory(arg) {
                    eprintln!("Error compiling directory:\n {}", e);
                    std::process::exit(1);
                }
            }
            println!("Compilation completed in {:.2?} ms", time.elapsed().as_millis());
        }
        None => {
            eprintln!("Usage: jack_compiler <file.jack|directory>");
        }
    }

}
