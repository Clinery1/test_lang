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

    /// test if we are at EOF
    fn at_eof(&self)->bool {
        self.lookahead[0].is_none()
    }

    /// parse a file's worth of statements
    pub fn parse_file(&mut self)->Result<Vec<Stmt>, Error> {
        let mut items = Vec::new();

        self.skip_newline();

        while !self.at_eof() {
            items.push(self.parse_stmt()?);

            self.skip_newline();
        }

        return Ok(items);
    }

    /// parse a statement
    pub fn parse_stmt(&mut self)->Result<Stmt, Error> {
        let ret = match self.peek()? {
            Token::Keyword(Keyword::Function)=>self.parse_function_stmt(),
            Token::Keyword(Keyword::Var|Keyword::Let|Keyword::Mut)=>self.parse_create_var_stmt(),
            Token::Keyword(Keyword::Set)=>self.parse_set_var_stmt(),
            Token::Keyword(Keyword::Const)=>self.parse_create_const_stmt(),
            Token::Keyword(Keyword::Class)=>self.parse_class_stmt(),
            Token::Keyword(Keyword::If)=>self.parse_if_stmt(),
            Token::Keyword(Keyword::While)=>self.parse_while_stmt(),
            Token::Keyword(Keyword::Delete)=>{
                self.next()?;

                let name = self.ident()?;

                Ok(Stmt::DeleteVar(name))
            },
            _=>self.parse_expr().map(Stmt::Expression),
        }?;

        match self.peek() {
            Ok(Token::Newline|Token::Semicolon)=>{
                self.next()?;
            },
            Ok(_)=>return Err(Error::new(self.peek_span(), ErrorType::LineEnding)),
            _=>{},
        }

        return Ok(ret);
    }

    /// parse a while loop statement
    fn parse_while_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::While))?;

        let condition = self.parse_expr()?;

        let body = self.parse_block()?;

        return Ok(Stmt::WhileLoop {
            condition,
            body,
        });
    }

    /// parse an if-if else-else statement
    fn parse_if_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::If))?;

        let mut conditions = vec![
            (self.parse_expr()?, self.parse_block()?),
        ];

        let mut default = None;

        loop {
            match self.peek() {
                Ok(Token::Keyword(Keyword::Else))=>{
                    self.next()?;
                    match self.peek() {
                        Ok(Token::Keyword(Keyword::If))=>{
                            self.next()?;
                            conditions.push((
                                self.parse_expr()?,
                                self.parse_block()?,
                            ));
                        },
                        Ok(Token::CurlyStart)=>{
                            default = Some(self.parse_block()?);
                            break;
                        },
                        Ok(_)=>return Err(Error::new(
                            self.peek1_span(),
                            ErrorType::ExpectedToken(Token::Keyword(Keyword::If)),
                        )),
                        Err(e)=>return Err(e),
                    }
                },
                Ok(_)=>break,
                Err(e)=>return Err(e),
            }
        }

        return Ok(Stmt::If {
            conditions,
            default,
        });
    }

    /// parse a class definition statement
    fn parse_class_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::Class))?;

        let name = self.ident()?;

        self.try_next(Token::CurlyStart)?;
        let start = self.span();

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        self.skip_newline();

        loop {
            match self.peek() {
                Ok(Token::CurlyEnd)=>{
                    self.next()?;
                    break;
                },
                Ok(Token::Keyword(Keyword::Var|Keyword::Let|Keyword::Mut))=>{
                    let var_type = self.parse_var_type()?;

                    let name = self.ident()?;

                    fields.push((var_type, name));
                },
                Ok(Token::Ident(_))=>{
                    let method = self.parse_abrv_function_stmt()?;

                    methods.push(method);
                },
                Ok(_)=>return Err(Error::token(self.peek_span())),
                Err(Error{err_type:ErrorType::UnexpectedEOF,..})=>{
                    let span = self.peek_span();
                    return Err(Error::new(start.start..span.end, ErrorType::UnclosedCurly));
                },
                Err(e)=>return Err(e),
            }

            match self.peek() {
                Ok(Token::CurlyEnd)=>{
                    self.next()?;
                    break;
                },
                Ok(Token::Newline|Token::Semicolon)=>{
                    self.next()?;
                },
                Ok(_)=>return Err(Error::new(self.peek_span(), ErrorType::LineEnding)),
                Err(Error{err_type:ErrorType::UnexpectedEOF,..})=>{
                    let span = self.peek_span();
                    return Err(Error::new(start.start..span.end, ErrorType::UnclosedCurly));
                },
                Err(e)=>return Err(e),
            }

            self.skip_newline();
        }

        return Ok(Stmt::Class {
            name,
            fields,
            methods,
        });
    }

    /// parse a var set statement
    fn parse_set_var_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::Set))?;

        let left = self.parse_expr()?;

        self.try_next(Token::Assign)?;

        let data = self.parse_expr()?;

        return Ok(Stmt::SetVar {left, data});
    }

    /// parse a const var statement
    fn parse_create_const_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::Const))?;

        let name = self.ident()?;

        self.try_next(Token::Assign)?;

        let data = self.parse_expr()?;

        return Ok(Stmt::CreateConst {name, data});
    }

    /// Parses the var type. Used multiple places
    fn parse_var_type(&mut self)->Result<VarType, Error> {
        let mut var_type = match self.peek() {
            Ok(Token::Keyword(Keyword::Mut))=>{
                self.next()?;
                VarType::MUTATE
            },
            _=>VarType::empty(),
        };
        var_type |= match self.next() {
            Ok(Token::Keyword(Keyword::Var))=>VarType::REASSIGN,
            Ok(Token::Keyword(Keyword::Let))=>VarType::empty(),
            _=>return Err(Error::token(self.peek_span())),
        };

        return Ok(var_type);
    }

    /// parses a var creation statement
    fn parse_create_var_stmt(&mut self)->Result<Stmt, Error> {
        let var_type = self.parse_var_type()?;

        let name = self.ident()?;

        let data = match self.peek() {
            Ok(Token::Assign)=>{
                self.next()?;
                Some(self.parse_expr()?)
            },
            _=>None,
        };

        return Ok(Stmt::CreateVar {
            var_type,
            name,
            data,
        });
    }

    /// parses a full function using the abbreviated helper function
    fn parse_function_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::Function))?;

        return self.parse_abrv_function_stmt().map(Stmt::Function);
    }

    /// a function statement used in class definitions and the inner part of a normal function
    /// definition.
    fn parse_abrv_function_stmt(&mut self)->Result<Function, Error> {
        let name = self.ident()?;

        let params = self.parse_paren_list(Self::ident)?;

        let body = self.parse_block()?;

        return Ok(Function {
            name,
            params,
            body,
        });
    }

    /// parse a block of statements in curly brackets
    fn parse_block(&mut self)->Result<Vec<Stmt>, Error> {
        self.try_next(Token::CurlyStart)?;
        let start = self.span();

        let mut items = Vec::new();

        loop {
            self.skip_newline();

            match self.peek() {
                // break the loop
                Ok(Token::CurlyEnd)=>{
                    self.next()?;
                    break;
                },
                // convert EOF to unclosed curly bracket error
                Err(Error{err_type:ErrorType::UnexpectedEOF,..})=>{
                    let span = self.peek_span();

                    return Err(Error::new(start.start..span.end, ErrorType::UnclosedCurly));
                },
                // return all other errors
                Err(e)=>return Err(e),
                // parse the next stmt
                _=>{
                    let item = match self.parse_stmt() {
                        Ok(s)=>s,
                        Err(Error{err_type:ErrorType::UnexpectedEOF,..})=>{
                            let span = self.peek_span();
                            return Err(Error::new(start.start..span.end, ErrorType::UnclosedCurly));
                        },
                        Err(e)=>return Err(e),
                    };
                    items.push(item);
                },
            }
        }

        return Ok(items);
    }

    /// Parse a single expression
    pub fn parse_expr(&mut self)->Result<Expr, Error> {
        let left = match self.peek()? {
            Token::Keyword(Keyword::Copy)=>{
                self.next()?;
                let name = self.ident()?;
                Expr::Copy(name)
            },
            Token::Keyword(Keyword::Ref)=>{
                self.next()?;
                let var_type = self.parse_var_type()?;

                let name = self.ident()?;

                Expr::Ref(var_type, name)
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
                    let mut items = self.parse_paren_list(Self::parse_expr)?;
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

    /// a generic function to parse a comma separated list of `T` which is parsed by the function
    /// `F`
    fn parse_paren_list<T, F:FnMut(&mut Self)->Result<T, Error>>(&mut self, mut f: F)->Result<Vec<T>, Error> {
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
                _=>{
                    let item = match f(self) {
                        Ok(e)=>e,
                        Err(Error{err_type:ErrorType::UnexpectedEOF,..})=>{
                            let span = self.peek_span();
                            return Err(Error::new(start.start..span.end, ErrorType::UnclosedParen));
                        },
                        Err(e)=>return Err(e),
                    };
                    items.push(item);
                },
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

    /// parse a binary operation, if possible
    fn parse_bin_op(&mut self, peek_second: bool)->Option<BinaryOp> {
        let peek = if peek_second {
            self.peek1()
        } else {
            self.peek()
        };
        let op = match peek {
            Ok(Token::Add)=>BinaryOp::Add,
            Ok(Token::Sub)=>BinaryOp::Sub,
            Ok(Token::Mul)=>BinaryOp::Mul,
            Ok(Token::Div)=>BinaryOp::Div,
            Ok(Token::Mod)=>BinaryOp::Mod,
            Ok(Token::Equal)=>BinaryOp::Equal,
            Ok(Token::NotEqual)=>BinaryOp::NotEqual,
            Ok(Token::Greater)=>BinaryOp::Greater,
            Ok(Token::Less)=>BinaryOp::Less,
            Ok(Token::GreaterEqual)=>BinaryOp::GreaterEqual,
            Ok(Token::LessEqual)=>BinaryOp::LessEqual,
            Ok(Token::Keyword(Keyword::And))=>BinaryOp::LogicAnd,
            Ok(Token::Keyword(Keyword::Or))=>BinaryOp::LogicOr,
            _=>return None,
        };

        self.next().unwrap();

        return Some(op);
    }

    /// parse a binary operation, if we can.
    fn parse_bin_op_expr(&mut self)->Result<Expr, Error> {
        // parse the left side
        let left = self.parse_paren_expr()?;

        // peek to see if we have an newline or an operator. Without this peek, we will sometimes
        // remove newlines used by `parse_stmt`
        let op = match self.peek()? {
            Token::Newline=>match self.parse_bin_op(true) {
                Some(op)=>op,
                // if we have no operator, then return the left side expression
                _=>return Ok(left),
            },
            _=>match self.parse_bin_op(false) {
                Some(op)=>op,
                _=>return Ok(left),
            },
        };

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
                let expr = match self.parse_expr() {
                    Ok(e)=>e,
                    Err(Error{err_type:ErrorType::UnexpectedEOF,..})=>{
                        let span = self.peek_span();
                        return Err(Error::new(start.start..span.end, ErrorType::UnclosedParen));
                    },
                    Err(e)=>return Err(e),
                };

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
            Token::Keyword(Keyword::This)=>Ok(Expr::This),
            _=>Err(Error::token(self.span())),
        }
    }
}
