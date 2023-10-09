use logos::Span;
use string_interner::DefaultSymbol as Symbol;
use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult,
};


pub trait GetSpan {
    fn span(&self)->Span;
}


#[derive(Debug)]
pub enum Stmt {
    Function(Span, Function),
    DeleteVar(Span, Symbol),
    Class {
        span: Span,
        name: Symbol,
        // TODO: types
        fields: Vec<(VarType, Symbol)>,

        methods: Vec<Function>,
    },
    CreateConst {
        span: Span,
        name: Symbol,
        data: Expr,
    },
    CreateVar {
        span: Span,
        var_type: VarType,
        name: Symbol,
        data: Option<Expr>,
    },
    SetVar {
        span: Span,
        left: Expr,
        data: Expr,
    },
    If {
        span: Span,
        conditions: Vec<(Expr, Block)>,
        default: Option<Block>,
    },
    WhileLoop {
        span: Span,
        condition: Expr,
        body: Block,
    },
    Expression(Span, Expr),
    Return(Span, Option<Expr>),
    Continue(Span),
    Break(Span),
}
impl GetSpan for Stmt {
    fn span(&self)->Span {
        use Stmt::*;
        match self {
            Function(span, _)|
                DeleteVar(span, _)|
                Class{span, ..}|
                CreateConst{span,..}|
                CreateVar{span,..}|
                SetVar{span,..}|
                If{span,..}|
                WhileLoop{span,..}|
                Expression(span, _)|
                Return(span, _)|
                Continue(span)|
                Break(span)=>span.clone(),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    // Copy a variable instead of move.
    Copy(Span, Symbol),
    BinaryOp(Span, BinaryOp, Box<[Self;2]>),
    UnaryOp(Span, UnaryOp, Box<Self>),
    Integer(Span, i64),
    Float(Span, f64),
    String(Span, Symbol),
    Named(Span, Symbol),
    Field(Span, Box<Self>, Symbol),
    // the first item is the thing we call, or the function/method name, etc.
    Call(Span, Vec<Self>),
    Bool(Span, bool),
    Ref(Span, VarType, Symbol),
    List(Span, Vec<Self>),
    Index(Span, Box<[Self;2]>),
    This(Span),
}
impl GetSpan for Expr {
    fn span(&self)->Span {
        use Expr::*;
        match self {
            Copy(span,..)|
                BinaryOp(span,..)|
                UnaryOp(span,..)|
                Integer(span,..)|
                Float(span,..)|
                String(span,..)|
                Named(span,..)|
                Field(span,..)|
                Call(span,..)|
                Bool(span,..)|
                Ref(span,..)|
                List(span,..)|
                Index(span,..)|
                This(span,..)=>span.clone(),
        }
    }
}
impl Expr {
    fn is_literal(&self)->bool {
        use Expr::*;
        match self {
            Named(..)|String(..)|Float(..)|Integer(..)|Bool(..)|List(..)|This(..)=>true,
            _=>false,
        }
    }

    fn is_trailing_expr(&self)->bool {
        use Expr::*;
        match self {
            Field(..)|Call(..)|Index(..)=>true,
            _=>false,
        }
    }
}
impl Display for Expr {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        use Expr::*;
        match self {
            // we don't have access to the string interner, so we make do by showing that symbols
            // are a placeholder
            Copy(_, sym)=>write!(f, "copy <{:?}>", sym)?,
            Named(_, sym)=>write!(f, "<{:?}>", sym)?,
            String(_, sym)=>write!(f, "\"<{:?}>\"", sym)?,
            Integer(_, i)=>write!(f,"{}", i)?,
            Float(_, i)=>write!(f,"{}", i)?,
            Bool(_, b)=>write!(f,"{}", b)?,
            Ref(_, var_type, sym)=>write!(f,"ref {} <{:?}>", var_type, sym)?,
            This(_)=>write!(f,"this")?,
            List(_, items)=>{
                write!(f,"[")?;
                if items.len()>0 {
                    for item in &items[..items.len()-1] {
                        if f.alternate() {
                            write!(f,"{}, ",item)?;
                        } else {
                            write!(f,"{},",item)?;
                        }
                    }
                    write!(f,"{}",items.last().unwrap())?;
                }
                write!(f,"]")?;
            },
            Index(_, items)=>write!(f,"{}[{}]",items[0],items[1])?,
            BinaryOp(_, op, items)=>{
                // parenthesize the left if it is not a literal expression
                if items[0].is_literal() {
                    write!(f, "{}", items[0])?;
                } else {
                    write!(f, "({})", items[0])?;
                }

                // add spaces if we need to and print the operator
                if f.alternate() {
                    write!(f, " {} ", op)?;
                } else {
                    write!(f, "{}", op)?;
                }

                // parenthesize the right if it is not a literal expression
                if items[1].is_literal() {
                    write!(f, "{}", items[1])?;
                } else {
                    write!(f, "({})", items[1])?;
                }
            },
            UnaryOp(_, op, item)=>{
                // print the operator
                write!(f, "{}", op)?;

                // parenthesize the inner expression if it is not a literal
                if item.is_literal() {
                    write!(f, "{}", item)?;
                } else {
                    write!(f, "({})", item)?;
                }
            },
            Field(_, left, name)=>if left.is_literal()||left.is_trailing_expr() {
                // the left side is a literal, field, or call
                write!(f, "{}.<{:?}>", left, name)?;
            } else {
                // add parenthesis to a complex left expression
                write!(f, "({}).<{:?}>", left, name)?;
            },
            Call(_, items)=>{
                if items[0].is_literal()||items[0].is_trailing_expr() {
                    write!(f,"{}(", items[0])?;
                } else {
                    // add parenthesis to a complex left expression
                    write!(f,"({})(", items[0])?;
                }
                // if we have arguments to the call, print them
                if items.len() > 1 {
                    // print second to second-to-last args
                    for item in &items[1..items.len()-1] {
                        write!(f, "{}", item)?;
                        // add space if needed
                        if f.alternate() {
                            write!(f, ", ")?;
                        } else {
                            write!(f, ",")?;
                        }
                    }
                    write!(f, "{}", items.last().unwrap())?;
                }
                write!(f, ")")?;
            },
        }

        return Ok(());
    }
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    LogicAnd,
    LogicOr,
}
impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        match self {
            Self::Add=>write!(f,"+"),
            Self::Sub=>write!(f,"-"),
            Self::Mul=>write!(f,"*"),
            Self::Div=>write!(f,"/"),
            Self::Mod=>write!(f,"%"),
            Self::Equal=>write!(f,"=="),
            Self::NotEqual=>write!(f,"!="),
            Self::Greater=>write!(f,">"),
            Self::Less=>write!(f,"<"),
            Self::GreaterEqual=>write!(f,">="),
            Self::LessEqual=>write!(f,"<="),
            Self::LogicAnd=>write!(f,"and"),
            Self::LogicOr=>write!(f,"or"),
        }
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    Negate,
    Not,
}
impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        match self {
            Self::Negate=>write!(f,"-"),
            Self::Not=>write!(f,"!"),
        }
    }
}


bitflags::bitflags! {
    #[derive(Debug)]
    pub struct VarType: u32 {
        /// Allows assigning a new value of the same type to the container.
        /// example: `set x = 5`
        const REASSIGN =    0b01;
        /// Allows mutation of the data in the container
        /// example: `list.push(5)`
        const MUTATE =      0b10;
    }
}
impl Display for VarType {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        if self.contains(VarType::MUTATE) {
            write!(f, "mut ")?;
        }

        if self.contains(VarType::REASSIGN) {
            write!(f, "var")
        } else {
            write!(f,"let")
        }
    }
}


#[derive(Debug)]
pub struct Function {
    pub span: Span,
    pub name: Symbol,
    // TODO: types
    pub params: Vec<(Span, VarType, Symbol)>,
    pub body: Block,
}
impl GetSpan for Function {
    fn span(&self)->Span {self.span.clone()}
}

#[derive(Debug)]
pub struct Block {
    pub span: Span,
    pub body: Vec<Stmt>,
}
impl GetSpan for Block {
    fn span(&self)->Span {self.span.clone()}
}
