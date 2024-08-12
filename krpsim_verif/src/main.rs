#![allow(warnings)]
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::env;
use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;
use std::io::{self, BufRead};
use std::fs::File;

mod lexer;
mod parser;
mod tokens;

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

#[derive(Debug)]
pub struct Execution {
    pub time: u64,
    pub process_name: String,
}

fn get_args() -> (String, String) {
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
            Arg::new("result_to_test")
                .help("Path to the result to test")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();

    let file: PathBuf = matches.get_one::<PathBuf>("file").unwrap().clone();

    let result_to_test: PathBuf = matches.get_one::<PathBuf>("result_to_test").unwrap().clone();


    (file.to_string_lossy().to_string(), result_to_test.to_string_lossy().to_string())
}

pub fn parse_result_file(lines: io::Lines<io::BufReader<File>>) -> (Vec<Execution>, HashMap<String, u64>) {
    let mut executions = Vec::new();
    let mut final_stocks = HashMap::new();

    let mut parsing_final_stocks = false;

    for line in lines {
        match line {
            Ok(trimmed) => {
                let trimmed = trimmed.trim();

                if trimmed.starts_with("Final stocks:") {
                    parsing_final_stocks = true;
                    continue;
                }

                if parsing_final_stocks {
                    if !trimmed.is_empty() {
                        let parts: Vec<&str> = trimmed.split(':').collect();
                        if parts.len() == 2 {
                            let stock_name = parts[0].trim().to_string();
                            let stock_qty: u64 = parts[1].trim().parse().unwrap_or(0);
                            final_stocks.insert(stock_name, stock_qty);
                        }
                        else {
                            eprintln!("Invalid final stock line: {}", trimmed);
                        }
                    }
                } else if !trimmed.is_empty() && trimmed.contains(':') {
                    let parts: Vec<&str> = trimmed.split(':').collect();
                    if parts.len() == 2 {
                        let time: u64 = parts[0].trim().parse().unwrap_or(0);
                        let process_name = parts[1].trim().to_string();
                        executions.push(Execution { time, process_name });
                    }
                    else {
                        eprintln!("Invalid execution line: {}", trimmed);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error reading line: {:?}", err);
            }
        }
    }

    if (!parsing_final_stocks) {
        eprintln!("Final stocks not found in result file.");
    }

    (executions, final_stocks)
}

pub fn check_execution(data: &Data, executions: &Vec<Execution>, final_stocks: &HashMap<String, u64>) -> Result<(), String> {
    let mut current_stocks = data.stocks.clone();
    let mut buffered_stocks = HashMap::new();
    let mut previous_time = None;

    for execution in executions {

        if Some(execution.time) != previous_time {
            for (stock_name, qty) in buffered_stocks.drain() {
                *current_stocks.entry(stock_name).or_insert(0) += qty;
            }
            previous_time = Some(execution.time);
        }

        let process = data.processes.iter().find(|p| p.id == execution.process_name);
        if process.is_none() {
            return Err(format!("Process '{}' not found at time {}.", execution.process_name, execution.time));
        }

        if let Some(process) = process {
            for (input_name, input_qty) in &process.input {
                let stock_qty = current_stocks.get(input_name).cloned().unwrap_or(0);
                if stock_qty < *input_qty {
                    return Err(format!(
                        "Not enough stock for process '{}' at time {}. Needed {} of {}, but only {} available.",
                        process.id, execution.time, input_qty, input_name, stock_qty
                    ));
                }
            }

            for (input_name, input_qty) in &process.input {
                *current_stocks.get_mut(input_name).unwrap() -= input_qty;
            }
            for (output_name, output_qty) in &process.output {
                *buffered_stocks.entry(output_name.clone()).or_insert(0) += output_qty;
            }
        } else {
            return Err(format!("Process '{}' not found at time {}.", execution.process_name, execution.time));
        }
    }

    for (stock_name, qty) in buffered_stocks {
        *current_stocks.entry(stock_name).or_insert(0) += qty;
    }

    if current_stocks == *final_stocks {
        Ok(())
    } else {
        Err(format!(
            "Final stocks do not match. Expected {:?}, but found {:?}.",
            final_stocks, current_stocks
        ))
    }
}

fn main() {

    let (file, result_to_test) = get_args();

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
    let data = Data {
        stocks: parser.stocks,
        processes: parser.process,
        objectives: parser.optimize.unwrap(),
    };

    let result_file_static: &'static str = Box::leak(file.clone().into_boxed_str());

    let result_file = match File::open(result_to_test) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening result file: {:?}", err);
            return;
        }
    };

    let result_file_lines = io::BufReader::new(result_file).lines();
  
    let (executions, final_stocks) = parse_result_file(result_file_lines);

    println!("stocks: {:?}\n", data.stocks);
    println!("processes:");
    for p in &data.processes {
        println!("{:?}", p);
    }
    println!();
    println!("objectives: {:?}\n", data.objectives);
    println!();
    println!("executions:");
    for e in &executions {
        println!("{:?}", e);
    }
    println!("final stocks: {:?}", final_stocks);   
    println!();
    /**********************/

    match check_execution(&data, &executions, &final_stocks) {
        Ok(()) => println!("Execution is valid."),
        Err(e) => println!("Execution is invalid: {}", e),
    }
}
