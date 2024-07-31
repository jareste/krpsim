#![allow(warnings)]
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::env;

mod lexer;
mod forbidden_name;
mod parser;
mod tokens;
mod dijkstra;
mod delay;

#[derive(Debug, Clone)]
pub struct Process {
    id: String,
    input: Vec<(String, u64)>,
    output: Vec<(String, u64)>,
    time: u64,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub stocks: HashMap<String, u64>,
    pub processes: Vec<Process>,
    pub objectives: Vec<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path to file>", args[0]);
        return;
    }
    
    let file = args[1].clone();
    let file_static: &'static str = Box::leak(file.into_boxed_str());

    let mut parser = parser::Parser::new(file_static);
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

    let delay = 1;

    println!("\nOptimizing with Tabu Search...\n");

    let (best_solution, best_time) = forbidden_name::tabu_search(&x, usize::MAX, usize::MAX, delay);

    println!("Optimized in {} units of time with stocks: {:?}\n", best_time, best_solution.stocks);

    /* 10 will be the delay. */
    println!("\nOptimizing with Dijkstra's algorithm...\n");
    if let Some((time, final_stocks)) = dijkstra::optimize(x, delay) {
        println!("Optimized in {} units of time with stocks: {:?}", time, final_stocks);
    } else {
        println!("No solution found");
    }


}
