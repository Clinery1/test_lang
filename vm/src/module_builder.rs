use num_enum::{IntoPrimitive,FromPrimitive};
use test_lang_common::{
    error::*,
    Span,
};
use super::{
    bytecode::*,
    *,
};
use ConstantId as CID;
use Instruction as I;


pub const U24_MAX: usize = 0xffffff;


pub enum ConstantId {
    One(u8),
    Two(u16),
    Three(u32),
}


/// A builder that is used to build [`Module`]s. Has a `push_NAME` method for each instruction.
/// Just like other structs that implement the builder pattern, we require a call to `finish`, and
/// return `&mut Self` where possible for method chaining.
pub struct ModuleBuilder {
    code: Vec<u8>,
    constants: Vec<Constant>,
    /// a list of spans indexing both the bytecode and source code
    spans: Vec<BytecodeSpan>,
    /// the span that indexes the source code
    current_source_span: Span,
    /// the offset into the bytecode buffer
    current_code_span_start: usize,
}
impl ModuleBuilder {
    pub fn new(start_span: Span)->Self {
        ModuleBuilder {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
            current_source_span: start_span,
            current_code_span_start: 0,
        }
    }

    #[inline]
    fn ins(&mut self, ins: Instruction) {
        self.code.push(ins.into());
    }

    #[inline]
    fn bytes<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        self.code.extend(iter);
    }

    #[inline]
    fn byte(&mut self, byte: u8) {
        self.code.push(byte);
    }

    pub fn set_span(&mut self, new_span: Span)->&mut Self {
        let end = self.code.len();

        self.spans.push(BytecodeSpan {
            source_span: self.current_source_span.clone(),
            instruction_span: self.current_code_span_start..end,
        });

        self.current_source_span = new_span;
        self.current_code_span_start = end;

        return self;
    }

    pub fn register_constant(&mut self, constant: Constant)->ConstantId {
        let const_count = self.constants.len();

        let id = if const_count <= (u8::MAX as usize) {
            ConstantId::One(const_count as u8)
        } else if const_count < (u16::MAX as usize) {
            ConstantId::Two(const_count as u16)
        } else if const_count < U24_MAX {
            ConstantId::Three(const_count as u32)
        } else {
            panic!("Maximum of {U24_MAX} constants reached!");
        };

        self.constants.push(constant);

        return id;
    }

    pub fn push_const(&mut self, id: ConstantId)->&mut Self {
        match id {
            CID::One(n)=>{
                self.ins(I::Constant);
                self.byte(n);
            },
            CID::Two(n)=>{
                self.ins(I::Constant2);
                self.bytes(n.to_le_bytes());
            },
            CID::Three(n)=>{
                self.ins(I::Constant3);
                self.bytes(n.to_le_bytes());
            },
        }

        return self;
    }

    pub fn push_ret(&mut self)->&mut Self {
        self.ins(I::Return);
        return self;
    }

    pub fn push_ret_val(&mut self, val: ())->&mut Self {
        self.ins(I::ReturnValue);
        // TODO: return value

        return self;
    }

    pub fn push_call(&mut self, arg_count: u8)->&mut Self {
        self.ins(I::Call);
        self.byte(arg_count);

        return self;
    }

    pub fn push_nop(&mut self)->&mut Self {
        self.ins(I::Nop);

        return self;
    }
}
