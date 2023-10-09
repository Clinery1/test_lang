use logos::{
    Logos,
    SpannedIter,
    Span,
};
use string_interner::DefaultSymbol as Symbol;
use crate::{
    error::*,
    lexer::*,
    ast::*,
};


pub struct Parser<'a> {
    pub lexer: SpannedIter<'a, Token>,
    lookahead: [Option<Result<Token, ()>>;2],
    spans: [Span;3],
}
impl<'a> Parser<'a> {
    pub fn new(source: &'a str)->Self {
        let lexer = Token::lexer(source).spanned();
        let mut ret = Parser {
            lexer,
            lookahead: [None, None],
            spans: [0..0, 0..0, 0..0],
        };
        ret.next().ok();
        ret.next().ok();
        return ret;
    }

    /// a helper function to peek at the next token
    fn peek(&self)->Result<&Token, Error> {
        match &self.lookahead[0] {
            Some(Ok(t))=>Ok(&t),
            Some(Err(_))=>Err(Error::token(self.span())),
            None=>Err(Error::eof(self.spans[0].clone())),
        }
    }

    /// a helper function to peek at the token after the next token
    fn peek1(&self)->Result<&Token, Error> {
        match &self.lookahead[1] {
            Some(Ok(t))=>Ok(&t),
            Some(Err(_))=>Err(Error::token(self.span())),
            None=>Err(Error::eof(self.spans[0].clone())),
        }
    }

    /// helper function to get the span of the **current** token
    fn span(&self)->Span {
        self.spans[0].clone()
    }

    /// helper function to get the span of the next token
    fn peek_span(&self)->Span {
        self.spans[1].clone()
    }

    /// helper function to get the span of the token after the next token
    fn peek1_span(&self)->Span {
        self.spans[2].clone()
    }

    /// shuffles lookahead and returns the next token
    fn next(&mut self)->Result<Token, Error> {
        // shuffle the two lookahead items
        let ret = self.lookahead[0].take();
        let tmp = self.lookahead[1].take();
        self.lookahead[0] = tmp;
        self.lookahead[1] = self.lexer
            .next()
            .map(|(t,_)|t);

        // shuffle the current and two lookahead spans
        let span0 = self.spans[1].clone();
        let span1 = self.spans[2].clone();
        self.spans[0] = span0;
        self.spans[1] = span1;
        if self.lookahead[1].is_some() {
            self.spans[2] = self.lexer.span();
        } else {
            let end = self.lexer.source().len();
            self.spans[2] = end..end;
        }

        match ret {
            Some(Ok(t))=>Ok(t),
            Some(Err(_))=>Err(Error::token(self.span())),
            None=>Err(Error::eof(self.spans[0].clone())),
        }
    }

    /// Attempts to match the next token. Returns an error if it does not, and consumes if it does.
    fn try_next(&mut self, tok: Token)->Result<(), Error> {
        if self.peek()? == &tok {
            self.next()?;
            return Ok(());
        }
        return Err(Error::new(self.peek_span(), ErrorType::ExpectedToken(tok)));
    }

    /// Match an `Ident` token and return its symbol
    fn ident(&mut self)->Result<Symbol, Error> {
        let tok = self.next()?;
        match tok {
            Token::Ident(i)=>Ok(i),
            _=>Err(Error::ident(self.span())),
        }
    }

    /// skip newline tokens (should be only one in a row)
    fn skip_newline(&mut self) {
        while let Ok(Token::Newline) = self.peek() {
            self.next().ok();
        }
    }

    /// Parse a single expression
    pub fn parse_expr(&mut self)->Result<Expr, Error> {
        let left = match self.peek()? {
            Token::Keyword(Keyword::Copy)=>{
                self.next()?;
                let name = self.ident()?;
                Expr::Copy(name)
            },
            Token::Not|Token::Sub=>self.parse_unary_op_expr()?,
            _=>self.parse_bin_op_expr()?,
        };

        return self.parse_tail_expr(left);
    }

    /// parse field accesses and function calls; field accesses can happen over multiple lines
    fn parse_tail_expr(&mut self, mut left: Expr)->Result<Expr, Error> {
        loop {
            match self.peek() {
                // Field
                Ok(Token::Dot)=>{
                    self.next()?;
                    let name = self.ident()?;

                    left = Expr::Field(Box::new(left), name);
                },
                // Function call
                Ok(Token::ParenStart)=>{
                    let mut items = self.parse_paren_list_expr()?;
                    items.insert(0, left);

                    left = Expr::Call(items);
                },
                Ok(Token::Newline)=>{
                    match self.peek1() {
                        Ok(Token::Dot)=>{
                            // consume token if we have another dot after the newline. Methods on a
                            // second line are acceptable.
                            self.next()?;
                        },
                        _=>break,
                    }
                },
                _=>break,
            }
        }

        return Ok(left);
    }

    /// parse a parenthesized list of expressions separated by commas with an optional comma before
    /// the closing parenthesis
    fn parse_paren_list_expr(&mut self)->Result<Vec<Expr>, Error> {
        // match the starting parenthesis and store the span of it
        self.try_next(Token::ParenStart)?;
        let start = self.span();

        // parse the inner expressions
        let mut items = Vec::new();
        loop {
            self.skip_newline();

            match self.peek() {
                // if we have parenthesis end, the n consume and end the loop
                Ok(Token::ParenEnd)=>{
                    self.next()?;
                    break;
                },
                // if we have an EOF error, convert it to an "unclosed paren" error spanning the
                // entire parsed area
                Err(Error{err_type:ErrorType::UnexpectedEOF,..})=>{
                    let span = self.peek_span();
                    return Err(Error::new(start.start..span.end, ErrorType::UnclosedParen));
                },
                // otherwise parse the next expression
                _=>items.push(self.parse_expr()?),
            }

            self.skip_newline();

            match self.next() {
                // end the loop
                Ok(Token::ParenEnd)=>break,
                // continue: there may be more expressions
                Ok(Token::Comma)=>{},
                // any unexpected token is an `Expected parenthesis` error
                Ok(_)=>return Err(Error::new(self.span(), ErrorType::ExpectedToken(Token::ParenEnd))),
                // EOF errors are converted to unclosed paren errors
                Err(Error{err_type:ErrorType::UnexpectedEOF,..})=>{
                    let span = self.peek_span();
                    return Err(Error::new(start.start..span.end, ErrorType::UnclosedParen));
                },
                // return other errors
                Err(e)=>return Err(e),
            }
        }

        return Ok(items);
    }

    /// parse a binary operation, if we can.
    fn parse_bin_op_expr(&mut self)->Result<Expr, Error> {
        // parse the left side
        let left = self.parse_paren_expr()?;

        self.skip_newline();

        // peek to see if we have an operation
        let op = match self.peek()? {
            Token::Add=>BinaryOp::Add,
            Token::Sub=>BinaryOp::Sub,
            Token::Mul=>BinaryOp::Mul,
            Token::Div=>BinaryOp::Div,
            Token::Mod=>BinaryOp::Mod,
            Token::Equal=>BinaryOp::Equal,
            Token::NotEqual=>BinaryOp::NotEqual,
            Token::Greater=>BinaryOp::Greater,
            Token::Less=>BinaryOp::Less,
            Token::GreaterEqual=>BinaryOp::GreaterEqual,
            Token::LessEqual=>BinaryOp::LessEqual,
            // if we have no operator, then return the left side expression
            _=>return Ok(left),
        };
        // if we do, then consume the next token
        self.next()?;

        self.skip_newline();

        // parse the right expression
        let right = self.parse_paren_expr()?;

        return Ok(Expr::BinaryOp(op, Box::new([left, right])));
    }

    /// parse a unary expression
    fn parse_unary_op_expr(&mut self)->Result<Expr, Error> {
        // determine which operation we have
        let op = match self.next()? {
            Token::Sub=>UnaryOp::Negate,
            Token::Not=>UnaryOp::Not,
            _=>return Err(Error::token(self.span())),
        };

        // parse the right side
        let expr = self.parse_paren_expr()?;

        return Ok(Expr::UnaryOp(op, Box::new(expr)));
    }

    /// parse an expression in parenthesis or a literal expression
    fn parse_paren_expr(&mut self)->Result<Expr, Error> {
        match self.peek()? {
            Token::ParenStart=>{
                // consume the parenthesis
                self.next()?;

                // store the start
                let start = self.span();

                // parse the inner
                let expr = self.parse_expr()?;

                // convert errors to unclosed paren errors
                match self.try_next(Token::ParenEnd) {
                    Err(_)=>{
                        let span = self.peek_span();
                        return Err(Error::new(start.start..span.end, ErrorType::UnclosedParen));
                    },
                    _=>{},
                }

                return Ok(expr);
            },
            _=>return self.parse_literal_expr(),
        }
    }

    /// parse a literal expression
    fn parse_literal_expr(&mut self)->Result<Expr, Error> {
        match self.next()? {
            Token::Ident(i)=>Ok(Expr::Named(i)),
            Token::Integer(i)=>Ok(Expr::Integer(i)),
            Token::Float(f)=>Ok(Expr::Float(f)),
            Token::String(s)=>Ok(Expr::String(s)),
            Token::Keyword(Keyword::True)=>Ok(Expr::Bool(true)),
            Token::Keyword(Keyword::False)=>Ok(Expr::Bool(false)),
            _=>Err(Error::token(self.span())),
        }
    }
}
