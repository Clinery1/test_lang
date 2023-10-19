//! TODO:
//!     - COMMENT THE CODE!
//!     - proper error handling
//!     - public/private class members, classes, interfaces, functions
//!     - reference counting
//!     - finish parser
//!     - tree-walking interpreter
//!     - a proper REPL and compiler that takes CLI inputs
//!     - parse comments
//!     - static analysis
//!     - types


use std::{
    time::{
        Instant,
        Duration,
    },
    hint::black_box,
    fs::read_to_string,
};
use parser::Parser;
pub use test_lang_common::{
    error,
    Span,
};


mod lexer;
mod ast;
mod parser;
mod static_analysis;

fn main() {
    test_expr_parser();

    // Test the `parse_example` file, and report errors. If this succeeds, then we can benchmark
    // the parser
    test_parser();

    // I am leaving this here so we always have a performance metric to let us know if something is
    // wrong, like if I introduce an exponential time function instead of a linear time function in
    // the parser (unlikely).
    benchmark_parser(200);

    let data = read_to_string("example").unwrap();

    let (mut parser, _this_sym) = Parser::new(&data);
    let res = parser.parse_file();
    // for (sym, name) in parser.lexer.extras.into_iter() {
    //     println!("{:?} = {}", sym, name);
    // }
    // println!();
    match res {
        Ok(_stmts)=>{
            let error = parser.non_fatal_errors.len() > 0;
            for err in parser.non_fatal_errors.drain(..) {
                err.print(&data);
            }
            if error {
                return;
            }

            // for stmt in stmts.iter() {
            //     println!("{:#?}", stmt);
            // }

            // println!("Running code...");
            // let start = Instant::now();
            // let elapsed = start.elapsed();
            // match out {
            //     Ok(d)=>{
            //         println!("Code output: {:?}", d);
            //         println!("Execution took {:?}", elapsed);
            //     },
            //     Err(e)=>e.print(&data),
            // }
        },
        Err(e)=>e.print(&data),
    }
}

fn test_expr_parser() {
    let source = read_to_string("expr_test").unwrap();

    let (mut parser, _) = Parser::new(&source);
    let mut expr_parser = parser::expr::ExprParser::new(&mut parser);
    match expr_parser.parse() {
        Ok(e)=>println!("{:#}", e),
        Err(e)=>e.print(&source),
    }
}

/// Test the parser with the `parse_example` file. If it fails, then we have a regression. For now,
/// this is always ran at startup. Once I finish the tree-walking interpreter, I will start making
/// the executable more production-ready.
fn test_parser() {
    let source = read_to_string("parse_example").unwrap();

    let (mut parser, _) = Parser::new(&source);
    match parser.parse_file() {
        Err(e)=>{
            e.print(&source);
            panic!("Parse failed!");
        },
        _=>{},
    }
}

/// Benchmarks the parser over `count` iterations and averages the time and MB/s
#[allow(dead_code)]
fn benchmark_parser(count: usize) {
    let raw_source = read_to_string("parse_example").unwrap();
    let mut source = String::new();

    // store 4 times the data for a better average
    source.push_str(&raw_source);
    source.push_str(&raw_source);
    source.push_str(&raw_source);
    source.push_str(&raw_source);

    // parse the code `count` times and sum the times
    let sum_times = (0..count)
        .map(|_|{
            let (mut parser, _) = Parser::new(&source);
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
    println!(
        "Total parse time: {:?} for {} iterations; Average parse time: {:?} @ {:.2} MB/s",
        Duration::from_secs_f64(sum_times),
        count,
        Duration::from_secs_f64(average_time),
        mb_per_sec,
    );
}
