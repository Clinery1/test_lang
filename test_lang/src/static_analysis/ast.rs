use logos::Span;
use string_interner::DefaultSymbol as Symbol;
use crate::{
    ast::{
        Permissions,
        FunctionType,
        BinaryOp,
        UnaryOp,
    },
};
use super::utils::*;


pub enum Stmt {
    CreateVar {
        span: Span,
        perms: Permissions,
        name: Symbol,
        slot: VarSlot,
        init: Option<ExprId>,
    },
    SetVar {
        span: Span,
        name: Symbol,
        slot: VarSlot,
        fields: Vec<Symbol>,
        data: ExprId,
    },
    CreateConst {
        span: Span,
        name: Symbol,
        slot: VarSlot,
        init: ExprId,
    },
    While {
        span: Span,
        condition: ExprId,
        block: BlockId,
    },
    If {
        span: Span,
        conditions: Vec<(ExprId, BlockId)>,
        default: Option<BlockId>,
    },
    Class(Span, Class),
    Expression(Span, ExprId),
    Return(Span, Option<ExprId>),
    Continue(Span),
    Break(Span),
    Print(Span, ExprId),
    Function(Span, FunctionId),
    DeleteVar(Span, VarSlot),
}

/// Each one of these is assigned to an SSA variable and used exactly once.
pub enum SSAExpr {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),

    // Composite literals
    Object(Vec<(Symbol, SSAId)>),
    List(Vec<SSAId>),

    // Operation
    BinaryOp(SSAId, BinaryOp, SSAId),
    UnaryOp(UnaryOp, SSAId),
    Call(SSAId, Vec<SSAId>),
    Index(SSAId, SSAId),
    Field(SSAId, Symbol),

    // Misc
    VarSlot(VarSlot),
    ExternExpr(ExprId),
    AssociatedValue(Symbol, Symbol),
}


pub struct AnalysisFile {
    functions: SlotMap<FunctionId, Function>,
    classes: SlotMap<ClassId, Class>,
    exprs: SlotMap<ExprId, Expr>,
    blocks: SlotMap<BlockId, Block>,
}

pub struct Block {
    pub parent: Option<BlockId>,
    pub children: Vec<BlockId>,
    pub body: Vec<Stmt>,
}

pub struct Function {
    pub span: Span,
    pub name: Symbol,
    pub id: FunctionId,
    pub func_type: FunctionType,
    pub perms: Permissions,
    pub params: Vec<(Span, Permissions, Symbol)>,
    pub body: BlockId,
}

pub struct Class {
    pub span: Span,
    pub id: ClassId,
    pub perms: Permissions,
    pub name: Symbol,
    pub constructor: Option<FunctionId>,
    pub fields: Vec<(Permissions, Symbol)>,
    pub methods: Vec<FunctionId>,
    pub associated: Vec<FunctionId>,
}

pub struct Expr {
    pub inner_ssa: Vec<SSAExpr>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FunctionId(usize);
impl Key for FunctionId {
    fn from_id(id: usize)->Self {FunctionId(id)}
    fn get_id(&self)->usize {self.0}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ClassId(usize);
impl Key for ClassId {
    fn from_id(id: usize)->Self {ClassId(id)}
    fn get_id(&self)->usize {self.0}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ExprId(usize);
impl Key for ExprId {
    fn from_id(id: usize)->Self {ExprId(id)}
    fn get_id(&self)->usize {self.0}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SSAId(usize);
impl Key for SSAId {
    fn from_id(id: usize)->Self {SSAId(id)}
    fn get_id(&self)->usize {self.0}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BlockId(usize);
impl Key for BlockId {
    fn from_id(id: usize)->Self {BlockId(id)}
    fn get_id(&self)->usize {self.0}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct VarSlot(usize);
