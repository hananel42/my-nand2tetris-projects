use crate::symbols_table::{JackVariablesEngine, VarKind, VarType,};
use crate::tokenizer::Token::*;
use crate::tokenizer::{JackTokenizer, Keyword, Symbol, Token};
use crate::vm_writer::{LabelGenerator, PSegment, VMCommand, VMWriter};

pub enum CompilationErrorType {
    TokenizerError,
    UnexpectedToken,
    UndefinedVariable,
    UnexpectedEndOfInput,
    IOError,
}
pub struct CompilationError {
    pub error_type: CompilationErrorType,
    pub message: String,
    pub token: Option<Token>,
    pub line: i64,
    pub char: i64,
}

impl From<std::io::Error> for CompilationError {
    fn from(err: std::io::Error) -> Self {
        CompilationError {
            error_type: CompilationErrorType::IOError,
            message: err.to_string(),
            token: None,
            line: -1,
            char: -1,
        }
    }
}

// simple default display implementation for compilation errors
impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.error_type {
            CompilationErrorType::TokenizerError => {
                write!(f, "Tokenizer error: {}\n", self.message)?
            }
            CompilationErrorType::UnexpectedToken => {
                write!(f, "Unexpected token: {}\n", self.message)?
            }
            CompilationErrorType::UnexpectedEndOfInput => {
                write!(f, "Unexpected end of input: {}\n", self.message)?
            }
            CompilationErrorType::IOError => write!(f, "IO error: {}\n", self.message)?,
            CompilationErrorType::UndefinedVariable => {
                write!(f, "Undefined variable: {}\n", self.message)?
            }
        }
        if let Some(token) = &self.token {
            write!(
                f,
                "got {} \nat line {}, char {}",
                token, self.line, self.char
            )?;
        } else {
            write!(f, "at line {}, char {}", self.line, self.char)?;
        }

        Ok(())
    }
}

pub struct CompilationEngine<I: Iterator<Item = char>> {
    class_name: String,
    tokens: JackTokenizer<I>,
    vm_writer: VMWriter,
    label_generator: LabelGenerator,
    vars_engine: JackVariablesEngine,
}



// A macro to simplify token consumption and error handling
macro_rules! next_token {
    ($self:expr,$pattern:pat,$error:expr) => {{
        match $self.tokens.next() {
            Some(Ok(token @ $pattern)) => token,
            Some(Err(e)) => {
                return {
                    Err(CompilationError {
                        error_type: CompilationErrorType::TokenizerError,
                        message: e,
                        token: None,
                        line: $self.tokens.line(),
                        char: $self.tokens.char(),
                    })
                };
            }
            Some(Ok(x)) => {
                return Err(CompilationError {
                    error_type: CompilationErrorType::UnexpectedToken,
                    message: $error.to_string(),
                    token: Some(x),
                    line: $self.tokens.line(),
                    char: $self.tokens.char(),
                });
            }
            None => {
                return Err(CompilationError {
                    error_type: CompilationErrorType::UnexpectedEndOfInput,
                    message: $error.to_string(),
                    token: None,
                    line: $self.tokens.line(),
                    char: $self.tokens.char(),
                });
            }
        }
    }};
    ($self:expr, $pattern:pat => $ret:expr, $error:expr) => {{
        match $self.tokens.next() {
            Some(Ok($pattern)) => $ret,
            Some(Err(e)) => {
                return Err(CompilationError {
                    error_type: CompilationErrorType::TokenizerError,
                    message: e,
                    token: None,
                    line: $self.tokens.line(),
                    char: $self.tokens.char(),
                });
            }
            Some(Ok(x)) => {
                return Err(CompilationError {
                    error_type: CompilationErrorType::UnexpectedToken,
                    message: $error.to_string(),
                    token: Some(x),
                    line: $self.tokens.line(),
                    char: $self.tokens.char(),
                });
            }
            None => {
                return Err(CompilationError {
                    error_type: CompilationErrorType::UnexpectedEndOfInput,
                    message: $error.to_string(),
                    token: None,
                    line: $self.tokens.line(),
                    char: $self.tokens.char(),
                });
            }
        }
    }};
}
macro_rules! data_type {
    () => {
        Token::Keyword(Keyword::Int)
            | Token::Keyword(Keyword::Char)
            | Token::Keyword(Keyword::Boolean)
            | Token::Identifier(_)
    };
}
macro_rules! var_kind {
    () => {
        Token::Keyword(Keyword::Field | Keyword::Static)
    };
}
macro_rules! subroutine_type {
    () => {
        Token::Keyword(Keyword::Constructor | Keyword::Function | Keyword::Method)
    };
}
macro_rules! operator {
    () => {
        Token::Symbol(
            Symbol::Plus
                | Symbol::Minus
                | Symbol::Asterisk
                | Symbol::Slash
                | Symbol::Pipe
                | Symbol::Ampersand
                | Symbol::LessThan
                | Symbol::GreaterThan
                | Symbol::Equals,
        )
    };
}
macro_rules! constant_keyword {
    () => {
        Token::Keyword(Keyword::True | Keyword::False | Keyword::Null | Keyword::This)
    };
}
macro_rules! unary_op {
    () => {
        Token::Symbol(Symbol::Tilde | Symbol::Minus)
    };
}
macro_rules! expr_start {
    () => {
        Identifier(_)
            | IntegerConst(_)
            | StringConst(_)
            | constant_keyword!()
            | Symbol(Symbol::LeftParen)
    };
}

use PSegment::*;
use VMCommand::*;
impl<I: Iterator<Item = char>> CompilationEngine<I> {
    pub fn new(name: String, jack_tokenizer: JackTokenizer<I>, vm_writer: VMWriter) -> Self {
        Self {
            tokens: jack_tokenizer,
            vm_writer,
            vars_engine: JackVariablesEngine::new(),
            label_generator: LabelGenerator::new(name.clone()),
            class_name: name,
        }
    }
    pub fn compile(&mut self) -> Result<(), CompilationError> {
        self.compile_class()
    }
    fn compile_class(&mut self) -> Result<(), CompilationError> {
        let _ = next_token!(
            self,
            Token::Keyword(Keyword::Class),
            "Expected 'class' keyword"
        );
        let name = next_token!(self, Identifier(_), "Expected class name");
        if let Identifier(n) = &name
            && self.class_name != *n
        {
            return Err(CompilationError {
                error_type: CompilationErrorType::UnexpectedToken,
                message: format!("Class name mismatch: expected '{}'", self.class_name),
                token: Some(name),
                line: self.tokens.line(),
                char: self.tokens.char(),
            });
        }
        let _ = next_token!(self, Token::Symbol(Symbol::LeftBrace), "Expected '{'");
        while let Some(Ok(var_kind!())) = self.tokens.peek() {
            self.compile_class_var_dec()?;
        }
        while let Some(Ok(subroutine_type!())) = self.tokens.peek() {
            self.compile_subroutine()?;
        }
        let _ = next_token!(
            self,
            Token::Symbol(Symbol::RightBrace),
            "Expected class component or '}'"
        );
        Ok(())
    }

    fn compile_class_var_dec(&mut self) -> Result<(), CompilationError> {
        let kind = next_token!(self, var_kind!(), "Expected 'field' or 'static' keyword");
        let data_type = next_token!(self, data_type!(), "Expected type");

        let name = next_token!(self, Identifier(n) => n, "Expected variable name");
        let type_ = match data_type {
            Keyword(Keyword::Int) => VarType::Int,
            Keyword(Keyword::Char) => VarType::Char,
            Keyword(Keyword::Boolean) => VarType::Boolean,
            Identifier(n) => VarType::Class(n),
            _ => unreachable!(),
        };
        let kind = match kind {
            Keyword(Keyword::Static) => VarKind::Static,
            Keyword(Keyword::Field) => VarKind::Field,
            _ => unreachable!(),
        };
        self.vars_engine.define(name, type_.clone(), kind);

        while let Some(Ok(token)) = self.tokens.next() {
            match token {
                Token::Symbol(Symbol::Semicolon) => {
                    break;
                }
                Token::Symbol(Symbol::Comma) => {
                    let name = next_token!(self, Identifier(n) => n, "Expected variable name");
                    self.vars_engine.define(name, type_.clone(), kind);
                }
                _ => {
                    return Err(CompilationError {
                        error_type: CompilationErrorType::UnexpectedToken,
                        message: "Expected semicolon or comma".to_string(),
                        token: Some(token),
                        line: self.tokens.line(),
                        char: self.tokens.char(),
                    });
                }
            }
        }
        Ok(())
    }

    fn compile_subroutine(&mut self) -> Result<(), CompilationError> {
        self.vars_engine.enter_subroutine();
        let s_type = next_token!(
            self,
            Token::Keyword(t @ (Keyword::Constructor | Keyword::Function | Keyword::Method)) => t,
            "Expected subroutine statement"
        );
        let _return_type = next_token!(
            self,
            data_type!() | Keyword(Keyword::Void),
            "Expected return type"
        ); //we don't care about the return type.
        let name = next_token!(self, Identifier(name) => name, "Expected subroutine name");
        let _ = next_token!(self, Token::Symbol(Symbol::LeftParen), "Expected '('");

        if s_type == Keyword::Method {
            self.vars_engine.define(
                "this".to_string(),
                VarType::Class(self.class_name.clone()),
                VarKind::Arg,
            );
        }
        self.compile_parameter_list()?;

        let _ = next_token!(self, Token::Symbol(Symbol::RightParen), "Expected ')'");


        //subroutine body

        let _ = next_token!(self, Token::Symbol(Symbol::LeftBrace), "Expected '{'");
        while let Some(Ok(Keyword(Keyword::Var))) = self.tokens.peek() {
            self.compile_var_dec()?;
        }

        self.vm_writer.write_command(Function(
            format!("{}.{}", self.class_name, name.clone()),
            self.vars_engine.var_count(VarKind::Var) as u16,
        ))?;


        match s_type {
            Keyword::Constructor => {
                self.vm_writer.write_commands([
                    Push(CONSTANT, self.vars_engine.var_count(VarKind::Field) as u16),
                    Call("Memory.alloc".to_string(), 1),
                    Pop(POINTER, 0), //set THIS to the base address of the new object
                ])?;
            }
            Keyword::Function => {
                //do nothing here
            }
            Keyword::Method => {
                self.vm_writer.write_commands([
                    Push(ARG, 0),
                    Pop(POINTER, 0), //set THIS to the base address of the current object
                ])?;
            }
            _ => unreachable!(),
        }

        self.compile_statements()?;

        let _ = next_token!(self, Token::Symbol(Symbol::RightBrace), "Expected '}'");

        self.vars_engine.exit_subroutine();
        Ok(())
    }

    fn compile_var_dec(&mut self) -> Result<(), CompilationError> {
        let _ = next_token!(self, Token::Keyword(Keyword::Var), "Expected 'var' keyword");
        let type_ = next_token!(self, data_type!(), "Expected type");
        let name = next_token!(self, Identifier(name) => name, "Expected variable name");
        let data_type = match type_ {
            Keyword(Keyword::Int) => VarType::Int,
            Keyword(Keyword::Char) => VarType::Char,
            Keyword(Keyword::Boolean) => VarType::Boolean,
            Identifier(n) => VarType::Class(n),
            _ => unreachable!(),
        };
        self.vars_engine
            .define(name, data_type.clone(), VarKind::Var);
        while let Some(Ok(token)) = self.tokens.next() {
            match token {
                Symbol(Symbol::Semicolon) => {
                    break;
                }
                Symbol(Symbol::Comma) => {
                    let name =
                        next_token!(self, Identifier(name) => name, "Expected variable name");
                    self.vars_engine
                        .define(name, data_type.clone(), VarKind::Var);
                }
                _ => {
                    return Err(CompilationError {
                        error_type: CompilationErrorType::UnexpectedToken,
                        message: "Expected semicolon or comma".to_string(),
                        token: Some(token),
                        line: self.tokens.line(),
                        char: self.tokens.char(),
                    });
                }
            }
        }
        Ok(())
    }

    fn compile_parameter_list(&mut self) -> Result<(), CompilationError> {
        if let Some(Ok(t @ data_type!())) = self.tokens.peek() {
            let data_type = match t {
                Keyword(Keyword::Int) => VarType::Int,
                Keyword(Keyword::Char) => VarType::Char,
                Keyword(Keyword::Boolean) => VarType::Boolean,
                Identifier(n) => VarType::Class(n.clone()),
                _ => unreachable!(),
            };
            self.tokens.next();
            let name = next_token!(self, Identifier(name) => name, "Expected parameter name");
            self.vars_engine.define(name, data_type, VarKind::Arg);
            while let Some(Ok(Symbol(Symbol::Comma))) = self.tokens.peek() {
                self.tokens.next();
                let type_ = next_token!(self, data_type!(), "Expected data type");
                let data_type: VarType = type_.try_into().unwrap();

                let name = next_token!(self, Identifier(name) => name, "Expected parameter name");
                self.vars_engine.define(name, data_type, VarKind::Arg);
            }
        }
        Ok(())
    }
    fn compile_statements(&mut self) -> Result<(), CompilationError> {
        while let Some(Ok(token)) = self.tokens.peek() {
            match token {
                Token::Keyword(Keyword::Let) => self.compile_let()?,
                Token::Keyword(Keyword::If) => self.compile_if()?,
                Token::Keyword(Keyword::While) => self.compile_while()?,
                Token::Keyword(Keyword::Do) => self.compile_do()?,
                Token::Keyword(Keyword::Return) => self.compile_return()?,
                _ => break,
            }
        }
        Ok(())
    }
    fn compile_let(&mut self) -> Result<(), CompilationError> {
        let _ = next_token!(self, Token::Keyword(Keyword::Let), "Expected 'let' keyword");
        let name = next_token!(self, Identifier(name) => name, "Expected variable name");
        let var = self.vars_engine.get(&name).ok_or(CompilationError {
            error_type: CompilationErrorType::UndefinedVariable,
            message: format!("Undefined variable '{}'", name),
            token: Some(Identifier(name)),
            line: self.tokens.line(),
            char: self.tokens.char(),
        })?;
        let idx = var.index();
        let kind = var.kind();

        let mut is_array = false;

        if let Some(Ok(Token::Symbol(Symbol::LeftBracket))) = self.tokens.peek() {
            self.tokens.next();

            self.vm_writer.write_command(Push(kind.into(), idx))?;
            self.compile_expression()?; //and leave it at the top of the stack
            self.vm_writer.write_command(Add)?;
            let _ = next_token!(self, Token::Symbol(Symbol::RightBracket), "Expected ']'");
            is_array = true;
        }
        let _ = next_token!(self, Token::Symbol(Symbol::Equals), "Expected '='");

        self.compile_expression()?;

        if is_array {
            self.vm_writer.write_commands([
                Pop(TEMP, 0),    //store the expression value in a temp variable.
                Pop(POINTER, 1), //set THAT to the address of the array that  we calculated before.
                Push(TEMP, 0),   //push the expression value back to the stack.
                Pop(THAT, 0),    //pop the expression value into the array.
            ])?;
        } else {
            //simply pop the value into the variable
            self.vm_writer.write_command(Pop(kind.into(), idx))?;
        }

        let _ = next_token!(self, Token::Symbol(Symbol::Semicolon), "Expected ';'");
        Ok(())
    }

    fn compile_if(&mut self) -> Result<(), CompilationError> {
        let _ = next_token!(self, Token::Keyword(Keyword::If), "Expected 'if'");
        let _ = next_token!(self, Token::Symbol(Symbol::LeftParen), "Expected '('");
        self.compile_expression()?;
        let _ = next_token!(self, Token::Symbol(Symbol::RightParen), "Expected ')'");
        let _ = next_token!(self, Token::Symbol(Symbol::LeftBrace), "Expected '{'");

        let label = self.label_generator.generate_label("IF_FALSE");

        self.vm_writer
            .write_commands([Not, IfGoto(label.clone())])?;
        self.compile_statements()?;

        let _ = next_token!(self, Token::Symbol(Symbol::RightBrace), "Expected '}'");
        if let Some(Ok(Token::Keyword(Keyword::Else))) = self.tokens.peek() {
            self.tokens.next();
            let _ = next_token!(self, Token::Symbol(Symbol::LeftBrace), "Expected '{'");
            let end_label = self.label_generator.generate_label("IF_END");
            self.vm_writer
                .write_commands([Goto(end_label.clone()), Label(label)])?;
            self.compile_statements()?;
            self.vm_writer.write_command(Label(end_label))?;
            let _ = next_token!(self, Token::Symbol(Symbol::RightBrace), "Expected '}'");
        } else {
            self.vm_writer.write_command(Label(label))?;
        }
        Ok(())
    }
    fn compile_while(&mut self) -> Result<(), CompilationError> {
        let _ = next_token!(self, Token::Keyword(Keyword::While), "Expected 'while'");
        let _ = next_token!(self, Token::Symbol(Symbol::LeftParen), "Expected '('");
        let start_label = self.label_generator.generate_label("WHILE_START");
        let end_label = self.label_generator.generate_label("WHILE_END");
        self.vm_writer.write_command(Label(start_label.clone()))?;
        self.compile_expression()?;

        let _ = next_token!(self, Token::Symbol(Symbol::RightParen), "Expected ')'");
        let _ = next_token!(self, Token::Symbol(Symbol::LeftBrace), "Expected '{'");

        self.vm_writer
            .write_commands([Not, IfGoto(end_label.clone())])?;
        self.compile_statements()?;

        self.vm_writer
            .write_commands([Goto(start_label), Label(end_label)])?;
        let _ = next_token!(self, Token::Symbol(Symbol::RightBrace), "Expected '}'");
        Ok(())
    }

    fn compile_return(&mut self) -> Result<(), CompilationError> {
        let _ = next_token!(self, Token::Keyword(Keyword::Return), "Expected 'return'");
        if matches!(
            self.tokens.peek(),
            Some(Ok(Token::Symbol(Symbol::Semicolon)))
        ) {
            self.vm_writer.write_command(Push(CONSTANT, 0))?; //push 0 for void return
        } else {
            self.compile_expression()?;
        }

        self.vm_writer.write_command(Return)?;

        let _ = next_token!(self, Token::Symbol(Symbol::Semicolon), "Expected ';'");

        Ok(())
    }
    fn compile_do(&mut self) -> Result<(), CompilationError> {
        let _ = next_token!(self, Keyword(Keyword::Do), "Expected 'do'");
        let mut ident = next_token!(self, Identifier(name) => name, "Expected subroutine name");
        let mut is_method = false;
        if let Some(Ok(Symbol(Symbol::Dot))) = self.tokens.peek() {
            self.tokens.next();
            let fn_ident = next_token!(self, Identifier(name) => name, "Expected function identifier");


            if let Some(var) = self.vars_engine.get(ident.as_str()){
                if let VarType::Class(class_name) = var.type_() {
                    let idx = var.index();
                    let kind = var.kind();
                    ident = format!("{}.{}", class_name, fn_ident);
                    self.vm_writer.write_command(Push(kind.into(), idx))?; //push the object
                    is_method = true;
                } else {
                    return Err(CompilationError {
                        error_type: CompilationErrorType::UnexpectedToken,
                        message: "Expected the variable to be an object".to_string(),
                        token: Some(Identifier(ident)),
                        line: self.tokens.line(),
                        char: self.tokens.char(),
                    });
                }

            }
            else {
                ident = format!("{}.{}", ident, fn_ident); //ClassName.functionName - static function call
            }
        }
        else {
            is_method = true;
            self.vm_writer.write_command(Push(POINTER, 0))?; //push the current object
            ident = format!("{}.{}", self.class_name, ident);
        }
        let _ = next_token!(self, Token::Symbol(Symbol::LeftParen), "Expected '('");
        let arg_count = self.compile_expression_list()?;

        self.vm_writer.write_commands([
            Call(ident, arg_count + if is_method {1} else {0}),
            Pop(TEMP,0) //discard the return value of the function
        ])?;




        let _ = next_token!(self, Token::Symbol(Symbol::RightParen), "Expected ')'");
        let _ = next_token!(self, Token::Symbol(Symbol::Semicolon), "Expected ';'");
        Ok(())
    }

    fn compile_expression(&mut self) -> Result<(), CompilationError> {
        self.compile_term()?;
        while let Some(Ok(op @ operator!())) = self.tokens.peek() {
            let command = match op {
                Token::Symbol(Symbol::Plus) => Add,
                Token::Symbol(Symbol::Minus) => Sub,
                Token::Symbol(Symbol::Asterisk) => Call("Math.multiply".to_string(), 2),
                Token::Symbol(Symbol::Slash) => Call("Math.divide".to_string(), 2),
                Token::Symbol(Symbol::Pipe) => OR,
                Token::Symbol(Symbol::Ampersand) => AND,
                Token::Symbol(Symbol::LessThan) => LT,
                Token::Symbol(Symbol::GreaterThan) => GT,
                Token::Symbol(Symbol::Equals) => EQ,
                _ => unreachable!(),
            };
            self.tokens.next();
            self.compile_term()?;
            self.vm_writer.write_command(command)?;
        }
        Ok(())
    }

    fn compile_term(&mut self) -> Result<(), CompilationError> {
        if let Some(Ok(token)) = self.tokens.next() {
            match token {
                kwd @ constant_keyword!() => {
                    match kwd {
                        Keyword(Keyword::True) => {
                            self.vm_writer.write_commands([
                                Push(CONSTANT, 0),
                                Not
                            ])?;
                        }
                        Keyword(Keyword::False) | Keyword(Keyword::Null) => {
                            self.vm_writer.write_command(Push(CONSTANT, 0))?;
                        }
                        Keyword(Keyword::This) => {
                            self.vm_writer.write_command(Push(POINTER,0))?;
                        }
                        _=> unreachable!()
                    }
                }
                unary_op @ unary_op!() => {
                    let command = match unary_op {
                        Symbol(Symbol::Minus) => Neg,
                        Symbol(Symbol::Tilde) => Not,
                        _ => unreachable!()
                    };
                    self.compile_term()?;
                    self.vm_writer.write_command(command)?;

                }
                Symbol(Symbol::LeftParen) => {
                    self.compile_expression()?;
                    let _ = next_token!(self, Token::Symbol(Symbol::RightParen), "Expected ')'");
                }
                IntegerConst(num) => {
                    self.vm_writer.write_command(Push(CONSTANT, num as u16))?;
                }
                StringConst(string) => {
                    let length = string.len() as u16;
                    self.vm_writer.write_commands([
                        Push(CONSTANT, length),
                        Call("String.new".to_string(),1),
                    ])?;
                    for c in string.chars() {
                        self.vm_writer.write_commands([
                            Push(CONSTANT, c as u16),
                            Call("String.appendChar".to_string(),2),
                        ])?;
                    }
                }
                Identifier(ident) => {
                    if let Some(Ok(token)) = self.tokens.peek() {
                        match token {
                            Symbol(Symbol::Dot | Symbol::LeftParen) => {
                                let is_dot = matches!(token, Symbol(Symbol::Dot));
                                let mut is_method = true;
                                self.tokens.next();
                                let mut full_name = format!("{}.{}",self.class_name,ident);
                                if is_dot {
                                    let identifier = next_token!(self,Identifier(n) => n,"Expected subroutine name");
                                    full_name = format!("{}.{}", ident, identifier);

                                    if let Some(var) = self.vars_engine.get(ident.as_str()) {
                                        if let VarType::Class(class_name) = var.type_() {
                                            let idx = var.index();
                                            let kind = var.kind();
                                            full_name = format!("{}.{}", class_name, identifier);
                                            self.vm_writer.write_command(Push(kind.into(), idx))?; //push the object
                                        } else {
                                            return Err(CompilationError {
                                                error_type: CompilationErrorType::UnexpectedToken,
                                                message: "Expected the variable to be an object".to_string(),
                                                token: Some(Identifier(ident)),
                                                line: self.tokens.line(),
                                                char: self.tokens.char(),
                                            });
                                        }
                                    } else {
                                        is_method = false;
                                    }
                                    let _ = next_token!(self,Token::Symbol(Symbol::LeftParen),"Expected '('");
                                } else {
                                    self.vm_writer.write_command(Push(POINTER,0))?;
                                }


                                let var_count = self.compile_expression_list()?;
                                self.vm_writer.write_command(Call(full_name,var_count + if is_method {1} else {0} ))?;
                                let _ = next_token!(self,Token::Symbol(Symbol::RightParen),"Expected ')'");
                            } //fn call
                            Symbol(Symbol::LeftBracket) => {
                                self.tokens.next();
                                let var = self.vars_engine.get(ident.as_str()).ok_or_else(|| CompilationError {
                                    error_type: CompilationErrorType::UndefinedVariable,
                                    message: format!("Undefined variable '{}'", ident),
                                    token: Some(Identifier(ident.clone())),
                                    line: self.tokens.line(),
                                    char: self.tokens.char(),
                                })?;
                                let idx = var.index();
                                let kind = var.kind();
                                self.compile_expression()?;
                                self.vm_writer.write_commands([
                                    Push(kind.into(), idx),
                                    Add,
                                    Pop(POINTER,1),
                                    Push(THAT,0),

                                ])?;
                                let _ = next_token!(self,Token::Symbol(Symbol::RightBracket),"Expected ']'");
                            } //array
                            _ => {
                                let var = self.vars_engine.get(ident.as_str()).ok_or_else(|| CompilationError {
                                    error_type: CompilationErrorType::UndefinedVariable,
                                    message: "Undefined variable".to_string(),
                                    token: Some(Identifier(ident.clone())),
                                    line: self.tokens.line(),
                                    char: self.tokens.char(),
                                })?;

                                let idx = var.index();
                                let kind = var.kind();
                                self.vm_writer.write_command(Push(kind.into(), idx))?;
                            } //var
                        }
                    }
                    // Else, if the tokenizer returns an error, or if there is no next token,
                    // we don't have to do anything here; the called function will handle it as an error because jack program cannot end with an expression
                    // Optional: TODO: catch it as a varName
                }
                _ => {return Err(CompilationError {
                    error_type: CompilationErrorType::UnexpectedToken,
                    message: "Expected a term".to_string(),
                    token: Some(token),
                    line: self.tokens.line(),
                    char: self.tokens.char(),
                })}
            }
        }
        Ok(())
    }

    fn compile_expression_list(&mut self) -> Result<u16, CompilationError> {
        let mut count = 0;
        if let Some(Ok(expr_start!())) = self.tokens.peek() {
            self.compile_expression()?;
            count += 1;
        }
        while let Some(Ok(Symbol(Symbol::Comma))) = self.tokens.peek() {
            self.tokens.next();
            self.compile_expression()?;
            count += 1;
        }
        Ok(count)
    }
}
