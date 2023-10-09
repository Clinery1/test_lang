//! TODO:
//!     - COMMENT THE CODE!
//!     - proper error handling
//!     - finish parser
//!     - tree-walking interpreter
//!     - parse comments
//!     - static analysis
//!     - types


#![allow(dead_code)]


use std::fs::read_to_string;
use parser::Parser;


mod error;
mod lexer;
mod ast;
mod parser;


fn main() {
    let data = read_to_string("example").unwrap();

    let mut parser = Parser::new(&data);
    let expr = parser.parse_expr();
    for (sym, name) in parser.lexer.extras.into_iter() {
        println!("{:?} = {}", sym, name);
    }
    println!();
    match expr {
        Ok(e)=>println!("{:#?}\n{0:#}", e),
        Err(e)=>e.print(&data),
    }
}
