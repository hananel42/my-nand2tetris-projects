use crate::symbols_table::{VarKind, VarType};
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

pub enum PSegment {
    LCL,
    ARG,
    THIS,
    THAT,
    POINTER,
    STATIC,
    CONSTANT,
    TEMP,
}
impl std::fmt::Display for PSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PSegment::LCL => write!(f, "local"),
            PSegment::ARG => write!(f, "argument"),
            PSegment::THIS => write!(f, "this"),
            PSegment::THAT => write!(f, "that"),
            PSegment::POINTER => write!(f, "pointer"),
            PSegment::STATIC => write!(f, "static"),
            PSegment::CONSTANT => write!(f, "constant"),
            PSegment::TEMP => write!(f, "temp"),
        }
    }
}
pub enum VMCommand {
    Function(String, u16),
    Call(String, u16),
    Return,
    Label(String),
    Goto(String),
    IfGoto(String),
    Push(PSegment, u16),
    Pop(PSegment, u16),
    Add,
    Sub,
    Neg,
    GT,
    EQ,
    LT,
    AND,
    OR,
    Not,
}

impl From<VarKind> for PSegment {
    fn from(var_kind: VarKind) -> Self {
        match var_kind {
            VarKind::Var => PSegment::LCL,
            VarKind::Arg => PSegment::ARG,
            VarKind::Static => PSegment::STATIC,
            VarKind::Field => PSegment::THIS,
        }
    }
}
impl std::fmt::Display for VMCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VMCommand::Function(name, n) => write!(f, "function {} {}", name, n),
            VMCommand::Call(name, n) => write!(f, "call {} {}", name, n),
            VMCommand::Return => write!(f, "return"),
            VMCommand::Label(label) => write!(f, "label {}", label),
            VMCommand::Goto(label) => write!(f, "goto {}", label),
            VMCommand::IfGoto(label) => write!(f, "if-goto {}", label),
            VMCommand::Push(segment, index) => {
                write!(f, "push {} {}", segment, index)
            }
            VMCommand::Pop(segment, index) => {
                write!(f, "pop {} {}", segment, index)
            }
            VMCommand::Add => write!(f, "add"),
            VMCommand::Sub => write!(f, "sub"),
            VMCommand::Neg => write!(f, "neg"),
            VMCommand::GT => write!(f, "gt"),
            VMCommand::EQ => write!(f, "eq"),
            VMCommand::LT => write!(f, "lt"),
            VMCommand::AND => write!(f, "and"),
            VMCommand::OR => write!(f, "or"),
            VMCommand::Not => write!(f, "not"),
        }
    }
}

pub struct LabelGenerator {
    counters: HashMap<String, usize>,
    class_name: String,
}
impl LabelGenerator {
    pub fn new(class_name: String) -> Self {
        LabelGenerator {
            counters: HashMap::new(),
            class_name,
        }
    }

    pub fn generate_label(&mut self, prefix: &str) -> String {
        let count = self.counters.entry(prefix.to_string()).or_insert(0);
        let label = format!("{}_{}_{}", self.class_name, prefix, count);
        *count += 1;
        label
    }
}

pub struct VMWriter {
    file: BufWriter<fs::File>,
}

impl VMWriter {
    pub fn new<P: AsRef<Path>>(dest: P) -> std::io::Result<Self> {
        let file = BufWriter::new(fs::File::create(dest)?);
        Ok(VMWriter { file })
    }
    pub fn write_command(&mut self, command: VMCommand) -> std::io::Result<()> {
        writeln!(self.file, "{}", command)
    }

    pub fn write_comment(&mut self, comment: &str) -> std::io::Result<()> {
        writeln!(self.file, "// {}", comment)
    }

    pub fn write_commands<const I: usize>(
        &mut self,
        commands: [VMCommand; I],
    ) -> std::io::Result<()> {
        for command in commands {
            self.write_command(command)?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> std::io::Result<()> {
        self.file.flush()?;
        Ok(())
    }
}
