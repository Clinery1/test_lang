//! TODO:
//!     - COMMENT THE CODE!
//!     - proper error handling
//!     - finish parser
//!     - tree-walking interpreter
//!     - a proper REPL and compiler that takes CLI inputs
//!     - parse comments
//!     - static analysis
//!     - types


// #![allow(dead_code)]


use std::{
    time::{
        Instant,
        Duration,
    },
    hint::black_box,
    fs::read_to_string,
};
use parser::Parser;


mod error;
mod lexer;
mod ast;
mod parser;
mod tree_walk;

fn main() {
    let data = read_to_string("example").unwrap();

    let mut parser = Parser::new(&data);
    let res = parser.parse_file();
    // for (sym, name) in parser.lexer.extras.into_iter() {
    //     println!("{:?} = {}", sym, name);
    // }
    // println!();
    match res {
        Ok(stmts)=>{
            // for stmt in stmts {
            //     println!("{:#?}", stmt);
            // }
            
            let mut interpreter = tree_walk::Interpreter::new();

            println!("Running code...");
            let start = Instant::now();
            let out = interpreter.interpret_program(&stmts);
            let elapsed = start.elapsed();
            match out {
                Ok(d)=>{
                    println!("Code output: {:?}", d);
                    println!("Execution took {:?}", elapsed);
                },
                Err(e)=>e.print(&data),
            }
        },
        Err(e)=>e.print(&data),
    }

    // I am leaving this here so we always have a performance metric to let us know if something is
    // wrong, like if I introduce an exponential time function instead of a linear time function in
    // the parser (unlikely).
    benchmark_parser(200);
}

#[allow(dead_code)]
fn benchmark_parser(count: usize) {
    let source = read_to_string("example").unwrap();

    // parse the code `count` times and sum the times
    let sum_times = (0..count)
        .map(|_|{
            let mut parser = Parser::new(&source);
            let start = Instant::now();
            let _parsed = black_box(parser.parse_file().unwrap());
            let elapsed = start.elapsed();

            elapsed.as_secs_f64()
        })
        .sum::<f64>();

    // calculate average
    let average_time = sum_times / (count as f64);

    // calculate bytes/sec and MB/s
    let bytes_per_sec = (source.len() as f64) / average_time;
    let mb_per_sec = bytes_per_sec / (1024.0*1024.0);

    // log the data
    println!("Average parse time: {:?}", Duration::from_secs_f64(average_time));
    println!("{:.2} MB/s", mb_per_sec);
}
