use num_enum::{IntoPrimitive,FromPrimitive};


#[repr(u8)]
#[derive(IntoPrimitive, FromPrimitive, Default, Copy, Clone, Debug)]
pub enum Instruction {
    #[default]
    Nop,

    // function-related
    /// Returns from a function
    Return,
    /// Returns a value from a function
    ReturnValue,
    /// Calls a function. Reads the next byte to specify how many arguments the function has.
    Call,

    // data loading
    /// Reads the next byte as an index into the constant list
    Constant,
    /// Reads the next 2 bytes as an index into the constant list
    Constant2,
    /// Reads the next 3 bytes as an index into the constant list
    Constant3,
}
