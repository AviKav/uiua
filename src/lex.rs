use std::{error::Error, fmt, path::Path, sync::Arc};

use enum_iterator::{all, Sequence};

pub fn lex(input: &str, file: &Path) -> LexResult<Vec<Sp<Token>>> {
    let mut lexer = Lexer::new(input, file);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next_token()? {
        tokens.push(token);
    }

    Ok(tokens)
}

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar(char),
    Bang,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar(c) => write!(f, "unexpected char {c:?}"),
            LexError::Bang => {
                write!(f, " unexpected char '!', maybe you meant 'not'?")
            }
        }
    }
}

impl Error for LexError {}

pub type LexResult<T = ()> = Result<T, Sp<LexError>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc {
    pub pos: usize,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: Loc,
    pub end: Loc,
    pub file: Arc<Path>,
    pub input: Arc<str>,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file.to_string_lossy(), self.start)
    }
}

impl Span {
    pub fn merge(self, end: Self) -> Self {
        Self {
            start: self.start.min(end.start),
            end: self.end.max(end.end),
            ..self
        }
    }
    pub(crate) const fn sp<T>(self, value: T) -> Sp<T> {
        Sp { value, span: self }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Sp<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Sp<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Sp<U> {
        Sp {
            value: f(self.value),
            span: self.span,
        }
    }
    pub fn as_ref(&self) -> Sp<&T> {
        Sp {
            value: &self.value,
            span: self.span.clone(),
        }
    }
    pub fn filter_map<U>(self, f: impl FnOnce(T) -> Option<U>) -> Option<Sp<U>> {
        f(self.value).map(|value| Sp {
            value,
            span: self.span,
        })
    }
}

impl<T: Clone> Sp<&T> {
    pub fn cloned(self) -> Sp<T> {
        Sp {
            value: self.value.clone(),
            span: self.span,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Sp<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.span)?;
        self.value.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Sp<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.span, self.value)
    }
}

impl<T: Error> Error for Sp<T> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Comment(String),
    DocComment(String),
    Ident(String),
    Integer(String),
    Real(String),
    Keyword(Keyword),
    Simple(Simple),
}

impl Token {
    pub fn as_ident(&self) -> Option<&str> {
        match self {
            Token::Ident(ident) => Some(ident),
            _ => None,
        }
    }
    pub fn as_integer(&self) -> Option<&str> {
        match self {
            Token::Integer(integer) => Some(integer),
            _ => None,
        }
    }
    pub fn as_real(&self) -> Option<&str> {
        match self {
            Token::Real(real) => Some(real),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Simple {
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    Comma,
    Period,
    Elipses,
    Colon,
    SemiColon,
    Arrow,
    Plus,
    PlusEquals,
    Minus,
    MinusEquals,
    Star,
    StarEquals,
    Slash,
    SlashEquals,
    Equals,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl fmt::Display for Simple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Simple::OpenParen => "(",
                Simple::CloseParen => ")",
                Simple::OpenCurly => "{",
                Simple::CloseCurly => "}",
                Simple::OpenBracket => "[",
                Simple::CloseBracket => "]",
                Simple::Comma => ",",
                Simple::Period => ".",
                Simple::Elipses => "..",
                Simple::Colon => ":",
                Simple::SemiColon => ";",
                Simple::Arrow => "->",
                Simple::Plus => "+",
                Simple::PlusEquals => "+=",
                Simple::Minus => "-",
                Simple::MinusEquals => "-=",
                Simple::Star => "*",
                Simple::StarEquals => "*=",
                Simple::Slash => "/",
                Simple::SlashEquals => "/=",
                Simple::Equals => "=",
                Simple::Equal => "==",
                Simple::NotEqual => "!=",
                Simple::Less => "<",
                Simple::LessEqual => "<=",
                Simple::Greater => ">",
                Simple::GreaterEqual => ">=",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum Keyword {
    Struct,
    Enum,
    Let,
    Fn,
    If,
    Else,
    Return,
    And,
    Or,
    While,
    For,
    In,
    Break,
    Continue,
    True,
    False,
    Not,
}

impl From<Keyword> for Token {
    fn from(k: Keyword) -> Self {
        Self::Keyword(k)
    }
}

impl From<Simple> for Token {
    fn from(s: Simple) -> Self {
        Self::Simple(s)
    }
}

struct Lexer {
    input_chars: Vec<char>,
    loc: Loc,
    file: Arc<Path>,
    input: Arc<str>,
}

impl Lexer {
    fn new(input: &str, file: &Path) -> Self {
        Self {
            input_chars: input.chars().collect(),
            loc: Loc {
                pos: 0,
                line: 1,
                col: 1,
            },
            file: file.into(),
            input: input.into(),
        }
    }
    fn next_char_if(&mut self, f: impl Fn(char) -> bool) -> Option<char> {
        let c = *self.input_chars.get(self.loc.pos)?;
        if !f(c) {
            return None;
        }
        match c {
            '\n' => {
                self.loc.line += 1;
                self.loc.col = 1;
            }
            '\r' => {}
            _ => self.loc.col += 1,
        }
        self.loc.pos += 1;
        Some(c)
    }
    fn next_char_exact(&mut self, c: char) -> bool {
        self.next_char_if(|c2| c2 == c).is_some()
    }
    fn next_char(&mut self) -> Option<char> {
        self.next_char_if(|_| true)
    }
    fn end_span(&self, start: Loc) -> Span {
        Span {
            start,
            end: self.loc,
            file: self.file.clone(),
            input: self.input.clone(),
        }
    }
    fn end(&self, token: impl Into<Token>, start: Loc) -> LexResult<Option<Sp<Token>>> {
        Ok(Some(Sp {
            value: token.into(),
            span: self.end_span(start),
        }))
    }
    fn switch_next<T, U, const N: usize>(
        &mut self,
        a: T,
        others: [(char, U); N],
        start: Loc,
    ) -> LexResult<Option<Sp<Token>>>
    where
        T: Into<Token>,
        U: Into<Token>,
    {
        let token = others
            .into_iter()
            .find(|(c, _)| self.next_char_exact(*c))
            .map(|(_, t)| t.into())
            .unwrap_or(a.into());
        self.end(token, start)
    }
    fn next_token(&mut self) -> LexResult<Option<Sp<Token>>> {
        use {self::Simple::*, Token::*};
        loop {
            let start = self.loc;
            let Some(c) = self.next_char() else {
                break Ok(None);
            };
            match c {
                '(' => return self.end(OpenParen, start),
                ')' => return self.end(CloseParen, start),
                '{' => return self.end(OpenCurly, start),
                '}' => return self.end(CloseCurly, start),
                '[' => return self.end(OpenBracket, start),
                ']' => return self.end(CloseBracket, start),
                '.' => return self.switch_next(Period, [('.', Elipses)], start),
                ':' => return self.end(Colon, start),
                ';' => return self.end(SemiColon, start),
                ',' => return self.end(Comma, start),
                '+' => return self.switch_next(Plus, [('=', PlusEquals)], start),
                '-' => return self.switch_next(Minus, [('=', MinusEquals), ('>', Arrow)], start),
                '*' => return self.switch_next(Star, [('=', StarEquals)], start),
                '/' => {
                    if self.next_char_exact('=') {
                        return self.end(SlashEquals, start);
                    } else if self.next_char_exact('/') {
                        // Comments
                        let token = if self.next_char_exact('/') {
                            DocComment
                        } else {
                            Comment
                        };
                        let mut comment = String::new();
                        while let Some(c) = self.next_char_if(|c| c != '\n') {
                            comment.push(c);
                        }
                        let end = self.loc;
                        self.next_char();
                        return Ok(Some(Sp {
                            value: token(comment),
                            span: Span {
                                start,
                                end,
                                file: self.file.clone(),
                                input: self.input.clone(),
                            },
                        }));
                    } else {
                        return self.end(Slash, start);
                    }
                }
                '=' => return self.switch_next(Equals, [('=', Equal)], start),
                '<' => return self.switch_next(Less, [('=', LessEqual)], start),
                '>' => return self.switch_next(Greater, [('=', GreaterEqual)], start),
                '!' => {
                    return if self.next_char_exact('=') {
                        self.end(NotEqual, start)
                    } else {
                        Err(self.end_span(start).sp(LexError::Bang))
                    }
                }
                // Identifiers and keywords
                c if is_ident_start(c) => {
                    let mut ident = String::new();
                    ident.push(c);
                    while let Some(c) = self.next_char_if(is_ident) {
                        ident.push(c);
                    }
                    let token = if let Some(keyword) =
                        all::<self::Keyword>().find(|k| format!("{k:?}").to_lowercase() == ident)
                    {
                        Keyword(keyword)
                    } else {
                        Ident(ident)
                    };
                    return self.end(token, start);
                }
                // Numbers
                c if c.is_ascii_digit() => {
                    // Whole part
                    let mut number = String::from(c);
                    while let Some(c) = self.next_char_if(|c| c.is_ascii_digit()) {
                        number.push(c);
                    }
                    // Fractional part
                    let before_dot = self.loc;
                    if self.next_char_exact('.') {
                        if self.next_char_exact('.') {
                            self.loc = before_dot;
                        } else {
                            number.push('.');
                            while let Some(c) = self.next_char_if(|c| c.is_ascii_digit()) {
                                number.push(c);
                            }
                        }
                    }
                    // Exponent
                    if let Some(e) = self.next_char_if(|c| c == 'e' || c == 'E') {
                        number.push(e);
                        if let Some(sign) = self.next_char_if(|c| c == '-' || c == '+') {
                            number.push(sign);
                        }
                        while let Some(c) = self.next_char_if(|c| c.is_ascii_digit()) {
                            number.push(c);
                        }
                    }
                    let token = if number.contains(['.', 'e', 'E']) {
                        Real
                    } else {
                        Integer
                    }(number);
                    return self.end(token, start);
                }
                c if c.is_whitespace() => {}
                c => return Err(self.end_span(start).sp(LexError::UnexpectedChar(c))),
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}