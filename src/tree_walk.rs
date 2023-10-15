#![allow(dead_code)]

use fnv::{
    FnvHashMap,
    FnvHashSet,
};
use logos::Span;
use string_interner::DefaultSymbol as Symbol;
use crate::{
    ast::*,
    error::*,
};


pub type Object = FnvHashMap<Symbol, (VarType, Data)>;


#[derive(Debug, Clone)]
pub enum Data {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    FunctionPtr(FunctionId),
    Class(Class),
    List(Vec<Self>),
    Object(Object),
    // TakeVar(Span, Symbol),
    None,
}
impl Data {
    // TODO: classes
    pub fn is_copy(&self)->bool {
        use Data::*;
        match self {
            Bool(_)|Integer(_)|Float(_)|FunctionPtr(_)|None=>true,
            _=>false,
        }
    }
    pub fn get_mut_mutate(&mut self, span: Span, sym: Symbol)->Result<&mut Self, Error> {
        match self {
            Self::Object(inner)|Self::Class(Class{inner,..})=>{
                if let Some((var_type, field_mut)) = inner.get_mut(&sym) {
                    if var_type.contains(VarType::MUTATE) {
                        return Ok(field_mut);
                    } else {
                        return Err(Error::new(span, ErrorType::CannotMutate));
                    }
                } else {
                    return Err(Error::new(span, ErrorType::NoField(sym)));
                }
            },
            _=>return Err(Error::new(span, ErrorType::TypeHasNoFields)),
        }
    }

    pub fn get_mut_reassign(&mut self, span: Span, sym: Symbol)->Result<&mut Self, Error> {
        match self {
            Self::Object(inner)|Self::Class(Class{inner,..})=>{
                if let Some((var_type, field_mut)) = inner.get_mut(&sym) {
                    if var_type.contains(VarType::REASSIGN) {
                        return Ok(field_mut);
                    } else {
                        return Err(Error::new(span, ErrorType::CannotReassign));
                    }
                } else {
                    return Err(Error::new(span, ErrorType::NoField(sym)));
                }
            },
            _=>return Err(Error::new(span, ErrorType::TypeHasNoFields)),
        }
    }

    // #[inline]
    // pub fn take_from(self, prog: &mut Interpreter)->Result<Self, Error> {
    //     prog.take_var(self)
    // }
}

#[derive(Debug)]
pub enum OutputData {
    Return(Option<Data>),
    Break,
    Continue,
    None,
}


pub struct Interpreter<'a> {
    functions: FnvHashMap<FunctionId, RTFunction<'a>>,
    global_functions: Vec<(Symbol, FunctionId)>,
    /// stores all scopes. When a function is called, we push a new scope stack with the function's
    /// params and things here
    scope_stack: Vec<ScopeStack>,
}
impl<'a> Interpreter<'a> {
    pub fn new()->Self {
        Interpreter {
            functions: FnvHashMap::default(),
            global_functions: Vec::new(),
            scope_stack: vec![ScopeStack::new()],
        }
    }

    fn scope(&mut self)->&mut ScopeStack {
        if self.scope_stack.len() == 0 {
            self.scope_stack.push(ScopeStack::new());
        }

        self.scope_stack.last_mut().unwrap()
    }

    fn push_scope_stack(&mut self) {
        self.scope_stack.push(ScopeStack::new());
        let scope = self.scope_stack.last_mut().unwrap();
        for (sym, id) in self.global_functions.iter().copied() {
            let func = self.functions.get(&id).unwrap();
            scope.push_var(sym, VarState {
                created_at: func.created_at.clone(),
                last_modified_at: func.created_at.clone(),
                var_type: VarType::empty(),
                taken: None,
                takeable: false,
                data: Some(Data::FunctionPtr(id)),
            }).unwrap();
        }
    }

    fn pop_scope_stack(&mut self) {
        self.scope_stack.pop();
    }

    pub fn interpret_block(&mut self, block: &'a Block)->Result<OutputData, Error> {
        self.scope().push_scope();
        for stmt in block.body.iter() {
            match self.interpret_stmt(stmt)? {
                OutputData::None=>{},
                d=>{
                    self.scope().drop_scope();
                    return Ok(d);
                },
            }
        }

        self.scope().drop_scope();
        return Ok(OutputData::None);
    }

    pub fn interpret_program(&mut self, stmts: &'a [Stmt])->Result<OutputData, Error> {
        let functions = self.register_functions(stmts)?;
        for (sym, id) in functions {
            let func = self.functions.get(&id).unwrap();
            let span = func.created_at.clone();
            self.global_functions.push((sym, id));
            self.scope().push_var(sym, VarState {
                created_at: span.clone(),
                last_modified_at: span,
                var_type: VarType::empty(),
                taken: None,
                takeable: false,
                data: Some(Data::FunctionPtr(id)),
            })?;
        }

        for stmt in stmts {
            match self.interpret_stmt(stmt)? {
                OutputData::Return(d)=>return Ok(OutputData::Return(d)),
                _=>{},
            }
        }

        return Ok(OutputData::None);
    }

    pub fn interpret_stmt_list(&mut self, stmts: &'a [Stmt])->Result<OutputData, Error> {
        for stmt in stmts {
            match self.interpret_stmt(stmt)? {
                OutputData::Return(d)=>return Ok(OutputData::Return(d)),
                _=>{},
            }
        }

        return Ok(OutputData::None);
    }

    pub fn register_functions(&mut self, stmts: &'a [Stmt])->Result<FnvHashMap<Symbol, FunctionId>, Error> {
        let mut funcs = FnvHashMap::default();

        for stmt in stmts {
            // TODO: properly handle functions in ifs, whiles, etc.
            match stmt {
                Stmt::Function(_, func)=>{
                    let id = self.register_function(func)?;
                    if funcs.contains_key(&func.name) {
                        return Err(Error::new(func.span.start..func.span.start, ErrorType::FunctionExists));
                    }
                    funcs.insert(func.name, id);
                },
                _=>{},
            }
        }

        return Ok(funcs);
    }

    pub fn register_function(&mut self, func: &'a Function)->Result<FunctionId, Error> {
        let id = FunctionId(func.id);

        let body = &func.body.body;
        let params = &func.params;
        let inner_functions = self.register_functions(body)?;

        self.functions.insert(id, RTFunction {
            name: func.name,
            created_at: func.span.clone(),
            inner_functions,
            params,
            body,
        });

        return Ok(id);
    }

    pub fn interpret_stmt(&mut self, stmt: &'a Stmt)->Result<OutputData, Error> {
        match stmt {
            // we do nothing here because we add functions in a previous pass over the AST
            Stmt::Function(..)=>Ok(OutputData::None),
            Stmt::DeleteVar(s, sym)=>{
                self.scope().take_var(s.clone(), *sym)?;
                Ok(OutputData::None)
            },
            // Stmt::Class{span,name,fields,methods}=>{
            //     todo!();
            // },
            Stmt::CreateConst{span,name,data}=>{
                let data = self.interpret_expr(data)?;
                let state = VarState {
                    created_at: span.clone(),
                    last_modified_at: span.clone(),
                    var_type: VarType::empty(),
                    taken: None,
                    takeable: false,
                    data: Some(data),
                };

                self.scope().push_var(*name, state)?;

                Ok(OutputData::None)
            },
            Stmt::CreateVar{span,var_type,name,data}=>{
                let data = if let Some(data) = data {
                    Some(self.interpret_expr(data)?)
                } else {
                    None
                };

                let state = VarState {
                    created_at: span.clone(),
                    last_modified_at: span.clone(),
                    var_type: *var_type,
                    taken: None,
                    takeable: true,
                    data,
                };

                self.scope().push_var(*name, state)?;

                Ok(OutputData::None)
            },
            Stmt::SetVar{span,left,data}=>{
                assert!(left.len() > 0);

                let data = self.interpret_expr(data)?;

                let scope = self.scope_stack.last_mut().unwrap();
                let mut left_data = scope.get_mut_reassign(span.clone(), left[0])?;

                // iterate over each field in the path to get the list
                if left.len() >= 2 {
                    for name in left[1..(left.len() - 2)].iter() {
                        left_data = left_data.get_mut_mutate(span.clone(), *name)?;
                    }

                    left_data = left_data.get_mut_reassign(span.clone(), *left.last().unwrap())?;
                }

                *left_data = data;

                Ok(OutputData::None)
            },
            Stmt::If{span,conditions,default}=>{
                for (condition, block) in conditions {
                    match self.interpret_expr(condition)? {
                        Data::Bool(b)=>if b {
                            return self.interpret_block(block);
                        },
                        _=>return Err(Error::new(span.clone(), ErrorType::InvalidType)),
                    }
                }

                if let Some(block) = default {
                    return self.interpret_block(block);
                } else {
                    return Ok(OutputData::None);
                }

            },
            Stmt::WhileLoop{span,condition,body}=>{
                loop {
                    match self.interpret_expr(condition)? {
                        Data::Bool(b)=>if !b {break},
                        _=>return Err(Error::new(span.clone(), ErrorType::InvalidType)),
                    }

                    match self.interpret_block(body)? {
                        OutputData::Return(d)=>return Ok(OutputData::Return(d)),
                        OutputData::Break=>break,
                        // Continue and None are handled the same since at this level
                        _=>{},
                    }
                }

                return Ok(OutputData::None);
            },
            Stmt::Expression(_, expr)=>{
                self.interpret_expr(expr)?;

                Ok(OutputData::None)
            },
            Stmt::Return(_, maybe_expr)=>if let Some(expr) = maybe_expr {
                Ok(OutputData::Return(Some(self.interpret_expr(expr)?)))
            } else {
                Ok(OutputData::Return(None))
            },
            Stmt::Continue(_)=>Ok(OutputData::Continue),
            Stmt::Break(_)=>Ok(OutputData::Break),
            Stmt::Print(_, expr)=>{
                let data = self.interpret_expr(expr)?;

                match data {
                    Data::String(s)=>print!("{s}"),
                    Data::Bool(b)=>print!("{b}"),
                    Data::Integer(i)=>print!("{i}"),
                    Data::Float(f)=>print!("{f}"),
                    d=>print!("{d:?}"),
                }

                Ok(OutputData::None)
            },
            _=>todo!(),
        }
    }

    pub fn interpret_expr(&mut self, expr: &'a Expr)->Result<Data, Error> {
        match expr {
            Expr::Copy(s, sym)=>self.scope().copy_var(s.clone(),*sym),
            Expr::BinaryOp(s, op, sides)=>{
                use Data::*;
                use BinaryOp::*;

                let left = self.interpret_expr(&sides[0])?;
                let right = self.interpret_expr(&sides[1])?;
                let error = Error::binary(s.clone(), *op);

                match (left, right) {
                    (Integer(i1), Integer(i2))=>match op {
                        Add=>Ok(Integer(i1 + i2)),
                        Sub=>Ok(Integer(i1 - i2)),
                        Mul=>Ok(Integer(i1 * i2)),
                        Div=>Ok(Integer(i1 / i2)),
                        Mod=>Ok(Integer(i1 % i2)),
                        Equal=>Ok(Bool(i1 == i2)),
                        NotEqual=>Ok(Bool(i1 != i2)),
                        Greater=>Ok(Bool(i1 > i2)),
                        Less=>Ok(Bool(i1 < i2)),
                        GreaterEqual=>Ok(Bool(i1 >= i2)),
                        LessEqual=>Ok(Bool(i1 <= i2)),
                        LogicAnd|LogicOr=>Err(error),
                    },
                    (Float(f1), Float(f2))=>match op {
                        Add=>Ok(Float(f1 + f2)),
                        Sub=>Ok(Float(f1 - f2)),
                        Mul=>Ok(Float(f1 * f2)),
                        Div=>Ok(Float(f1 / f2)),
                        Mod=>Ok(Float(f1 % f2)),
                        Equal=>Ok(Bool(f1 == f2)),
                        NotEqual=>Ok(Bool(f1 != f2)),
                        Greater=>Ok(Bool(f1 > f2)),
                        Less=>Ok(Bool(f1 < f2)),
                        GreaterEqual=>Ok(Bool(f1 >= f2)),
                        LessEqual=>Ok(Bool(f1 <= f2)),
                        LogicAnd|LogicOr=>Err(error),
                    },
                    (String(s1), String(s2))=>match op {
                        Add=>Ok(String(s1 + &s2)),
                        Equal=>Ok(Bool(s1 == s2)),
                        NotEqual=>Ok(Bool(s1 != s2)),
                        Sub|
                            Mul|
                            Div|
                            Mod|
                            Greater|
                            Less|
                            GreaterEqual|
                            LessEqual|
                            LogicAnd|
                            LogicOr=>Err(error),
                    },
                    (Bool(b1), Bool(b2))=>match op {
                        LogicAnd=>Ok(Bool(b1 && b2)),
                        LogicOr=>Ok(Bool(b1 || b2)),
                        Equal=>Ok(Bool(b1 == b2)),
                        NotEqual=>Ok(Bool(b1 != b2)),
                        Add|
                            Sub|
                            Mul|
                            Div|
                            Mod|
                            Greater|
                            Less|
                            GreaterEqual|
                            LessEqual=>Err(error),
                    },
                    (FunctionPtr(b1), FunctionPtr(b2))=>match op {
                        Equal=>Ok(Bool(b1.0 == b2.0)),
                        NotEqual=>Ok(Bool(b1.0 != b2.0)),
                        Add|
                            Sub|
                            Mul|
                            Div|
                            Mod|
                            Greater|
                            Less|
                            GreaterEqual|
                            LessEqual|
                            LogicAnd|
                            LogicOr=>Err(error),
                    },
                    (List(mut l1), List(mut l2))=>match op {
                        Add=>{
                            l1.append(&mut l2);

                            Ok(List(l1))
                        },
                        Equal|
                            NotEqual|
                            Sub|
                            Mul|
                            Div|
                            Mod|
                            Greater|
                            Less|
                            GreaterEqual|
                            LessEqual|
                            LogicAnd|
                            LogicOr=>Err(error),
                    },
                    _=>Err(error),
                }
            },
            Expr::UnaryOp(s, op, right)=>{
                use Data::*;
                use UnaryOp::*;

                let error = Error::unary(s.clone(), *op);
                let data = self.interpret_expr(right)?;
                match data {
                    Integer(i)=>match op {
                        Negate=>Ok(Integer(-i)),
                        Not=>Err(error),
                    },
                    Float(f)=>match op {
                        Negate=>Ok(Float(-f)),
                        Not=>Err(error),
                    },
                    Bool(b)=>match op {
                        Not=>Ok(Bool(!b)),
                        Negate=>Err(error),
                    },
                    _=>Err(error),
                }
            },
            Expr::Integer(_, i)=>Ok(Data::Integer(*i)),
            Expr::Float(_, f)=>Ok(Data::Float(*f)),
            Expr::String(_, string)=>Ok(Data::String(string.clone())),
            Expr::Named(s, sym)=>self.scope().take_var(s.clone(), *sym),
            Expr::Field(s, left, sym)=>{
                let left = self.interpret_expr(left)?;

                match left {
                    Data::Object(mut fields)|Data::Class(Class{inner:mut fields,..})=>{
                        if let Some((_,data)) = fields.remove(sym) {
                            Ok(data)
                        } else {
                            Err(Error::new(s.clone(), ErrorType::NoField(*sym)))
                        }
                    },
                    _=>Err(Error::new(s.clone(), ErrorType::NoField(*sym))),
                }
            },
            Expr::Call(s, items)=>{
                let left = self
                    .interpret_expr(&items[0])?
                    ;

                match left {
                    Data::FunctionPtr(ptr)=>{
                        let mut args = Vec::new();

                        for expr in &items[1..] {
                            args.push(self.interpret_expr(expr)?);
                        }

                        self.interpret_function_call(s.clone(), ptr, args)
                    },
                    _=>Err(Error::new(s.clone(), ErrorType::CannotCall)),
                }
            },
            Expr::Bool(_, b)=>Ok(Data::Bool(*b)),
            // Expr::Ref(s, ty, sym)=>{
            //     todo!();
            // },
            Expr::List(_, items)=>{
                let mut list = Vec::with_capacity(items.len());

                for item in items {
                    list.push(self.interpret_expr(item)?);
                }

                Ok(Data::List(list))
            },
            Expr::Index(s, sides)=>{
                let left = self.interpret_expr(&sides[0])?;

                match left {
                    Data::List(mut items)=>{
                        match self.interpret_expr(&sides[1])? {
                            Data::Integer(i)=>{
                                if i < 0 {
                                    Err(Error::new(s.clone(), ErrorType::CannotIndex))
                                } else {
                                    let i = i as usize;
                                    if i < items.len() {
                                        Ok(items.remove(i))
                                    } else {
                                        Err(Error::new(s.clone(), ErrorType::ArrayOutOfBounds))
                                    }
                                }
                            },
                            _=>Err(Error::new(s.clone(), ErrorType::InvalidIndexType)),
                        }
                    },
                    _=>Err(Error::new(s.clone(), ErrorType::CannotIndex)),
                }
            },
            _=>todo!(),
        }
    }

    fn interpret_function_call(&mut self, span: Span, id: FunctionId, args: Vec<Data>)->Result<Data, Error> {
        // the most important part: create a new scope stack for this function
        self.push_scope_stack();

        // get a ref to the function
        let func = self.functions.get(&id).unwrap();

        // check if there are the correct amount of args
        if args.len() != func.params.len() {
            return Err(Error::new(span, ErrorType::InvalidFunctionArgs(func.params.len(), args.len())));
        }

        // get a ref to the scope
        let scope = self.scope_stack.last_mut().unwrap();

        // add the inner functions
        for (sym, id) in func.inner_functions.iter() {
            let func = self.functions.get(&id).unwrap();
            let span = func.created_at.clone();
            scope.push_var(*sym, VarState {
                created_at: span.clone(),
                last_modified_at: span,
                var_type: VarType::empty(),
                taken: None,
                takeable: false,
                data: Some(Data::FunctionPtr(*id)),
            })?;
        }

        // add this function's name for recursion, if it doesn't exist yet.
        if !scope.has_var(func.name) {
            scope.push_var(func.name, VarState {
                created_at: span.clone(),
                last_modified_at: span,
                var_type: VarType::empty(),
                taken: None,
                takeable: false,
                data: Some(Data::FunctionPtr(id)),
            })?;
        }

        // allow the parameters to shadow defined functions
        scope.push_scope();

        // add the parameters
        for ((span, var_type, name), arg) in func.params.iter().zip(args) {
            scope.push_var(*name, VarState {
                created_at: span.clone(),
                last_modified_at: span.clone(),
                var_type: *var_type,
                taken: None,
                takeable: true,
                data: Some(arg),
            })?;
        }

        // get the data returned from the block
        let data = match self.interpret_stmt_list(func.body)? {
            OutputData::Return(d)=>d.unwrap_or(Data::None),
            _=>Data::None,
        };

        // remove this function's scope stack
        self.pop_scope_stack();

        return Ok(data);
    }
}

/// TODO: references
pub struct ScopeStack {
    vars: FnvHashMap<Symbol, Vec<VarState>>,
    scopes: Vec<Scope>,
}
impl ScopeStack {
    pub fn new()->Self {
        ScopeStack {
            vars: FnvHashMap::default(),
            scopes: vec![Scope::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn drop_scope(&mut self) {
        if let Some(Scope{vars}) = self.scopes.pop() {
            for var in vars {
                self.vars
                    .get_mut(&var)
                    .unwrap()
                    .pop();
            }
        }
    }

    pub fn has_var(&self, sym: Symbol)->bool {
        self.vars.contains_key(&sym)
    }

    pub fn push_var(&mut self, sym: Symbol, state: VarState)->Result<(), Error> {
        let scope = self.scopes.last_mut().unwrap();

        // test if the scope has the var already
        if scope.has_sym(sym) {
            return Err(Error::new(state.created_at, ErrorType::VarExistsInScope));
        }

        // add the var to the scope and storage
        scope.add(sym);
        let states = self.vars
            .entry(sym)
            .or_default();

        if let Some(last) = states.last_mut() {
            if last.taken.is_some() {
                *last = state;
            } else {
                states.push(state);
            }
        } else {
            states.push(state);
        }

        return Ok(());
    }

    pub fn take_var(&mut self, span: Span, sym: Symbol)->Result<Data, Error> {
        self.remove_undefined(sym);

        if let Some(states) = self.vars.get_mut(&sym) {
            let last = states
                .last_mut()
                .unwrap();

            if !last.takeable {
                return Ok(last.data.clone().unwrap());
            }

            if let Some(data) = &last.data {
                // return early if the data can be copied
                if data.is_copy() {
                    return Ok(data.clone());
                }

                let data = last.data.take().unwrap();

                // mark the var as taken so we can clean it up later, if needed
                last.taken = Some(span.clone());
                last.last_modified_at = span;

                // if the var has no reassign privilege, then fully remove it.
                if !last.var_type.contains(VarType::REASSIGN) {
                    self.remove_undefined(sym);
                }

                return Ok(data);
            } else {
                return Err(Error::new(span, ErrorType::VarUndefined));
            }
        }

        return Err(Error::new(span, ErrorType::VarDoesNotExist));
    }

    pub fn get_mut_mutate(&mut self, span: Span, sym: Symbol)->Result<&mut Data, Error> {
        if let Some(states) = self.vars.get_mut(&sym) {
            let state = states.last_mut().unwrap();
            if state.data.is_none() || state.var_type.contains(VarType::MUTATE) {
                state.taken = None;
                state.last_modified_at = span;

                return Ok(state.data.as_mut().unwrap());
            } else {
                return Err(Error::new(span, ErrorType::CannotReassign));
            }
        }

        return Err(Error::new(span, ErrorType::VarDoesNotExist));
    }

    /// Data MUST be assigned to the reference to maintain the proper internal state.
    pub fn get_mut_reassign(&mut self, span: Span, sym: Symbol)->Result<&mut Data, Error> {
        if let Some(states) = self.vars.get_mut(&sym) {
            let state = states.last_mut().unwrap();
            if state.data.is_none() || state.var_type.contains(VarType::REASSIGN) {
                state.taken = None;
                state.last_modified_at = span;
                if state.data.is_none() {
                    state.data = Some(Data::None);
                }

                return Ok(state.data.as_mut().unwrap());
            } else {
                return Err(Error::new(span, ErrorType::CannotReassign));
            }
        }

        return Err(Error::new(span, ErrorType::VarDoesNotExist));
    }

    pub fn assign_var(&mut self, span: Span, sym: Symbol, data: Data)->Result<(), Error> {
        let mutable_data = self.get_mut_reassign(span, sym)?;
        *mutable_data = data;

        return Ok(());
    }

    /// TODO: verify the var can actually be copied
    pub fn copy_var(&mut self, span: Span, sym: Symbol)->Result<Data, Error> {
        self.remove_undefined(sym);

        if let Some(states) = self.vars.get_mut(&sym) {
            if let Some(data) = &states.last().unwrap().data {
                return Ok(data.clone());
            } else {
                return Err(Error::new(span, ErrorType::VarUndefined));
            }
        }

        return Err(Error::new(span, ErrorType::VarDoesNotExist));
    }

    fn remove_undefined(&mut self, sym: Symbol) {
        if let Some(states) = self.vars.get_mut(&sym) {
            // early escape to avoid looping
            if states.last().unwrap().taken.is_none() {
                return;
            }

            for scope in self.scopes.iter_mut().rev() {
                if scope.has_sym(sym) {
                    let last = states.last().unwrap();
                    if last.taken.is_some() {
                        states.pop();
                        scope.remove(sym);
                    } else {
                        return;
                    }
                }
            }

            self.vars.remove(&sym);
        }
    }
}

pub struct Scope {
    /// contains variables and functions
    vars: FnvHashSet<Symbol>,
}
impl Scope {
    pub fn new()->Self {
        Scope {
            vars: FnvHashSet::default(),
        }
    }

    pub fn add(&mut self, sym: Symbol) {
        self.vars.insert(sym);
    }

    pub fn remove(&mut self, sym: Symbol) {
        self.vars.remove(&sym);
    }

    pub fn has_sym(&self, sym: Symbol)->bool {
        self.vars.contains(&sym)
    }
}

#[derive(Debug)]
pub struct VarState {
    created_at: Span,
    last_modified_at: Span,
    var_type: VarType,
    taken: Option<Span>,
    takeable: bool,
    data: Option<Data>,
}

#[derive(Debug, Clone)]
pub struct Class {
    name: Symbol,
    inner: Object,
}

pub struct RTFunction<'a> {
    name: Symbol,
    created_at: Span,
    inner_functions: FnvHashMap<Symbol, FunctionId>,
    params: &'a [(Span, VarType, Symbol)],
    body: &'a [Stmt],
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId(usize);
