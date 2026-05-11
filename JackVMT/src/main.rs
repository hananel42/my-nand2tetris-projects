use std::fs::{read_dir, File};
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;

enum PSegment{
    LCL,
    ARG,
    THIS,
    THAT,
    POINTER,
    STATIC,
    CONSTANT,
    TEMP
}
impl PSegment{
    fn as_str(&self) -> &str{
        match self {
            PSegment::LCL => "LCL",
            PSegment::ARG => "ARG",
            PSegment::THIS => "THIS",
            PSegment::THAT => "THAT",
            PSegment::POINTER => "POINTER",
            PSegment::STATIC => "STATIC",
            PSegment::CONSTANT => "CONSTANT",
            PSegment::TEMP => "TEMP"
        }
    }
}
enum VMCommand {
    Function(String,u16),
    Call(String,u16),
    Return,
    Label(String),
    Goto(String),
    IfGoto(String),
    Push(PSegment,u16),
    Pop(PSegment,u16),
    Add,
    Sub,
    Neg,
    GT,
    EQ,
    LT,
    AND,
    OR,
    Not
}
impl VMCommand{
    fn to_asm(&self,line_number:usize,file_name: &str) -> String{
        use VMCommand::*;
        use PSegment::*;
        match self {
            Function(name, locals) => {
                let init_locals = if *locals == 0 {"//there is no locals".to_string()} else {
                    format!(
"
@{locals}
D=A
(loop_{line_number})
@SP
M=M+1
A=M-1
M=0
@loop_{line_number}
D=D-1;JNE
"
                    )
                };
                format!("(${name}){init_locals}")
            }
            Call(name, args) => {
                let args_plus_five = args+5;
                format!(
"@retAddr{line_number}
D=A
@R13
M=D
@{args_plus_five}
D=A
@R14
M=D
@${name}
D=A
@R15
M=D
@DO_CALL
0;JMP
(retAddr{line_number})
"
                )
            }
            Return => {
"
@DO_RETURN
0;JMP
".to_string()

            }
            Label(name) => {
                format!("(${name})")
            } //wrap with $ for safety
            Goto(label) => {
                format!(
"@${label}
0;JMP"
                )
            }
            IfGoto(label) => {
                format!(
"@SP
AM=M-1
D=M
@${label}
D;JNE"

                )
            }
            Push(segment, index) => {
                match segment {
                    seg @ (LCL | ARG | THIS| THAT) => {
                        let segment = seg.as_str();
                        format!(
"@{segment}
D=M
@{index}
A=D+A
D=M
@SP
M=M+1
A=M-1
M=D"
                        )
                    }
                    seg @ (POINTER|TEMP) => {
                        let pointer = match seg {
                            POINTER => if *index == 1  { "THAT" } else { "THIS" } . to_string() ,
                            _ => (5+index).to_string()
                        };
                        format!(
                            "@{pointer}
D=M
@SP
M=M+1
A=M-1
M=D")
                    }
                    STATIC => {format!(
                        "@${file_name}.{index}
D=M
@SP
M=M+1
A=M-1
M=D"
                    )
                    }
                    CONSTANT => {format!(
"@{index}
D=A
@SP
M=M+1
A=M-1
M=D"
                    )
                    }
                }
            }
            Pop(segment, index) => {
                match segment {
                    seg @ (LCL | ARG | THIS| THAT) => {
                        let segment = seg.as_str();
                        format!(
                            "@{segment}
D=M
@{index}
D=D+A
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D"
                        )
                    }
                    seg @ (POINTER|TEMP) => {
                        let idx = match seg {
                            POINTER => if *index == 1  { "THAT" } else { "THIS" } . to_string() ,
                            _ => (5+index).to_string()
                        };
                        format!(
                            "@SP
AM=M-1
D=M
@{idx}
M=D")
                    }
                    STATIC => {format!(
                        "@SP
AM=M-1
D=M
@${file_name}.{index}
M=D"
                    )
                    }
                    CONSTANT => unreachable!()
                }
            }
            op @ (GT | EQ | LT) => {
                let jump = match op {
                    GT => "JGT",
                    LT => "JLT",
                    _ => "JEQ"
                };
                format!(
"@SP
AM=M-1
D=M
A=A-1
D=M-D
@COMP_TRUE{line_number}
D;{jump}
@SP
A=M-1
M=0
@COMP_END{line_number}
0;JMP
(COMP_TRUE{line_number})
@SP
A=M-1
M=-1
(COMP_END{line_number})
"
                )
            }
            op @ (Add | Sub | OR | AND) => {
                let operator = match op {
                    Add => "+",
                    Sub => "-",
                    AND => "&",
                    _ => "|",

                };
                format!(
"@SP
AM=M-1
D=M
A=A-1
M=M{operator}D"
                )
            }
            op @ (Neg | Not) => {
                let operator = match op {
                    Neg => '-',
                    _ => '!'
                };
                format!(
"@SP
A=M-1
M={operator}M",
                )
            }
        }


    }
}


#[derive(Debug)]
struct Project {
    files: Vec<PathBuf>,
    output_file: PathBuf,
}
impl Project {
    fn new(path: &str) -> Result<Project,String> {
        let path = PathBuf::from(path);
        if !path.exists() {return Err(format!("{} does not exist", path.display()));}
        let is_dir = path.is_dir();
        let mut files = Vec::new();
        if is_dir {
            if let Ok(itr) = read_dir(&path) {
                for entry in itr {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().and_then(|f| f.to_str()) == Some("vm") {
                            files.push(path);
                        }
                    } else {
                        return Err(format!("Could not read directory {:?}", path));
                    }
                }
            } else {
                return Err(format!("Could not read directory {:?}", path));
            }

        } else {
            files.push(path.clone());
        }


        let output_file = Self::get_output_path(&path)?;

        Ok(Project{ files, output_file })
    }
    fn compile(self) -> Option<String> {
        let mut line_number:usize = 0;
        let tmp_out_file = self.output_file.with_extension("tmp");
        let output_file = match File::create(&tmp_out_file) {
            Ok(f) => {f }
            Err(_) => {return Some(format!("Could not create output file {}", self.output_file.display()))}
        };
        let mut writer = BufWriter::new(output_file);
        let init = include_str!("./init.asm");
        match writeln!(writer, "{}", init) {
            Ok(_) => {}
            Err(_) => {return Some(format!("Could not write to output file {}", self.output_file.display()))}
        }

        for file in self.files {

            let mut this_file_line:usize = 0;
            let opened_file = if let Ok(file_) = File::open(&file) {file_} else {return Some("cannot open file".to_string()) };
            let reader = BufReader::new(opened_file);
            let file_name = match &file.file_stem() {
                None => {return Some("os error".to_string());}
                Some(s) => {s.to_string_lossy()}
            };
            match writeln!(writer,"//------{}---------",file_name) {
                Ok(_) => {}
                Err(_) => {return Some(format!("Could not write to output file {}", self.output_file.display()))}
            }
            for line in reader.lines() {
                this_file_line += 1;
                line_number = line_number+1;
                let line = match line {
                    Ok(line) => {line }
                    Err(_) => {return Some("cannot read line".to_string()) }
                };
                let ast = match parser(&line) {
                    Ok(a) => {a}
                    Err(e) => {return Some(format!("Error at file {}, line {this_file_line}:\n{e}",file.to_string_lossy()));}
                };
                if let Some(ast) = ast {
                    let asm = ast.to_asm(line_number,&file_name);
                    match writeln!(writer, "{}", asm) {
                        Ok(_) => {}
                        Err(_) => {return Some(format!("Could not write to file {}", file.display()))}
                    }
                }
            }
        }
        match fs::rename(&tmp_out_file, &self.output_file){
            Ok(_) => {None}
            Err(_) => {Some(format!("Could not move output file {}", self.output_file.display()))}
        }

    }
    fn get_output_path(input: &Path) -> Result<PathBuf, String> {
        if !input.exists() {
            return Err("path does not exist".to_string());
        }

        let is_dir = input.is_dir();

        let base_name = if is_dir {
            input.file_name()
        } else {
            input.file_stem()
        }
            .ok_or("invalid file name")?
            .to_string_lossy();

        let output = if is_dir {
            let mut p = input.to_path_buf();
            p.push(format!("{base_name}.asm"));
            p
        } else {
            let mut p = input.to_path_buf();
            p.set_extension("asm");
            p
        };

        Ok(output)
    }
}



fn parser(line:&str) -> Result<Option<VMCommand>,String> {
    let mut itr = line.split_whitespace();
    if let Some(cmd) = itr.next() {
        let vmc = match cmd {
            "add" => VMCommand::Add,
            "sub" => VMCommand::Sub,
            "neg" => VMCommand::Neg,
            "not" => VMCommand::Not,
            "gt" => VMCommand::GT,
            "lt" => VMCommand::LT,
            "eq" => VMCommand::EQ,
            "or" => VMCommand::OR,
            "and" => VMCommand::AND,
            "return" => VMCommand::Return,
            op @ ("label"|"goto"|"if-goto") => {
                if let Some(name) = itr.next(){
                    match op {
                        "label" => VMCommand::Label(name.to_string()),
                        "goto" => VMCommand::Goto(name.to_string()),
                        _ => {VMCommand::IfGoto(name.to_string())},
                    }
                }else { return Err(format!("Incomplete {}",op)) }
            }
            op @ ("push"|"pop"|"function"|"call") => {
                if let Some(s) = itr.next() && let Some(num_) = itr.next() && let Ok(num) = num_.parse::<u16>(){
                    match op {
                        "function" => VMCommand::Function(s.to_string(), num),
                        "call" => VMCommand::Call(s.to_string(), num),
                        _ => {
                            let segment = match s {
                                "local" => PSegment::LCL,
                                "argument" => PSegment::ARG,
                                "this" => PSegment::THIS,
                                "that" => PSegment::THAT,
                                "pointer" => {
                                    if num >1 {
                                        return Err("Illegal pointer index".to_string())
                                    } else {
                                        PSegment::POINTER
                                    }
                                },
                                "static" => PSegment::STATIC,
                                "temp" => {
                                    if 10>num {PSegment::TEMP} else {
                                        return Err("Illegal temp index".to_string())
                                    }
                                }
                                "constant" => PSegment::CONSTANT,
                                _ => {return Err(format!("unknown segment: {}",s))}
                            };
                            match op {
                                "push" => VMCommand::Push(segment, num),
                                _ => {
                                    match segment {
                                        PSegment::CONSTANT => {return Err("can not pop into constant".to_string())}
                                        _ => VMCommand::Pop(segment, num)
                                    }

                                }
                            }
                        }
                    }
                }else { return Err(format!("Incomplete {}",op)) }
            },
            comment if comment.starts_with("//") => {return Ok(None);}
            _ => {return Err(format!("Unknown command: {}", cmd))}
        };
        if let Some(text) = itr.next() && !text.starts_with("//") {
            return Err(format!("Unknown command: {}", text))
        }
        Ok(Some(vmc))
    } else {
        Ok(None)
    }
}





fn main() -> Result<(), Box<dyn std::error::Error>>{
    let start = Instant::now();
    if let Some(path) = std::env::args().nth(1) {
        let project = Project::new(path.as_str())?;
        let tmp_output_file = format!("{}.tmp",project.output_file.display());
        match project.compile() {
            None => {println!("Compiled successfully. took {:.6}s",start.elapsed().as_secs_f64());}
            Some(error) => {
                println!("{}", error);
                fs::remove_file(tmp_output_file)?;
            }
        }
    } else {
        println!("Usage : JackVMT <path>")
    }






    Ok(())
}
