#![allow(warnings)]
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod lexer;
mod parser;
mod tokens;
mod dijkstra;
mod delay;

#[derive(Debug)]
pub struct Process {
    id: String,
    input: Vec<(String, u64)>,
    output: Vec<(String, u64)>,
    time: u64,
}

#[derive(Debug)]
pub struct Data {
    pub stocks: HashMap<String, u64>,
    pub processes: Vec<Process>,
    pub objectives: Vec<String>,
}

fn main() {
    let mut parser = parser::Parser::new("test.txt");
    match parser.parse() {
        Err(err) => {
            eprintln!("\n\nERROR !!!!!: {:?}", err);
            return;
        }
        _ => {}
    };
    let x = Data {
        stocks: parser.stocks,
        processes: parser.process,
        objectives: parser.optimize.unwrap(),
    };
    println!("stocks: {:?}\n", x.stocks);
    println!("processes:");
    for p in &x.processes {
        println!("{:?}", p);
    }
    println!();
    println!("objectives: {:?}\n", x.objectives);

    /* 10 will be the delay. */
    let delay = 180;
    if let Some((time, final_stocks)) = dijkstra::optimize(x, delay) {
        println!("Optimized in {} units of time with stocks: {:?}", time, final_stocks);
    } else {
        println!("No solution found");
    }
}
