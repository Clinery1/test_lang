use logos::{
    Logos,
    Lexer,
};
use string_interner::{
    DefaultSymbol as Symbol,
    StringInterner,
};


#[derive(Logos, Debug, PartialEq)]
#[logos(extras = StringInterner)]
#[logos(skip "[ \t\r]")]
pub enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", intern_string)]
    Ident(Symbol),
    #[regex(r"[0-9][0-9_]*", parse_integer)]
    Integer(u64),
    #[regex(r"[0-9_]+\.[0-9_]+", parse_float)]
    Float(f64),
    #[token("function", |_|Keyword::Function)]
    #[token("var", |_|Keyword::Var)]
    #[token("val", |_|Keyword::Val)]
    #[token("const", |_|Keyword::Const)]
    #[token("class", |_|Keyword::Class)]
    #[token("this", |_|Keyword::This)]
    #[token("super", |_|Keyword::Super)]
    #[token("mut", |_|Keyword::Mutable)]
    #[token("set", |_|Keyword::Set)]
    #[token("copy", |_|Keyword::Copy)]
    #[token("true", |_|Keyword::True)]
    #[token("false", |_|Keyword::False)]
    #[token("delete", |_|Keyword::Delete)]
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
    #[regex("\n+")]
    Newline,
    #[token("\"", parse_string)]
    String(Symbol),
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Function,
    Var,
    Val,
    Const,
    Class,
    This,
    Super,
    Mutable,
    Set,
    Copy,
    True,
    False,
    Delete,
}


fn parse_string<'a>(lex: &mut Lexer<'a, Token>)->Option<Symbol> {
    let mut escape = false;
    let mut finished = false;

    for c in lex.remainder().chars() {
        if !escape {
            if c == '"' {
                finished = true;
                break;
            }
            escape = c == '\\';
        } else {
            escape = false;
        }

        lex.bump(c.len_utf8());
    }

    if !finished {
        return None;
    }

    let string = &lex.slice()[1..];
    lex.bump(1);

    return Some(lex.extras.get_or_intern(string));
}

fn intern_string<'a>(lex: &mut Lexer<'a, Token>)->Symbol {
    lex.extras.get_or_intern(lex.slice())
}

fn parse_float<'a>(lex: &mut Lexer<'a, Token>)->f64 {
    lex
        .slice()
        .parse::<f64>()
        .unwrap()
}

fn parse_integer<'a>(lex: &mut Lexer<'a, Token>)->u64 {
    lex
        .slice()
        .parse::<u64>()
        .unwrap()
}
