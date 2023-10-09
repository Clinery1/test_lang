//! TODO:
//!     - COMMENT THE CODE!
//!     - proper error handling
//!     - finish parser
//!     - tree-walking interpreter
//!     - parse comments
//!     - static analysis
//!     - types


// #![allow(dead_code)]


use std::fs::read_to_string;
use parser::Parser;


mod error;
mod lexer;
mod ast;
mod parser;


fn main() {
    let data = read_to_string("example").unwrap();

    let mut parser = Parser::new(&data);
    let res = parser.parse_file();
    for (sym, name) in parser.lexer.extras.into_iter() {
        println!("{:?} = {}", sym, name);
    }
    println!();
    match res {
        Ok(stmts)=>for stmt in stmts {
            println!("{:#?}", stmt);
        },
        Err(e)=>e.print(&data),
    }
}
