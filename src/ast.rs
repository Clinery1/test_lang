use string_interner::DefaultSymbol as Symbol;
use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult,
};


pub type Block = Vec<Stmt>;


#[derive(Debug)]
pub enum Stmt {
    Function(Function),
    DeleteVar(Symbol),
    Class {
        name: Symbol,
        // TODO: types
        fields: Vec<(VarType, Symbol)>,

        methods: Vec<Function>,
    },
    CreateConst {
        name: Symbol,
        data: Expr,
    },
    CreateVar {
        var_type: VarType,
        name: Symbol,
        data: Option<Expr>,
    },
    SetVar {
        left: Expr,
        data: Expr,
    },
    If {
        conditions: Vec<(Expr, Block)>,
        default: Option<Block>,
    },
    WhileLoop {
        condition: Expr,
        body: Block,
    },
    Expression(Expr),
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
    Ref(VarType, Symbol),
    This,
}
impl Expr {
    fn is_literal(&self)->bool {
        use Expr::*;
        match self {
            Named(_)|String(_)|Float(_)|Integer(_)|Bool(_)|This=>true,
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
            // we don't have access to the string interner, so we make do by showing that symbols
            // are a placeholder
            Copy(sym)=>write!(f, "copy <{:?}>", sym)?,
            Named(sym)=>write!(f, "<{:?}>", sym)?,
            String(sym)=>write!(f, "\"<{:?}>\"", sym)?,
            Integer(i)=>write!(f,"{}", i)?,
            Float(i)=>write!(f,"{}", i)?,
            Bool(b)=>write!(f,"{}", b)?,
            Ref(var_type, sym)=>write!(f,"ref {} <{:?}>", var_type, sym)?,
            This=>write!(f,"this")?,
            BinaryOp(op, items)=>{
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
            UnaryOp(op, item)=>{
                // print the operator
                write!(f, "{}", op)?;

                // parenthesize the inner expression if it is not a literal
                if item.is_literal() {
                    write!(f, "{}", item)?;
                } else {
                    write!(f, "({})", item)?;
                }
            },
            Field(left, name)=>if left.is_literal()||left.is_field_call() {
                // the left side is a literal, field, or call
                write!(f, "{}.<{:?}>", left, name)?;
            } else {
                // add parenthesis to a complex left expression
                write!(f, "({}).<{:?}>", left, name)?;
            },
            Call(items)=>{
                if items[0].is_literal()||items[0].is_field_call() {
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
    pub name: Symbol,
    // TODO: types
    pub params: Vec<Symbol>,
    pub body: Block,
}
