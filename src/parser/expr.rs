use string_interner::DefaultSymbol as Symbol;
use logos::Span;
use std::ops::{
    Deref,
    DerefMut,
};
use crate::{
    lexer::*,
    ast::*,
    error::*,
};
use super::Parser;


#[derive(Debug)]
pub enum ExprItem {
    Expr(Expr),
    Integer(Span, i64),
    Float(Span, f64),
    String(Span, String),
    Ident(Span, Symbol),
}
impl ExprItem {
    pub fn to_expr(self)->Expr {
        match self {
            Self::Expr(e)=>e,
            Self::Integer(sp, i)=>Expr::Integer(sp, i),
            Self::Float(sp, f)=>Expr::Float(sp, f),
            Self::String(sp, s)=>Expr::String(sp, s),
            Self::Ident(sp, i)=>Expr::Named(sp, i),
        }
    }
}

pub enum Associvity {
    Left,
    Right,
    Paren,
}

pub enum OpType {
    Infix,
    Prefix,
    Postfix,
}

#[derive(Debug, Copy, Clone)]
pub enum Operator {
    // arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // logic
    LogicAnd,
    LogicOr,

    // equality
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    // unary
    Negate,
    Not,

    // misc
    Index,
    IndexEnd,
    Field,
    Call,
    CallEnd,
    Comma,
}
impl Operator {
    pub fn as_binary_op(&self)->BinaryOp {
        use Operator::*;
        match self {
            Add=>BinaryOp::Add,
            Sub=>BinaryOp::Sub,
            Mul=>BinaryOp::Mul,
            Div=>BinaryOp::Div,
            Mod=>BinaryOp::Mod,
            LogicAnd=>BinaryOp::LogicAnd,
            LogicOr=>BinaryOp::LogicOr,
            Equal=>BinaryOp::Equal,
            NotEqual=>BinaryOp::NotEqual,
            Greater=>BinaryOp::Greater,
            Less=>BinaryOp::Less,
            GreaterEqual=>BinaryOp::GreaterEqual,
            LessEqual=>BinaryOp::LessEqual,
            _=>panic!("Not allowed"),
        }
    }

    pub fn as_unary_op(&self)->UnaryOp {
        use Operator::*;
        match self {
            Negate=>UnaryOp::Negate,
            Not=>UnaryOp::Not,
            _=>panic!("Not allowed"),
        }
    }

    pub fn operator_type(&self)->OpType {
        use Operator::*;
        match self {
            Add|
                Sub|
                Mul|
                Div|
                Mod|
                LogicAnd|
                LogicOr|
                Equal|
                NotEqual|
                Greater|
                Less|
                GreaterEqual|
                LessEqual=>OpType::Infix,

            Negate|
                Not=>OpType::Prefix,

            Index|
                IndexEnd|
                Field|
                Call|
                CallEnd|
                Comma=>OpType::Postfix,
        }
    }

    pub fn associvity(&self)->Associvity {
        use Operator::*;
        use Associvity::*;
        match self {
            Add|
                Sub|
                Mul|
                Div|
                Mod|
                LogicAnd|
                LogicOr|
                Index|
                IndexEnd|
                Field|
                Call|
                CallEnd|
                Comma=>Left,

            Negate|
                Not=>Right,

            Equal|
                NotEqual|
                Greater|
                Less|
                GreaterEqual|
                LessEqual=>Paren,
        }
    }

    pub fn base_prec(&self)->usize {
        use Operator::*;
        match self {
            // least binding
            CallEnd|
                IndexEnd|
                Comma=>0,

            LogicOr=>2,

            LogicAnd=>4,

            Equal|
                NotEqual|
                Greater|
                Less|
                GreaterEqual|
                LessEqual=>6,

            Add|
                Sub=>8,

            Mul|
                Div|
                Mod=>10,

            Negate|
                Not=>12,

            Index|
                Field|
                Call=>14,
        }
    }

    pub fn l_prec(&self)->Option<usize> {
        use Associvity::*;

        let base = self.base_prec();

        match self.associvity() {
            Left=>Some(base),
            Right=>Some(base + 1),
            Paren=>None,
        }
    }

    pub fn r_prec(&self)->Option<usize> {
        use Associvity::*;

        let base = self.base_prec();

        match self.associvity() {
            Left=>Some(base + 1),
            Right=>Some(base),
            Paren=>None,
        }
    }
}


/// A pratt-parser to parse expressions
pub struct ExprParser<'a, 'p> {
    parent: &'p mut Parser<'a>,
}
impl<'a, 'p> ExprParser<'a, 'p> {
    pub fn new(parent: &'p mut Parser<'a>)->Self {
        ExprParser {
            parent,
        }
    }

    pub fn parse(&mut self)->Result<Expr, Error> {
        self.parse_inner(2).map(ExprItem::to_expr)
    }

    fn parse_inner(&mut self, min_prec: usize)->Result<ExprItem, Error> {
        let mut left = match self.peek()? {
            Token::Integer(..)|
                Token::Float(..)|
                Token::Ident(..)=>self.parse_literal()?,
            Token::Sub|
                Token::Not=>{
                    let op = match self.next()? {
                        Token::Sub=>Operator::Negate,
                        Token::Not=>Operator::Not,
                        _=>unreachable!(),
                    };
                    let start = self.span().start;

                    let rhs = self.parse_inner(op.r_prec().unwrap())?;
                    let end = self.span().end;

                    ExprItem::Expr(Expr::UnaryOp(
                        start..end,
                        op.as_unary_op(),
                        Box::new(rhs.to_expr()),
                    ))
                },
            Token::ParenStart=>{
                self.next()?;
                let l = self.parse_inner(2)?;
                self.try_next(Token::ParenEnd)?;
                l
            },
            _=>return Err(Error::token(self.span())),
        };

        loop {
            let Some(operator) = self.peek_operator() else {
                break;
            };

            let Some(l_prec) = operator.l_prec() else {
                todo!("Paren associvity");
            };

            if l_prec < min_prec {
                break;
            }

            // consume the operator from above
            self.next()?;

            match operator.operator_type() {
                OpType::Infix=>{
                    let Some(r_prec) = operator.r_prec() else {
                        todo!("Paren associvity");
                    };

                    self.skip_newline();

                    let right = self.parse_inner(r_prec)?;

                    left = self.convert_to_bin_expr(left, operator, right);
                },
                OpType::Postfix=>{
                    match operator {
                        Operator::Field=>{
                            let name = self.ident()?;
                            left = ExprItem::Expr(Expr::Field(
                                self.span(),
                                Box::new(left.to_expr()),
                                name,
                            ));
                        },
                        Operator::Index=>{
                            let start = self.span().start;
                            let expr = self.parse_inner(2)?;
                            self.try_next(Token::SquareEnd)?;
                            let end = self.span().end;
                            left = ExprItem::Expr(Expr::Index(
                                start..end,
                                Box::new([left.to_expr(), expr.to_expr()]),
                            ));
                        },
                        Operator::Call=>{
                            let start = self.span().start;
                            let mut items = vec![
                                left.to_expr(),
                            ];

                            loop {
                                match self.peek()? {
                                    Token::ParenEnd=>{
                                        self.next()?;
                                        break;
                                    },
                                    _=>{},
                                }
                                items.push(self.parse_inner(2)?.to_expr());

                                match self.next()? {
                                    Token::Comma=>{},
                                    Token::ParenEnd=>break,
                                    _=>return Err(Error::token(self.span())),
                                }
                            }

                            let end = self.span().end;

                            left = ExprItem::Expr(Expr::Call(
                                start..end,
                                items,
                            ));
                        },
                        _=>unreachable!(),
                    }
                },
                _=>unreachable!(),
            }
        }

        return Ok(left);
    }

    fn convert_to_bin_expr(&self, left: ExprItem, op: Operator, right: ExprItem)->ExprItem {
        let left = left.to_expr();
        let right = right.to_expr();

        ExprItem::Expr(Expr::BinaryOp(
            left.span().start..right.span().end,
            op.as_binary_op(),
            Box::new([left, right]),
        ))
    }

    fn peek_operator(&mut self)->Option<Operator> {
        match self.peek() {
            Ok(Token::Add)=>Some(Operator::Add),
            Ok(Token::Sub)=>Some(Operator::Sub),
            Ok(Token::Mul)=>Some(Operator::Mul),
            Ok(Token::Div)=>Some(Operator::Div),
            Ok(Token::Mod)=>Some(Operator::Mod),
            Ok(Token::Equal)=>Some(Operator::Equal),
            Ok(Token::NotEqual)=>Some(Operator::NotEqual),
            Ok(Token::Greater)=>Some(Operator::Greater),
            Ok(Token::Less)=>Some(Operator::Less),
            Ok(Token::GreaterEqual)=>Some(Operator::GreaterEqual),
            Ok(Token::LessEqual)=>Some(Operator::LessEqual),
            Ok(Token::Keyword(Keyword::And))=>Some(Operator::LogicAnd),
            Ok(Token::Keyword(Keyword::Or))=>Some(Operator::LogicOr),
            Ok(Token::SquareStart)=>Some(Operator::Index),
            Ok(Token::Dot)=>Some(Operator::Field),
            Ok(Token::ParenStart)=>Some(Operator::Call),
            Ok(Token::ParenEnd)=>Some(Operator::CallEnd),
            Ok(Token::SquareEnd)=>Some(Operator::IndexEnd),
            Ok(Token::Comma)=>Some(Operator::Comma),
            _=>None,
        }
    }

    fn parse_literal(&mut self)->Result<ExprItem, Error> {
        // TODO: match lists, objects, etc.
        match self.next()? {
            Token::Integer(i)=>Ok(ExprItem::Integer(self.span(), i)),
            Token::Float(f)=>Ok(ExprItem::Float(self.span(), f)),
            Token::String(s)=>Ok(ExprItem::String(self.span(), s)),
            Token::Ident(i)=>Ok(ExprItem::Ident(self.span(), i)),
            _=>Err(Error::token(self.span())),
        }
    }
}
impl<'a, 'p> Deref for ExprParser<'a, 'p> {
    type Target = Parser<'a>;
    fn deref(&self)->&Parser<'a> {
        self.parent
    }
}
impl<'a, 'p> DerefMut for ExprParser<'a, 'p> {
    fn deref_mut(&mut self)->&mut Parser<'a> {
        self.parent
    }
}
