use logos::{
    Logos,
    Lexer,
};
use string_interner::{
    DefaultSymbol as Symbol,
    StringInterner,
};


#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(extras = StringInterner)]
#[logos(skip "[ \t\r]")]
pub enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", intern_string)]
    Ident(Symbol),
    #[regex(r"[0-9][0-9_]*", parse_integer)]
    Integer(i64),
    #[regex(r"[0-9_]+\.[0-9_]+", parse_float)]
    Float(f64),
    #[token("function", |_|Keyword::Function)]
    #[token("var", |_|Keyword::Var)]
    #[token("let", |_|Keyword::Let)]
    #[token("const", |_|Keyword::Const)]
    #[token("class", |_|Keyword::Class)]
    #[token("super", |_|Keyword::Super)]
    #[token("mut", |_|Keyword::Mut)]
    #[token("set", |_|Keyword::Set)]
    #[token("copy", |_|Keyword::Copy)]
    #[token("true", |_|Keyword::True)]
    #[token("false", |_|Keyword::False)]
    #[token("delete", |_|Keyword::Delete)]
    #[token("if", |_|Keyword::If)]
    #[token("else", |_|Keyword::Else)]
    #[token("while", |_|Keyword::While)]
    #[token("and", |_|Keyword::And)]
    #[token("or", |_|Keyword::Or)]
    #[token("ref", |_|Keyword::Ref)]
    #[token("return", |_|Keyword::Return)]
    #[token("break", |_|Keyword::Break)]
    #[token("continue", |_|Keyword::Continue)]
    #[token("print", |_|Keyword::Print)]
    #[token("pub", |_|Keyword::Public)]
    Keyword(Keyword),
    #[token("(")]
    ParenStart,
    #[token(")")]
    ParenEnd,
    #[token("{")]
    CurlyStart,
    #[token("}")]
    CurlyEnd,
    #[token("[")]
    SquareStart,
    #[token("]")]
    SquareEnd,
    #[token("=")]
    Assign,
    #[token(":")]
    Colon,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token(">")]
    Greater,
    #[token("<")]
    Less,
    #[token(">=")]
    GreaterEqual,
    #[token("<=")]
    LessEqual,
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Mod,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(";")]
    Semicolon,
    #[token("!")]
    Not,
    // we have to include whitespace here or we will get multiple newline tokens emitted if there
    // is a line containing only whitespace
    #[regex("\n[ \t\r\n]*")]
    Newline,
    #[token("\"", parse_string)]
    String(String),
    #[token("::")]
    ColonColon,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Function,
    Var,
    Let,
    Const,
    Class,
    Super,
    Mut,
    Set,
    Copy,
    True,
    False,
    Delete,
    If,
    Else,
    While,
    And,
    Or,
    Ref,
    Return,
    Break,
    Continue,
    Print,
    Public,
}


fn parse_string<'a>(lex: &mut Lexer<'a, Token>)->Option<String> {
    let mut escape = false;
    let mut finished = false;
    let mut out = String::new();

    // TODO: more escape sequences
    for c in lex.remainder().chars() {
        if !escape {    // if we are not in an escape
            match c {
                '"'=>{  // break the loop on double quote
                    finished = true;
                    break;
                },
                // set escape if the current character is a backslash
                '\\'=>escape = true,
                _=>out.push(c),
            }
        } else {
            // reset escape if it is on
            escape = false;
            match c {
                '"'=>out.push('"'),
                'n'=>out.push('\n'),
                'r'=>out.push('\r'),
                't'=>out.push('\t'),
                '\\'=>out.push('\\'),
                _=>{},  // ignore invalid escapes
            }
        }

        // bump the lexer by how many bytes c takes up
        lex.bump(c.len_utf8());
    }

    if !finished {  // if we reached EOF, then return None
        return None;
    }

    // bump the lexer past the trailing quote
    lex.bump(1);

    // intern the string, because using `Symbol` is easier than `&'a str` and faster than `String`
    return Some(out);
}

// intern the string slice of the current token and return the symbol
fn intern_string<'a>(lex: &mut Lexer<'a, Token>)->Symbol {
    lex.extras.get_or_intern(lex.slice())
}

// parse an f64 from the current token's string slice
fn parse_float<'a>(lex: &mut Lexer<'a, Token>)->f64 {
    lex
        .slice()
        .parse::<f64>()
        .unwrap()
}

// parse a i64 from the current token's string slice
fn parse_integer<'a>(lex: &mut Lexer<'a, Token>)->i64 {
    lex
        .slice()
        .parse::<i64>()
        .unwrap()
}
