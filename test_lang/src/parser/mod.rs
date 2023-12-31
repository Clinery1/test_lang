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
// use expr::*;


pub mod expr;


/// A parser using techniques inspired by left-corner parsers, we have a (hopefully) linear-time
/// parser. It is all hand-written, never backtracks, and uses 2 tokens of lookahead.
///
/// Using [`crate::benchmark_parser`] I was able to get ~170MB/s for all input sizes above 300
/// lines up to 3000 lines on my 1165G7 Framework 13. I think this is plenty sufficient for regular
/// use. If parse times become a problem, then I will revisit the parser and try optimizing it or
/// something. Likely the bottlenecks will be static analysis, interpreting the code, and
/// eventually code generation.
pub struct Parser<'a> {
    pub lexer: SpannedIter<'a, Token>,
    pub non_fatal_errors: Vec<Error>,
    lookahead: [Option<Result<Token, ()>>;2],
    spans: [Span;3],
    func_count: usize,
    class_count: usize,
    constructor_sym: Symbol,
}
impl<'a> Parser<'a> {
    /// Create a new parser from a source string
    pub fn new(source: &'a str)->(Self, Symbol) {
        let mut lexer = Token::lexer(source).spanned();

        let this_sym = lexer.extras.get_or_intern("this");

        let mut ret = Parser {
            constructor_sym: lexer.extras.get_or_intern("constructor"),
            lexer,
            lookahead: [None, None],
            spans: [0..0, 0..0, 0..0],
            func_count: 0,
            class_count: 0,
            non_fatal_errors: Vec::new(),
        };

        // fill the lookahead buffer
        ret.next().ok();
        ret.next().ok();

        return (ret, this_sym);
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
        return Err(Error::new(self.peek_span(), ErrorType::ExpectedToken(format!("{:?}", tok))));
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

    fn push_err(&mut self, err: Error) {
        self.non_fatal_errors.push(err);
    }

    fn get_class_id(&mut self)->usize {
        self.class_count += 1;
        return self.class_count - 1;
    }

    fn get_func_id(&mut self)->usize {
        self.func_count += 1;
        return self.func_count - 1;
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
        let mut need_ending = true;
        let publicity = self.parse_publicity()?;
        let ret;
        if let Some(publicity) = publicity {
            ret = match self.peek()? {
                Token::Keyword(Keyword::Function)=>{
                    need_ending = false;
                    self.parse_function_stmt(publicity)
                },
                Token::Keyword(Keyword::Class)=>{
                    need_ending = false;
                    self.parse_class_stmt(publicity)
                },
                _=>Err(Error::token(self.peek_span())),
            }?;
        } else {
            ret = match self.peek()? {
                Token::Keyword(Keyword::Function)=>{
                    need_ending = false;
                    self.parse_function_stmt(Permissions::empty())
                },
                Token::Keyword(Keyword::Class)=>{
                    need_ending = false;
                    self.parse_class_stmt(Permissions::empty())
                },
                Token::Keyword(Keyword::If)=>{
                    need_ending = false;
                    self.parse_if_stmt()
                },
                Token::Keyword(Keyword::While)=>{
                    need_ending = false;
                    self.parse_while_stmt()
                },
                Token::Keyword(Keyword::Var|Keyword::Let)=>self.parse_create_var_stmt(),
                Token::Keyword(Keyword::Set)=>self.parse_set_var_stmt(),
                Token::Keyword(Keyword::Const)=>self.parse_create_const_stmt(),
                Token::Keyword(Keyword::Break)=>{
                    self.next()?;
                    Ok(Stmt::Break(self.span()))
                },
                Token::Keyword(Keyword::Continue)=>{
                    self.next()?;
                    Ok(Stmt::Continue(self.span()))
                },
                Token::Keyword(Keyword::Return)=>{
                    self.next()?;
                    let start = self.span().start;

                    let expr = match self.peek() {
                        Ok(Token::Newline|Token::Semicolon)=>None,
                        Ok(_)=>Some(self.parse_expr()?),
                        Err(_)=>None,
                    };

                    let end = self.span().end;

                    Ok(Stmt::Return(start..end, expr))
                },
                Token::Keyword(Keyword::Delete)=>{
                    self.next()?;
                    let start = self.span().start;

                    let name = self.ident()?;

                    let end = self.span().end;

                    Ok(Stmt::DeleteVar(start..end, name))
                },
                Token::Keyword(Keyword::Print)=>{
                    self.next()?;
                    let start = self.span().start;

                    let data = self.parse_expr()?;

                    let end = self.span().end;

                    Ok(Stmt::Print(start..end, data))
                },
                _=>{
                    let start = self.peek_span().start;
                    let expr = self.parse_expr()?;
                    let end = self.span().end;
                    Ok(Stmt::Expression(start..end, expr))
                },
            }?;
        }

        if need_ending {
            self.parse_stmt_end()?;
        }

        return Ok(ret);
    }

    /// parse a semicolon, newline, or EOF
    fn parse_stmt_end(&mut self)->Result<(), Error> {
        match self.peek() {
            Ok(Token::Newline|Token::Semicolon)=>{
                self.next()?;
                Ok(())
            },
            Ok(_)=>Err(Error::new(self.peek_span(), ErrorType::LineEnding)),
            _=>Ok(()),
        }
    }

    /// parse a while loop statement
    fn parse_while_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::While))?;
        let start = self.span().start;

        let condition = self.parse_expr()?;

        let body = self.parse_block()?;

        let end = self.span().end;

        return Ok(Stmt::WhileLoop {
            span: start..end,
            condition,
            body,
        });
    }

    /// parse an if-if else-else statement
    fn parse_if_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::If))?;
        let start = self.span().start;

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
                            ErrorType::ExpectedToken("if".to_string()),
                        )),
                        Err(e)=>return Err(e),
                    }
                },
                Ok(_)=>break,
                Err(e)=>return Err(e),
            }
        }

        let end = self.span().end;

        return Ok(Stmt::If {
            span: start..end,
            conditions,
            default,
        });
    }

    /// parse a class definition statement
    fn parse_class_stmt(&mut self, permissions: Permissions)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::Class))?;
        let start = self.span().start;

        let name = self.ident()?;

        self.try_next(Token::CurlyStart)?;
        let curly_start = self.span().start;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut associated = Vec::new();
        let mut constructor: Option<Function> = None;


        loop {
            self.skip_newline();
            let permissions = self.parse_publicity()?.unwrap_or_default();

            match self.peek() {
                Ok(Token::CurlyEnd)=>{
                    self.next()?;
                    break;
                },
                Ok(Token::Keyword(Keyword::Constructor))=>{
                    let new_constructor = self.parse_function_inner(FunctionType::Normal, permissions)?;
                    if let Some(c) = constructor {
                        self.push_err(Error::two_location(
                            c.span(),
                            new_constructor.span(),
                            "Constructor previously defined here",
                            ErrorType::ConstructorRedefined,
                        ));
                    }
                    constructor = Some(new_constructor);
                },
                Ok(Token::Keyword(Keyword::Function))=>{
                    self.next()?;
                    associated.push(self.parse_function_inner(FunctionType::Normal, permissions)?);
                },
                Ok(Token::Keyword(Keyword::Var|Keyword::Let))=>{
                    let var_type = self.parse_var_type()?;
                    let name = self.ident()?;

                    fields.push((var_type, name));
                },
                Ok(Token::Keyword(Keyword::Mut))=>{
                    self.next()?;
                    let method = self.parse_function_inner(FunctionType::MutableMethod, permissions)?;

                    methods.push(method);
                },
                Ok(Token::Ident(_))=>{
                    let method = self.parse_function_inner(FunctionType::Method, permissions)?;

                    methods.push(method);
                },
                Ok(_)=>return Err(Error::token(self.peek_span())),
                Err(e)=>{
                    if e.err_type() == &ErrorType::UnexpectedEOF {
                        let span = self.peek_span();
                        return Err(Error::new(curly_start..span.end, ErrorType::UnclosedCurly));
                    }
                    return Err(e);
                },
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
                Err(e)=>{
                    if e.err_type() == &ErrorType::UnexpectedEOF {
                        let span = self.peek_span();
                        return Err(Error::new(curly_start..span.end, ErrorType::UnclosedCurly));
                    }
                    return Err(e);
                },
            }

            self.skip_newline();
        }

        let end = self.span().end;

        if fields.len() > 0 && constructor.is_none() {
            self.push_err(Error::new(start..end, ErrorType::ConstructorRequired));
        }

        return Ok(Stmt::Class {
            span: start..end,
            id: self.get_class_id(),
            permissions,
            name,
            constructor,
            fields,
            methods,
            associated,
        });
    }

    /// parse a var set statement
    fn parse_set_var_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::Set))?;
        let start = self.span().start;

        let mut left = vec![self.ident()?];

        while let Ok(Token::Dot) = self.peek() {
            self.next()?;

            left.push(self.ident()?);
        }

        self.try_next(Token::Assign)?;

        let data = self.parse_expr()?;

        let end = self.span().end;

        return Ok(Stmt::SetVar {
            span: start..end,
            left,
            data,
        });
    }

    /// parse a const var statement
    fn parse_create_const_stmt(&mut self)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::Const))?;
        let start = self.span().start;

        let name = self.ident()?;

        self.try_next(Token::Assign)?;

        let data = self.parse_expr()?;

        let end = self.span().end;

        return Ok(Stmt::CreateConst {
            span: start..end,
            name,
            data,
        });
    }

    /// parse the `pub{,(..)}` attribute
    fn parse_publicity(&mut self)->Result<Option<Permissions>, Error> {
        let mut perms = Permissions::empty();

        match self.peek()? {
            Token::Keyword(Keyword::Public)=>{
                self.next()?;

                match self.peek()? {
                    Token::ParenStart=>{
                        self.next()?;

                        match self.next()? {
                            Token::Keyword(Keyword::Var)=>{
                                perms |= Permissions::PUBLIC_REASSIGN;
                                match self.peek()? {
                                    Token::Keyword(Keyword::Mut)=>{
                                        self.next()?;
                                        perms |= Permissions::PUBLIC_MUTABLE;
                                    },
                                    _=>{}
                                }
                            },
                            Token::Keyword(Keyword::Mut)=>perms |= Permissions::PUBLIC_MUTABLE,
                            _=>return Err(Error::token(self.span())),
                        }

                        self.try_next(Token::ParenEnd)?;
                    },
                    _=>perms |= Permissions::PUBLIC,
                }
            },
            _=>{},
        }

        if perms.contains(Permissions::PUBLIC) {
            return Ok(Some(perms));
        } else {
            return Ok(None);
        }
    }

    /// Parses the var type. Used multiple places
    fn parse_var_type(&mut self)->Result<Permissions, Error> {
        let mut var_type = Permissions::empty();

        match self.next() {
            Ok(Token::Keyword(Keyword::Var))=>var_type |= Permissions::REASSIGN,
            Ok(Token::Keyword(Keyword::Let))=>var_type |= Permissions::IS_VARIABLE,
            _=>return Err(Error::token(self.span())),
        }

        match self.peek() {
            Ok(Token::Keyword(Keyword::Mut))=>{
                self.next()?;
                var_type |= Permissions::MUTATE;
            },
            _=>{},
        };

        return Ok(var_type);
    }

    /// parses a var creation statement
    fn parse_create_var_stmt(&mut self)->Result<Stmt, Error> {
        let start = self.peek_span().start;
        let var_type = self.parse_var_type()?;

        let name = self.ident()?;

        let data = match self.peek() {
            Ok(Token::Assign)=>{
                self.next()?;
                Some(self.parse_expr()?)
            },
            _=>None,
        };

        let end = self.span().end;

        return Ok(Stmt::CreateVar {
            span: start..end,
            var_type,
            name,
            data,
        });
    }

    /// parses a full function using the abbreviated helper function
    fn parse_function_stmt(&mut self, permissions: Permissions)->Result<Stmt, Error> {
        self.try_next(Token::Keyword(Keyword::Function))?;
        let start = self.span().start;

        let func = self.parse_function_inner(FunctionType::Normal, permissions)?;

        let end = self.span().end;

        return Ok(Stmt::Function(start..end, func));
    }

    /// Parse a function signature
    // fn parse_function_signature(&mut self, func_type: FunctionType, permissions: Permissions)->Result<FunctionSignature, Error> {
    //     let name = self.ident()?;
    //     let start = self.span().start;

    //     let params = self.parse_paren_list(Self::parse_function_param)?;
    //     let end = self.span().end;

    //     return Ok(FunctionSignature {
    //         permissions,
    //         func_type,
    //         span: start..end,
    //         name,
    //         params,
    //     });
    // }

    /// a function statement used in class definitions and the inner part of a normal function
    /// definition.
    fn parse_function_inner(&mut self, func_type: FunctionType, permissions: Permissions)->Result<Function, Error> {
        let name = match self.next()? {
            Token::Ident(i)=>i,
            Token::Keyword(Keyword::Constructor)=>self.constructor_sym,
            _=>return Err(Error::ident(self.span())),
        };
        let start = self.span().start;

        let params = self.parse_paren_list(Self::parse_function_param)?;

        if params.len() > u8::MAX as usize {
            self.push_err(Error::new(self.span(), ErrorType::TooManyParams));
        }

        let body = self.parse_block()?;

        let end = self.span().end;

        return Ok(Function {
            permissions,
            func_type,
            id: self.get_func_id(),
            span: start..end,
            name,
            params,
            body,
        });
    }

    /// Parse a partial var type. Includes `var`, `mut`, `var mut`, and nothing
    fn parse_partial_var_type(&mut self)->Result<Permissions, Error> {
        match self.peek()? {
            Token::Keyword(Keyword::Mut)=>{
                self.next()?;

                Ok(Permissions::MUTATE)
            },
            Token::Keyword(Keyword::Var)=>{
                self.next()?;
                match self.peek()? {
                    Token::Keyword(Keyword::Mut)=>{
                        self.next()?;
                        Ok(Permissions::REASSIGN | Permissions::MUTATE)
                    },
                    _=>Ok(Permissions::REASSIGN),
                }
            },
            _=>Ok(Permissions::empty()),
        }
    }

    /// Parse a single function parameter
    fn parse_function_param(&mut self)->Result<(Span, Permissions, Symbol), Error> {
        let start = self.peek_span().start;
        let var_type = self.parse_partial_var_type()?;

        let name = self.ident()?;
        let end = self.span().end;

        return Ok((start..end, var_type, name));
    }

    /// parse a block of statements in curly brackets
    fn parse_block(&mut self)->Result<Block, Error> {
        self.try_next(Token::CurlyStart)?;
        let start = self.span().start;

        let mut items = Vec::new();

        loop {
            self.skip_newline();

            match self.peek() {
                // break the loop
                Ok(Token::CurlyEnd)=>{
                    self.next()?;
                    break;
                },
                Err(e)=>{
                    // convert EOF to unclosed curly bracket error
                    if e.err_type() == &ErrorType::UnexpectedEOF {
                        let span = self.peek_span();
                        return Err(Error::new(start..span.end, ErrorType::UnclosedCurly));
                    }
                    // return all other errors
                    return Err(e);
                },
                // parse the next stmt
                _=>{
                    let item = match self.parse_stmt() {
                        Ok(s)=>s,
                        Err(e)=>{
                            if e.err_type() == &ErrorType::UnexpectedEOF {
                                let span = self.peek_span();
                                return Err(Error::new(start..span.end, ErrorType::UnclosedCurly));
                            }
                            return Err(e);
                        },
                    };
                    items.push(item);
                },
            }
        }

        let end = self.span().end;

        return Ok(Block {
            span: start..end,
            body: items,
        });
    }

    /// Parse a single expression
    pub fn parse_expr(&mut self)->Result<Expr, Error> {
        let left = match self.peek()? {
            Token::Keyword(Keyword::Copy)=>{
                self.next()?;
                let start = self.span().start;
                let name = self.ident()?;
                let end = self.span().end;
                Expr::Copy(start..end, name)
            },
            Token::Keyword(Keyword::Ref)=>{
                self.next()?;
                let start = self.span().start;
                let var_type = self.parse_var_type()?;

                let name = self.ident()?;
                let end = self.span().end;

                Expr::Ref(start..end, var_type, name)
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
                // Index
                Ok(Token::SquareStart)=>{
                    self.next()?;
                    let start = self.span().start;

                    self.skip_newline();

                    let right = match self.parse_expr() {
                        Ok(e)=>e,
                        Err(e)=>{
                            if e.err_type() == &ErrorType::UnexpectedEOF {
                                let span = self.peek_span();
                                return Err(Error::new(start..span.end, ErrorType::UnclosedSquare));
                            }
                            return Err(e);
                        },
                    };

                    self.skip_newline();

                    match self.next() {
                        Ok(Token::SquareEnd)=>{},
                        Ok(_)=>return Err(Error::token(self.span())),
                        Err(e)=>{
                            if e.err_type() == &ErrorType::UnexpectedEOF {
                                let span = self.peek_span();
                                return Err(Error::new(start..span.end, ErrorType::UnclosedSquare));
                            }
                            return Err(e);
                        },
                    }

                    let end = self.span().end;

                    left = Expr::Index(start..end, Box::new([left, right]));
                },
                // Field
                Ok(Token::Dot)=>{
                    self.next()?;
                    let start = self.span().start;
                    let name = self.ident()?;
                    let end = self.span().end;

                    left = Expr::Field(start..end, Box::new(left), name);
                },
                // Function call
                Ok(Token::ParenStart)=>{
                    let start = self.peek_span().start;
                    let mut items = self.parse_paren_list(Self::parse_expr)?;

                    if items.len() > u8::MAX as usize {
                        self.push_err(Error::new(self.span(), ErrorType::TooManyArgs));
                    }

                    let end = self.span().end;

                    left = match left {
                        Expr::Field(span, left, sym)=>{
                            items.insert(0, *left);

                            Expr::MethodCall(span.start..end, sym, items)
                        },
                        Expr::Named(span, name)=>Expr::AssociatedCall(span.start..end, name, items),
                        d=>{
                            items.insert(0, d);
                            Expr::Call(start..end, items)
                        },
                    };
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
        let start = self.span().start;

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
                Err(e)=>{
                    if e.err_type() == &ErrorType::UnexpectedEOF {
                        let span = self.peek_span();
                        return Err(Error::new(start..span.end, ErrorType::UnclosedParen));
                    }
                    return Err(e);
                },
                // otherwise parse the next expression
                _=>{
                    let item = match f(self) {
                        Ok(e)=>e,
                        Err(e)=>{
                            if e.err_type() == &ErrorType::UnexpectedEOF {
                                let span = self.peek_span();
                                return Err(Error::new(start..span.end, ErrorType::UnclosedParen));
                            }
                            return Err(e);
                        },
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
                Ok(_)=>return Err(Error::new(self.span(), ErrorType::ExpectedToken(")".to_string()))),
                Err(e)=>{
                    // EOF errors are converted to unclosed paren errors
                    if e.err_type() == &ErrorType::UnexpectedEOF {
                        let span = self.peek_span();
                        return Err(Error::new(start..span.end, ErrorType::UnclosedParen));
                    }
                    // return other errors
                    return Err(e);
                },
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
        let start = self.peek_span().start;
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
        let end = self.span().end;

        return Ok(Expr::BinaryOp(start..end, op, Box::new([left, right])));
    }

    /// parse a unary expression
    fn parse_unary_op_expr(&mut self)->Result<Expr, Error> {
        // determine which operation we have
        let op = match self.next()? {
            Token::Sub=>UnaryOp::Negate,
            Token::Not=>UnaryOp::Not,
            _=>return Err(Error::token(self.span())),
        };
        let start = self.span().start;

        // parse the right side
        let expr = self.parse_paren_expr()?;
        let end = self.span().end;

        return Ok(Expr::UnaryOp(start..end, op, Box::new(expr)));
    }

    /// parse an expression in parenthesis or a literal expression
    fn parse_paren_expr(&mut self)->Result<Expr, Error> {
        let left = match self.peek()? {
            Token::ParenStart=>{
                // consume the parenthesis
                self.next()?;

                // store the start
                let start = self.span().start;

                // parse the inner
                let expr = match self.parse_expr() {
                    Ok(e)=>e,
                    Err(e)=>{
                        if e.err_type() == &ErrorType::UnexpectedEOF {
                            let span = self.peek_span();
                            return Err(Error::new(start..span.end, ErrorType::UnclosedParen));
                        }
                        return Err(e);
                    },
                };

                // convert errors to unclosed paren errors
                match self.try_next(Token::ParenEnd) {
                    Err(_)=>{
                        let span = self.peek_span();
                        return Err(Error::new(start..span.end, ErrorType::UnclosedParen));
                    },
                    _=>{},
                }

                Ok(expr)
            },
            _=>self.parse_literal_expr(),
        }?;

        return self.parse_tail_expr(left);
    }

    /// parse a literal expression
    fn parse_literal_expr(&mut self)->Result<Expr, Error> {
        let start = self.peek_span();
        match self.next()? {
            Token::Ident(i)=>match self.peek() {
                Ok(Token::ColonColon)=>{
                    self.next()?;
                    let right = self.ident()?;
                    let end = self.span().end;

                    Ok(Expr::AssociatedValue(start.start..end, i, right))
                },
                _=>Ok(Expr::Named(start, i)),
            },
            Token::Integer(i)=>Ok(Expr::Integer(start, i)),
            Token::Float(f)=>Ok(Expr::Float(start, f)),
            Token::String(s)=>Ok(Expr::String(start, s)),
            Token::Keyword(Keyword::True)=>Ok(Expr::Bool(start, true)),
            Token::Keyword(Keyword::False)=>Ok(Expr::Bool(start, false)),
            Token::CurlyStart=>{
                let start = self.span().start;
                let mut items = Vec::new();

                loop {
                    self.skip_newline();

                    match self.peek() {
                        Ok(Token::CurlyEnd)=>{
                            self.next()?;
                            break;
                        },
                        Ok(_)=>{
                            let name = self.ident()?;
                            let name_span = self.span();

                            self.try_next(Token::Colon)?;

                            let expr = match self.parse_expr() {
                                Ok(e)=>e,
                                Err(e)=>{
                                    if e.err_type() == &ErrorType::UnexpectedEOF {
                                        let span = self.peek_span();
                                        return Err(Error::new(start..span.end, ErrorType::UnclosedCurly));
                                    }
                                    return Err(e);
                                },
                            };

                            items.push((name_span, name, expr));
                        },
                        Err(e)=>{
                            if e.err_type() == &ErrorType::UnexpectedEOF {
                                let span = self.peek_span();
                                return Err(Error::new(start..span.end, ErrorType::UnclosedCurly));
                            }
                            return Err(e);
                        },
                    }

                    self.skip_newline();

                    match self.next() {
                        Ok(Token::CurlyEnd)=>break,
                        Ok(Token::Comma)=>{},
                        Ok(_)=>return Err(Error::token(self.span())),
                        Err(e)=>{
                            if e.err_type() == &ErrorType::UnexpectedEOF {
                                let span = self.peek_span();
                                return Err(Error::new(start..span.end, ErrorType::UnclosedCurly));
                            }
                            return Err(e);
                        },
                    }
                }
                let end = self.span().end;

                Ok(Expr::Object(start..end, items))
            },
            Token::SquareStart=>{
                let start = self.span().start;
                let mut items = Vec::new();

                loop {
                    self.skip_newline();

                    match self.peek() {
                        Ok(Token::SquareEnd)=>{
                            self.next()?;
                            break;
                        },
                        Ok(_)=>match self.parse_expr() {
                            Ok(e)=>items.push(e),
                            Err(e)=>{
                                if e.err_type() == &ErrorType::UnexpectedEOF {
                                    let span = self.peek_span();
                                    return Err(Error::new(start..span.end, ErrorType::UnclosedSquare));
                                }
                                return Err(e);
                            },
                        },
                        Err(e)=>{
                            if e.err_type() == &ErrorType::UnexpectedEOF {
                                let span = self.peek_span();
                                return Err(Error::new(start..span.end, ErrorType::UnclosedSquare));
                            }
                            return Err(e);
                        },
                    }

                    self.skip_newline();

                    match self.next() {
                        Ok(Token::SquareEnd)=>break,
                        Ok(Token::Comma)=>{},
                        Ok(_)=>return Err(Error::token(self.span())),
                        Err(e)=>{
                            if e.err_type() == &ErrorType::UnexpectedEOF {
                                let span = self.peek_span();
                                return Err(Error::new(start..span.end, ErrorType::UnclosedSquare));
                            }
                            return Err(e);
                        },
                    }
                }
                let end = self.span().end;

                Ok(Expr::List(start..end, items))
            },
            _=>Err(Error::token(self.span())),
        }
    }
}
