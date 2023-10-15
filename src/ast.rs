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
        permissions: Permissions,
        name: Symbol,
        // TODO: types
        fields: Vec<(Permissions, Symbol)>,
        methods: Vec<Function>,
        associated: Vec<Function>,
    },
    CreateConst {
        span: Span,
        name: Symbol,
        data: Expr,
    },
    CreateVar {
        span: Span,
        var_type: Permissions,
        name: Symbol,
        data: Option<Expr>,
    },
    SetVar {
        span: Span,
        left: Vec<Symbol>,
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
    Interface {
        span: Span,
        permissions: Permissions,
        name: Symbol,
        methods: Vec<FunctionSignature>,
        associated: Vec<FunctionSignature>,
    },
    Enum {
        span: Span,
        permissions: Permissions,
        name: Symbol,
        items: Vec<EnumItem>,
    },
    InterfaceImpl {
        span: Span,
        interface_name: Symbol,
        class_name: Symbol,
        methods: Vec<Function>,
        associated: Vec<Function>,
    },
    Expression(Span, Expr),
    Return(Span, Option<Expr>),
    Continue(Span),
    Break(Span),
    Print(Span, Expr),
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
                Interface{span,..}|
                Enum{span,..}|
                InterfaceImpl{span,..}|
                Expression(span, _)|
                Return(span, _)|
                Continue(span)|
                Break(span)|
                Print(span, _)=>span.clone(),
        }
    }
}

#[derive(Debug)]
pub enum EnumItem {
    Name(Span, Symbol),
    NameValue(Span, Symbol, i64),
    // TODO: typed enums
    // NameType(Symbol, Span, Type),
    // NameTypeValue(Symbol, Span, Type, i64, Span),
}

#[derive(Debug)]
pub enum Expr {
    // Copy a variable instead of move.
    Copy(Span, Symbol),
    BinaryOp(Span, BinaryOp, Box<[Self;2]>),
    UnaryOp(Span, UnaryOp, Box<Self>),
    Integer(Span, i64),
    Float(Span, f64),
    String(Span, String),
    Named(Span, Symbol),
    Field(Span, Box<Self>, Symbol),
    // the first item is the thing we call, or the function/method name, etc.
    Call(Span, Vec<Self>),
    Bool(Span, bool),
    Ref(Span, Permissions, Symbol),
    List(Span, Vec<Self>),
    Index(Span, Box<[Self;2]>),
    Object(Span, Vec<(Span, Symbol, Self)>),
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
                Object(span,..)=>span.clone(),
        }
    }
}
impl Expr {
    fn is_literal(&self)->bool {
        use Expr::*;
        match self {
            Named(..)|String(..)|Float(..)|Integer(..)|Bool(..)|List(..)|Object(..)=>true,
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
            List(_, items)=>{
                write!(f,"[")?;
                if items.len() > 0 {
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
            Object(_, items)=>{
                write!(f,"{{")?;

                if items.len() > 0 {
                    for (_, name, expr) in &items[..items.len()-1] {
                        write!(f,"<{:?}>: {}, ", name, expr)?;
                    }

                    let (_, name, expr) = items.last().unwrap();
                    write!(f,"<{:?}>: {}", name, expr)?;
                }

                write!(f,"}}")?;
            },
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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub enum FunctionType {
    Method,
    MutableMethod,
    Normal,
}
impl Display for FunctionType {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        match self {
            Self::MutableMethod=>write!(f,"mut "),
            _=>Ok(()),
        }
    }
}


bitflags::bitflags! {
    #[derive(Debug, Copy, Clone, Default)]
    pub struct Permissions: u32 {
        /// Says if this is a variable
        const IS_VARIABLE =     0b100000;

        /// Allows assigning a new value of the same type to the container.
        /// example: `set x = 5`
        const REASSIGN =        0b110000;

        /// Allows mutation of the data in the container
        /// example: `list.push(5)`
        const MUTATE =          0b001000;

        /// A public item with no mutability permissions
        const PUBLIC =          0b000100;

        /// A public mutable item
        const PUBLIC_MUTABLE =  0b000110;

        /// A reassignable public item
        const PUBLIC_REASSIGN = 0b100101;
    }
}
impl Display for Permissions {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        if self.contains(Self::PUBLIC) {
            write!(f, "pub")?;
            if self.contains(Self::PUBLIC_MUTABLE) && self.contains(Self::PUBLIC_REASSIGN) {
                write!(f, "(var mut)")?;
            } else if self.contains(Self::PUBLIC_MUTABLE) {
                write!(f, "(mut)")?;
            } else if self.contains(Self::PUBLIC_REASSIGN) {
                write!(f, "(var)")?;
            }
        }
        if self.contains(Self::MUTATE) {
            write!(f, "mut ")?;
        }

        if self.contains(Self::REASSIGN) {
            write!(f, "var")
        } else {
            write!(f,"let")
        }
    }
}


#[derive(Debug)]
pub struct Function {
    pub permissions: Permissions,
    pub func_type: FunctionType,
    pub id: usize,
    pub span: Span,
    pub name: Symbol,
    // TODO: types
    pub params: Vec<(Span, Permissions, Symbol)>,
    pub body: Block,
}
impl GetSpan for Function {
    fn span(&self)->Span {self.span.clone()}
}

#[derive(Debug)]
pub struct FunctionSignature {
    pub permissions: Permissions,
    pub func_type: FunctionType,
    pub span: Span,
    pub name: Symbol,
    // TODO: types
    pub params: Vec<(Span, Permissions, Symbol)>,
}
impl GetSpan for FunctionSignature {
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
