#![allow(warnings)]
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::env;
use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;
use std::thread::JoinHandle;

mod lexer;
mod forbidden_name;
mod parser;
mod tokens;
mod dijkstra;
mod genetic;
mod simmulated_annealing;
mod delay;
mod aco;
mod gen_file;
mod stock_scores;
mod a_star;
mod ida_star;
mod sgs;

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

fn get_args() -> (String, u32, Vec<String>) {
    let matches = Command::new("my_cli_app")
        .version("1.0")
        .about("Executes various algorithms on the provided file")
        .arg(
            Arg::new("file")
                .help("Path to the file")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("delay")
                .help("Delay in seconds")
                .required(true)
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("algorithms")
                .help("Algorithms to execute")
                .required(false)
                .action(ArgAction::Append)
                .value_parser(["dijkstra", "aco", "tabu", "ga", "sa", "a*", "ida*", "sgs", "all"])
                .ignore_case(true),
        )
        .get_matches();

    let file: PathBuf = matches.get_one::<PathBuf>("file").unwrap().clone();

    let delay: u32 = *matches.get_one::<u32>("delay").unwrap_or(&3);

    let algorithms: Vec<String> = matches
        .get_many::<String>("algorithms")
        .map(|vals| vals.map(|v| v.to_string()).collect())
        .unwrap_or_else(Vec::new);

    (file.to_string_lossy().to_string(), delay, algorithms)
}

fn main() {

    let (file, delay, mut algorithms) = get_args();
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    /* PARSING */
    let file_static: &'static str = Box::leak(file.clone().into_boxed_str());

    let mut parser = parser::Parser::new(file_static);
    match parser.parse() {
        Err(err) => {
            eprintln!("\n\nFATAL ERROR !!!!!: {:?}", err);
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
    /**********************/

    if algorithms.contains(&"all".to_string()) || algorithms.is_empty() {
        algorithms = vec![
            "dijkstra".to_string(),
            "aco".to_string(),
            "tabu".to_string(),
            "ga".to_string(),
            "sa".to_string(),
            "a*".to_string(),
            "ida*".to_string(),
            "sgs".to_string(),
        ];
    }

    for algorithm in algorithms {
        match algorithm.to_lowercase().as_str() {
            "dijkstra" => {
                /* DIJKSTRA ALGO */
                println!("\x1b[36m\nOptimizing with Dijkstra's algorithm...\n\x1b[0m");
                if let Some((time, final_stocks, best_log)) = dijkstra::optimize(x.clone(), delay) {
                    println!("Optimized in {} units of time with stocks: {:?}\n", time, final_stocks);
                    handles.push(gen_file::run_in_thread("logs/dijkstra_log.txt".to_string(), final_stocks.clone(), best_log.clone(), time));
                } else {
                    println!("No solution found");
                }
                /**********************/
            },
            "aco" => {
                /* ACO ALGO */
                println!("\x1b[36m\nOptimizing with Ant Colony Optimitzation ...\n\x1b[0m");

                let (best_solution, best_time, best_stocks, best_log) = aco::aco_optimization(&x, usize::MAX, 10000, delay);
                println!("Optimized in {:?} units of time with stocks: {:?}\n", best_time, best_stocks);
                handles.push(gen_file::run_in_thread("logs/aco_log.txt".to_string(),  best_stocks.clone(), best_log.clone(), best_time));
                /**********************/
            },
            "tabu" => {
                /* TABU SEARCH ALGO */ 
                println!("\x1b[36m\nOptimizing with Tabu Search...\n\x1b[0m");

                let (best_solution, best_time, best_log) = forbidden_name::tabu_search(&x, usize::MAX, 1000, delay);
                println!("Optimized in {} units of time with stocks: {:?}\n", best_time, best_solution.stocks);
                handles.push(gen_file::run_in_thread("logs/tabu_log.txt".to_string(), best_solution.stocks.clone(), best_log.clone(), best_time));
                /**********************/
            },
            "ga" => {
                /* GENETIC ALGO */
                println!("\x1b[36m\nOptimizing with Genetic Algorithm...\n\x1b[0m");
                let Some((best_time, best_stocks, best_log)) = genetic::genetic_algorithm(x.clone(), delay) else {
                    println!("No solution found");
                    continue;
                };
                println!("Optimized in {} units of time with stocks: {:?}\n", best_time, best_stocks);
                handles.push(gen_file::run_in_thread("logs/genetic_log.txt".to_string(), best_stocks.clone(), best_log.clone(), best_time));
                /**********************/
            },
            "sa" => {
                /* SA_STAR ALGO */
                println!("\x1b[36m\nOptimizing with Simmulated Annealing algorithm...\n\x1b[0m");
                let (best_state, best_time, best_log) = simmulated_annealing::simulated_annealing(&x, 500000.0, 1.0, 0.98, delay as u64);
                println!("Optimized in {} units of time with stocks: {:?}\n", best_time, best_state);
                /**********************/

            },
            "a*" => {
                /* A_STAR ALGO */
                println!("\x1b[36m\nOptimizing with A*'s algorithm...\n\x1b[0m");
                if let Some((time, final_stocks, best_log)) = a_star::optimize(x.clone(), delay) {
                    println!("Optimized in {} units of time with stocks: {:?}\n", time, final_stocks);
                    handles.push(gen_file::run_in_thread("logs/a_star_log.txt".to_string(), final_stocks.clone(), best_log.clone(), time));
                } else {
                    println!("No solution found");
                }
                /**********************/
            },
            "ida*" => {
                /* IDA_STAR ALGO */
                println!("\x1b[36m\nOptimizing with IDA*'s algorithm...\n\x1b[0m");
                if let Some((time, final_stocks, best_log)) = ida_star::optimize(x.clone(), delay) {
                    println!("Optimized in {} units of time with stocks: {:?}\n", time, final_stocks);
                    handles.push(gen_file::run_in_thread("logs/ida_star_log.txt".to_string(), final_stocks.clone(), best_log.clone(), time));
                } else {
                    println!("No solution found");
                }
                /**********************/
            },
            "sgs" => {
                /* SA_STAR ALGO */
                println!("\x1b[36m\nOptimizing with SGS algorithm...\n\x1b[0m");
                let (data, time, log) = sgs::sgs_algorithm(x.clone(), delay as u64);
                handles.push(gen_file::run_in_thread("logs/sgs_log.txt".to_string(), data.stocks.clone(), log.clone(), time));                
                println!("Optimized in {} units of time with stocks: {:?}\n", time, data.stocks);
                /**********************/

            },
            _ => println!("Unknown algorithm: {}", algorithm),
        }
    }


    for handle in handles {
        if let Err(e) = handle.join() {
            eprintln!("Thread panicked: {:?}", e);
        }
    }
}
