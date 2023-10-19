use crate::{
    bytecode::*,
    Module,
};
use Instruction as I;


pub trait Disassemble {
    fn disassemble(&self);
}
impl<'a> Disassemble for Module<'a> {
    fn disassemble(&self) {
        let mut ip = 0;
        let mut bytecode_span = &self.spans[0];
        let mut next_bytecode_span = 1;

        while ip < self.code.len() {
            let print_bytecode_span = ip >= bytecode_span.instruction_span.end;
            if print_bytecode_span {
                bytecode_span = &self.spans[next_bytecode_span];
                next_bytecode_span += 1;
                print!("{:<9?}| ", bytecode_span.source_span);
            } else {
                print!("          | ");
            }

            print!("{ip}");

            let opcode = I::from(self.code[ip]);
            ip += 1;

            match opcode {
                I::Nop=>println!("nop"),
                I::Return=>println!("ret"),
                I::ReturnValue=>{
                    // TODO: values
                    println!("retVal");
                },
                I::Call=>{
                    let count = self.code[ip];
                    ip += 1;

                    println!("call      {count}");
                },
                I::Constant=>{
                    let constant = self.read_const1(&mut ip);

                    println!("const     {constant:?}");
                },
                I::Constant2=>{
                    let constant = self.read_const2(&mut ip);

                    println!("const     {constant:?}");
                },
                I::Constant3=>{
                    let constant = self.read_const3(&mut ip);

                    println!("const     {constant:?}");
                },
            }
        }
    }
}
