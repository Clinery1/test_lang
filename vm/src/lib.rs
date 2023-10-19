use num_enum::FromPrimitive;
use bytecode::*;
use test_lang_common::{
    error::*,
    Span,
};
use Instruction as I;


pub mod bytecode;
pub mod module_builder;
pub mod debug;


#[derive(Debug)]
pub enum Constant {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    ModuleId(usize),
}

pub enum ModuleReturn {
    // TODO: values
    Data,
    Call {
        call_id: ModuleId,
        ip: usize,
        args: Vec<()>,
    },
    Done,
}

#[derive(Clone)]
pub enum CallItem {
    Current(ModuleId),
    Suspended {
        module: ModuleId,
        ip: usize,
    },
    Start {
        module: ModuleId,
        args: Vec<()>,
    },
}
impl CallItem {
    pub fn mod_id(&self)->ModuleId {
        match self {
            Self::Current(id)|
                Self::Suspended{module:id,..}|
                Self::Start{module:id,..}=>*id,
        }
    }

    pub fn suspend(&mut self, new_ip: usize) {
        match self {
            Self::Current(module)=>*self = Self::Suspended {module: *module, ip: new_ip},
            Self::Suspended{ip,..}=>*ip = new_ip,
            Self::Start{module,..}=>*self = Self::Suspended {module: *module, ip: new_ip},
        }
    }

    pub fn resume(&mut self)->usize {
        match self {
            Self::Current(_)=>0,
            Self::Suspended{ip,module}=>{
                let ip = *ip;
                *self = Self::Current(*module);

                return ip;
            },
            Self::Start{..}=>0,
        }
    }

    pub fn args(&mut self)->Option<Vec<()>> {
        match self {
            Self::Start{module,..}=>{
                let new_self = Self::Current(*module);
                let old_self = std::mem::replace(self, new_self);
                let Self::Start{args,..} = old_self else {unreachable!()};

                return Some(args);
            },
            _=>None,
        }
    }
}


pub struct Program<'a> {
    modules: Vec<Module<'a>>,
    global_module: ModuleId,
}
impl<'a> Program<'a> {
    pub fn run(&mut self)->Result<(), Error> {
        let mut call_stack = vec![CallItem::Current(self.global_module)];
        while let Some(mut item) = call_stack.pop()  {
            let ret;
            if let Some(args) = item.args() {
                ret = self
                    .modules[item.mod_id().0]
                    .start(args)?;
            } else {
                ret = self
                    .modules[item.mod_id().0]
                    .run(item.resume())?;
            }

            match ret {
                // We have already done everything required to exit the scope
                ModuleReturn::Done=>{},
                // Suspend the current function and push the next one
                ModuleReturn::Call{call_id,ip,args}=>{
                    item.suspend(ip);
                    call_stack.push(item);
                    call_stack.push(CallItem::Start{module:call_id,args});
                },
                // TODO: function returns
                ModuleReturn::Data=>todo!(),
            }
        }
        return Ok(());
    }
}

pub struct Module<'a> {
    id: ModuleId,
    name: &'a str,
    code: Vec<u8>,
    constants: Vec<Constant>,
    spans: Vec<BytecodeSpan>,
}
impl<'a> Module<'a> {
    pub fn start(&self, _args: Vec<()>)->Result<ModuleReturn, Error> {
        // TODO: arguments
        return self.run(0);
    }
    /// Run with an optional `ip` parameter used to resume the module
    pub fn run(&self, mut ip: usize)->Result<ModuleReturn, Error> {

        while ip < self.code.len() {
            let ins_byte = self.code[ip];
            ip += 1;

            match Instruction::from_primitive(ins_byte) {
                I::Nop=>{},
                I::Return=>{
                    todo!();
                },
                I::ReturnValue=>{
                    todo!();
                },
                I::Call=>{
                    todo!();
                },
                I::Constant=>{
                    let num = self.code[ip];
                    ip += 1;

                    let constant = &self.constants[num as usize];

                    println!("Constant: {constant:?}");
                },
                I::Constant2=>{
                    let num = self.code[ip];
                    ip += 1;
                    let num1 = self.code[ip];
                    ip += 1;

                    let num = u16::from_le_bytes([num,num1]);

                    let constant = &self.constants[num as usize];

                    println!("Constant: {constant:?}");
                },
                I::Constant3=>{
                    let num = self.code[ip];
                    ip += 1;
                    let num1 = self.code[ip];
                    ip += 1;
                    let num2 = self.code[ip];
                    ip += 1;

                    let num = u32::from_le_bytes([num,num1,num2,0]);

                    let constant = &self.constants[num as usize];

                    println!("Constant: {constant:?}");
                },
            }
        }

        return Ok(ModuleReturn::Done);
    }

    pub fn read_const1(&self, ip: &mut usize)->&Constant {
        let num = self.code[*ip];
        *ip += 1;

        &self.constants[num as usize]
    }

    pub fn read_const2(&self, ip: &mut usize)->&Constant {
        let num = self.code[*ip];
        *ip += 1;
        let num1 = self.code[*ip];
        *ip += 1;

        let num = u16::from_le_bytes([num,num1]);

        &self.constants[num as usize]
    }

    pub fn read_const3(&self, ip: &mut usize)->&Constant {
        let num = self.code[*ip];
        *ip += 1;
        let num1 = self.code[*ip];
        *ip += 1;
        let num2 = self.code[*ip];
        *ip += 1;

        let num = u32::from_le_bytes([num,num1,num2,0]);

        &self.constants[num as usize]
    }
}

pub struct BytecodeSpan {
    pub instruction_span: Span,
    pub source_span: Span,
}
#[allow(dead_code)]
impl BytecodeSpan {
    pub fn new(instruction_span: Span, source_span: Span)->Self {
        Self {
            instruction_span,
            source_span,
        }
    }

    pub fn is_ip_inside(&self, ip: usize)->bool {
        self.instruction_span.contains(&ip)
    }

    pub fn try_get_span(&self, ip: usize)->Option<Span> {
        if self.instruction_span.contains(&ip) {
            Some(self.source_span.clone())
        } else {
            None
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ModuleId(pub usize);
