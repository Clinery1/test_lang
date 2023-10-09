use string_interner::DefaultSymbol as Symbol;
use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult,
};


#[derive(Debug)]
pub enum Stmt {
    Function(Function),
    DeleteVar(Symbol),
    Class {
        name: Symbol,
        // TODO: types
        fields: Vec<Symbol>,

        methods: Vec<Function>,
    },
    CreateVar {
        mutability: VarType,
        name: Symbol,
        data: Option<Expr>,
    },
    SetVar {
        name: Symbol,
        data: Expr,
    },
}

#[derive(Debug)]
pub enum Expr {
    // Copy a variable instead of move.
    Copy(Symbol),
    BinaryOp(BinaryOp, Box<[Self;2]>),
    UnaryOp(UnaryOp, Box<Self>),
    Integer(u64),
    Float(f64),
    String(Symbol),
    Named(Symbol),
    Field(Box<Self>, Symbol),
    // the first item is the thing we call, or the function/method name, etc.
    Call(Vec<Self>),
    Bool(bool),
}
impl Expr {
    fn is_literal(&self)->bool {
        use Expr::*;
        match self {
            Named(_)|String(_)|Float(_)|Integer(_)|Bool(_)=>true,
            _=>false,
        }
    }

    fn is_field_call(&self)->bool {
        use Expr::*;
        match self {
            Field(..)|Call(..)=>true,
            _=>false,
        }
    }
}
impl Display for Expr {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        use Expr::*;
        match self {
            Copy(sym)=>write!(f, "copy <{:?}>", sym)?,
            Named(sym)=>write!(f, "<{:?}>", sym)?,
            String(sym)=>write!(f, "\"<{:?}>\"", sym)?,
            Integer(i)=>write!(f,"{}", i)?,
            Float(i)=>write!(f,"{}", i)?,
            Bool(b)=>write!(f,"{}", b)?,
            BinaryOp(op, items)=>{
                if items[0].is_literal() {
                    write!(f, "{}", items[0])?;
                } else {
                    write!(f, "({})", items[0])?;
                }

                if f.alternate() {
                    write!(f, " {} ", op)?;
                } else {
                    write!(f, "{}", op)?;
                }

                if items[1].is_literal() {
                    write!(f, "{}", items[1])?;
                } else {
                    write!(f, "({})", items[1])?;
                }
            },
            UnaryOp(op, item)=>{
                write!(f, "{}", op)?;
                if item.is_literal() {
                    write!(f, "{}", item)?;
                } else {
                    write!(f, "({})", item)?;
                }
            },
            Field(left, name)=>if left.is_literal()||left.is_field_call() {
                write!(f, "{}.<{:?}>", left, name)?;
            } else {
                write!(f, "({}).<{:?}>", left, name)?;
            },
            Call(items)=>{
                if items[0].is_literal()||items[0].is_field_call() {
                    write!(f,"{}(", items[0])?;
                } else {
                    write!(f,"({})(", items[0])?;
                }
                if items.len() > 1 {
                    for item in &items[1..items.len()-1] {
                        write!(f, "{}", item)?;
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
        const CONST =   0b001;
        const VAR =     0b010;
        const VAL =     0b000;
        const MUT =     0b100;
    }
}


#[derive(Debug)]
pub struct Function {
    name: Symbol,
    // TODO: types
    params: Vec<Symbol>,
    body: Vec<Stmt>,
}
