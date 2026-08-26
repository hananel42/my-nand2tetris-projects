use Token::*;
use std::iter::Peekable;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}
impl Keyword {
    pub fn len(&self) -> usize {
        match self {
            Keyword::Constructor => 11,
            Keyword::Function => 8,
            Keyword::Boolean => 7,
            Keyword::Return | Keyword::Static | Keyword::Method => 6,
            Keyword::While | Keyword::False | Keyword::Class | Keyword::Field => 5,
            Keyword::Null
            | Keyword::This
            | Keyword::Else
            | Keyword::True
            | Keyword::Void
            | Keyword::Char => 4,
            Keyword::Var | Keyword::Int | Keyword::Let => 3,
            Keyword::Do | Keyword::If => 2,
        }
    }
}
impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Keyword::Class => write!(f, "class"),
            Keyword::Constructor => write!(f, "constructor"),
            Keyword::Function => write!(f, "function"),
            Keyword::Method => write!(f, "method"),
            Keyword::Field => write!(f, "field"),
            Keyword::Static => write!(f, "static"),
            Keyword::Var => write!(f, "var"),
            Keyword::Int => write!(f, "int"),
            Keyword::Char => write!(f, "char"),
            Keyword::Boolean => write!(f, "boolean"),
            Keyword::Void => write!(f, "void"),
            Keyword::True => write!(f, "true"),
            Keyword::False => write!(f, "false"),
            Keyword::Null => write!(f, "null"),
            Keyword::This => write!(f, "this"),
            Keyword::Let => write!(f, "let"),
            Keyword::Do => write!(f, "do"),
            Keyword::If => write!(f, "if"),
            Keyword::Else => write!(f, "else"),
            Keyword::While => write!(f, "while"),
            Keyword::Return => write!(f, "return"),
        }
    }
}
#[derive(Debug)]
pub enum Symbol {
    LeftBrace,    // '{'
    RightBrace,   // '}'
    LeftParen,    // '('
    RightParen,   // ')'
    LeftBracket,  // '['
    RightBracket, // ']'
    Dot,          // '.'
    Comma,        // ','
    Semicolon,    // ';'
    Plus,         // '+'
    Minus,        // '-'
    Asterisk,     // '*'
    Slash,        // '/'
    Pipe,         // '|'
    Ampersand,    // '&'
    LessThan,     // '<'
    GreaterThan,  // '>'
    Equals,       // '='
    Tilde,        // '~'
}
impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Symbol::LeftBrace => write!(f, "{{"),
            Symbol::RightBrace => write!(f, "}}"),
            Symbol::LeftParen => write!(f, "("),
            Symbol::RightParen => write!(f, ")"),
            Symbol::LeftBracket => write!(f, "["),
            Symbol::RightBracket => write!(f, "]"),
            Symbol::Dot => write!(f, "."),
            Symbol::Comma => write!(f, ","),
            Symbol::Semicolon => write!(f, ";"),
            Symbol::Plus => write!(f, "+"),
            Symbol::Minus => write!(f, "-"),
            Symbol::Asterisk => write!(f, "*"),
            Symbol::Slash => write!(f, "/"),
            Symbol::Pipe => write!(f, "|"),
            Symbol::Ampersand => write!(f, "&amp;"),
            Symbol::LessThan => write!(f, "&lt;"),
            Symbol::GreaterThan => write!(f, "&gt;"),
            Symbol::Equals => write!(f, "="),
            Symbol::Tilde => write!(f, "~"),
        }
    }
}
#[derive(Debug)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    IntegerConst(i16),
    StringConst(String),
    Identifier(String),
}

impl Token {
    pub fn to_xml(&self) -> String {
        match self {
            Keyword(keyword) => format!("<keyword> {} </keyword>", keyword),
            Symbol(symbol) => format!("<symbol> {} </symbol>", symbol),
            IntegerConst(value) => format!("<integerConstant> {} </integerConstant>", value),
            StringConst(value) => format!("<stringConstant> {} </stringConstant>", value),
            Identifier(value) => format!("<identifier> {} </identifier>", value),
        }
    }
    pub fn length(&self) -> usize {
        match self {
            Keyword(keyword) => keyword.len(),
            Symbol(symbol) => 1, // All symbols are single characters
            IntegerConst(value) => {
                if *value == 0 {
                    return 1;
                }
                (*value as f64).log10().floor() as usize + 1
            }
            StringConst(value) => value.len(),
            Identifier(value) => value.len(),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Keyword(keyword) => write!(f, "{}", keyword),
            Symbol(symbol) => write!(f, "{}", symbol),
            IntegerConst(value) => write!(f, "{}", value),
            StringConst(value) => write!(f, "\"{}\"", value),
            Identifier(value) => write!(f, "{}", value),
        }
    }
}

struct TrackingIterator<I: Iterator<Item = char>> {
    inner: Peekable<I>,
    line: i64,
    char: i64,
}
impl<I: Iterator<Item = char>> TrackingIterator<I> {
    fn new(inner: I) -> Self {
        TrackingIterator {
            inner: inner.peekable(),
            line: 1,
            char: 0,
        }
    }
    fn peek(&mut self) -> Option<&char> {
        self.inner.peek()
    }
}

impl<I: Iterator<Item = char>> Iterator for TrackingIterator<I> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.inner.next()?;
        if c == '\n' {
            self.line += 1;
            self.char = 0;
        } else {
            self.char += 1;
        }
        Some(c)
    }
}

pub struct JackTokenizer<I: Iterator<Item = char>> {
    chars_iterator: TrackingIterator<I>,
    next: Option<Result<Token, String>>,
}

impl<I: Iterator<Item = char>> JackTokenizer<I> {
    pub fn new(chars_iterator: I) -> Self {
        JackTokenizer {
            chars_iterator: TrackingIterator::new(chars_iterator),
            next: None,
        }
    }
    pub fn line(&self) -> i64 {
        self.chars_iterator.line
    }
    pub fn char(&self) -> i64 {
        self.chars_iterator.char
    }
    pub fn peek(&mut self) -> Option<&Result<Token, String>> {
        if self.next.is_none() {
            self.next = self.advance();
        }
        self.next.as_ref()
    }

    fn advance(&mut self) -> Option<Result<Token, String>> {
        while let Some(c) = self.chars_iterator.next() {
            return Some(Ok(match c {
                ' ' | '\r' | '\t' | '\n' => continue,
                '/' => {
                    if let Some('/') = self.chars_iterator.peek() {
                        while let Some(c) = self.chars_iterator.next()
                            && c != '\n'
                        {}
                        continue;
                    } else if let Some('*') = self.chars_iterator.peek() {
                        self.chars_iterator.next();
                        while self.chars_iterator.peek().is_some()
                            && (!matches!(self.chars_iterator.next(), Some('*'))
                                || !matches!(self.chars_iterator.peek(), Some('/')))
                        {
                        }
                        self.chars_iterator.next();
                        continue;
                    } else {
                        Token::Symbol(Symbol::Slash)
                    }
                }
                '*' => Symbol(Symbol::Asterisk),
                '+' => Symbol(Symbol::Plus),
                '-' => Symbol(Symbol::Minus),
                '>' => Symbol(Symbol::GreaterThan),
                '<' => Symbol(Symbol::LessThan),
                '=' => Symbol(Symbol::Equals),
                '~' => Symbol(Symbol::Tilde),
                '{' => Symbol(Symbol::LeftBrace),
                '}' => Symbol(Symbol::RightBrace),
                '[' => Symbol(Symbol::LeftBracket),
                ']' => Symbol(Symbol::RightBracket),
                '(' => Symbol(Symbol::LeftParen),
                ')' => Symbol(Symbol::RightParen),
                ',' => Symbol(Symbol::Comma),
                ';' => Symbol(Symbol::Semicolon),
                '.' => Symbol(Symbol::Dot),
                '&' => Symbol(Symbol::Ampersand),
                '|' => Symbol(Symbol::Pipe),
                '"' => {
                    let mut acc = String::new();
                    for c in self.chars_iterator.by_ref() {
                        if c == '"' {
                            break;
                        }
                        if c == '\n' {
                            return Some(Err("Unterminated string literal".to_string()));
                        }
                        acc.push(c);
                    }
                    StringConst(acc)
                }
                '0'..='9' => {
                    let mut num = String::from(c);
                    while let Some(&c) = self.chars_iterator.peek() {
                        if c.is_ascii_digit() {
                            num.push(c);
                            self.chars_iterator.next();
                        } else {
                            break;
                        }
                    }
                    if let Ok(n) = num.parse::<i16>() {
                        IntegerConst(n)
                    } else {
                        return Some(Err(format!("Invalid integer: {}", num)));
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut acc = String::from(c);
                    while let Some(&c) = self.chars_iterator.peek() {
                        if matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9') {
                            acc.push(c);
                            self.chars_iterator.next();
                        } else {
                            break;
                        }
                    }
                    match acc.as_str() {
                        "class" => Keyword(Keyword::Class),
                        "constructor" => Keyword(Keyword::Constructor),
                        "function" => Keyword(Keyword::Function),
                        "method" => Keyword(Keyword::Method),
                        "field" => Keyword(Keyword::Field),
                        "static" => Keyword(Keyword::Static),
                        "var" => Keyword(Keyword::Var),
                        "int" => Keyword(Keyword::Int),
                        "char" => Keyword(Keyword::Char),
                        "boolean" => Keyword(Keyword::Boolean),
                        "void" => Keyword(Keyword::Void),
                        "true" => Keyword(Keyword::True),
                        "false" => Keyword(Keyword::False),
                        "null" => Keyword(Keyword::Null),
                        "this" => Keyword(Keyword::This),
                        "let" => Keyword(Keyword::Let),
                        "do" => Keyword(Keyword::Do),
                        "if" => Keyword(Keyword::If),
                        "else" => Keyword(Keyword::Else),
                        "while" => Keyword(Keyword::While),
                        "return" => Keyword(Keyword::Return),
                        _ => Identifier(acc),
                    }
                }
                _ => return Some(Err(format!("Invalid character: {}", c))),
            }));
        }
        None
    }
}

impl<I: Iterator<Item = char>> Iterator for JackTokenizer<I> {
    type Item = Result<Token, String>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.take() {
            Some(next)
        } else {
            self.advance()
        }
    }
}
