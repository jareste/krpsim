use std::collections::{HashMap, VecDeque};
use crate::Data;
use crate::Process;
use crate::delay;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;
use rand::prelude::*;

fn initialize_pheromones(processes: &Vec<Process>) -> HashMap<String, f64> {
    let mut pheromones = HashMap::new();
    for process in processes {
        pheromones.insert(process.id.clone(), 1.0);
    }
    pheromones
}

fn construct_solution(data: &Data, pheromones: &HashMap<String, f64>, rng: &mut ThreadRng) -> Vec<(String, usize)> {
    let mut solution = Vec::new();
    let mut available_stocks = data.stocks.clone();
    let mut total_time = 0;

    loop {
        let next_processes = select_next_processes(&data.processes, &available_stocks, &pheromones, rng);
        if next_processes.is_empty() {
            break;
        }

        let mut max_time_per_step = 0;

        for (process, count) in &next_processes {
            let process_clone = process.clone();
            for _ in 0..*count {
                if update_stocks_and_time(&mut available_stocks, &process_clone) {
                    solution.push((process.id.clone(), *count));
                    max_time_per_step = max_time_per_step.max(process.time);
                }
            }
        }

        total_time += max_time_per_step;
    }

    solution
}

fn select_next_processes<'a>(
    processes: &'a Vec<Process>,
    available_stocks: &HashMap<String, u64>,
    pheromones: &HashMap<String, f64>,
    rng: &mut ThreadRng
) -> Vec<(&'a Process, usize)> {
    let mut selected_processes = Vec::new();
    let mut process_weights = Vec::new();
    let mut total_weight = 0.0;

    for process in processes {
        let mut max_count = u64::MAX;

        for (item, amount) in &process.input {
            if let Some(stock) = available_stocks.get(item) {
                max_count = max_count.min(stock / amount);
            } else {
                max_count = 0;
                break;
            }
        }

        if max_count > 0 {
            let pheromone_level = *pheromones.get(&process.id).unwrap();
            let heuristic_value = 1.0 / (process.time as f64);
            let weight = pheromone_level * heuristic_value;

            process_weights.push((process, max_count as usize, weight));
            total_weight += weight;
        }
    }

    if total_weight > 0.0 {
        let mut chosen_weight = rng.gen_range(0.0..total_weight);
        for (process, max_count, weight) in &process_weights {
            if chosen_weight < *weight {
                let count = rng.gen_range(1..=*max_count);
                selected_processes.push((*process, count));
                break;
            } else {
                chosen_weight -= *weight;
            }
        }
    }

    selected_processes
}

fn update_stocks_and_time(stocks: &mut HashMap<String, u64>, process: &Process) -> bool {
    for (input, amount) in &process.input {
        if let Some(stock) = stocks.get_mut(input) {
            if *stock < *amount {
                return false;
            }
        } else {
            return false;
        }
    }

    for (input, amount) in &process.input {
        if let Some(stock) = stocks.get_mut(input) {
            *stock -= amount;
        }
    }

    for (output, amount) in &process.output {
        let stock = stocks.entry(output.clone()).or_insert(0);
        *stock += amount;
    }

    true
}

fn evaluate_solution(data: &Data, solution: &Vec<(String, usize)>) -> (u64, u64, HashMap<String, u64>) {
    let mut available_stocks = data.stocks.clone();
    let mut total_time = 0;
    let mut max_time_per_step = 0;

    for (process_id, count) in solution {
        if let Some(process) = data.processes.iter().find(|p| &p.id == process_id) {
            for _ in 0..*count {
                if update_stocks_and_time(&mut available_stocks, process) {
                    max_time_per_step = max_time_per_step.max(process.time);
                }
            }
        }
        total_time += max_time_per_step;
        max_time_per_step = 0;
    }

    let mut objective_score = 0;
    for objective in &data.objectives {
        if let Some(stock) = available_stocks.get(objective) {
            objective_score += stock;
        }
    }

    (objective_score, total_time, available_stocks)
}

fn update_pheromones(
    pheromones: &mut HashMap<String, f64>,
    solutions: &Vec<(Vec<(String, usize)>, u64, u64)>
) {
    for pheromone in pheromones.values_mut() {
        *pheromone *= 0.9;
    }

    for (solution, objective_score, total_time) in solutions {
        let pheromone_increase = *objective_score as f64 / *total_time as f64;
        for (process_id, count) in solution {
            if let Some(pheromone) = pheromones.get_mut(process_id) {
                *pheromone += pheromone_increase * *count as f64;
            }
        }
    }
}

pub fn aco_optimization(data: &Data, num_iterations: usize, num_ants: usize, delay: u32) -> (Vec<(String, usize)>, u64, HashMap<String, u64>) {
    let mut pheromones = initialize_pheromones(&data.processes);
    let mut rng = thread_rng();

    let mut best_solution = Vec::new();
    let mut best_score = 0;
    let mut best_time = u64::MAX;
    let mut best_stocks = HashMap::new();

    let timer_flag = delay::start_timer(std::time::Duration::from_secs(delay as u64));
    let start = Instant::now();

    for _ in 0..num_iterations {
        if timer_flag.load(AtomicOrdering::SeqCst) {
            println!("Timer elapsed, stopping optimization\n");
            break;
        }
        
        let mut solutions = Vec::new();

        for _ in 0..num_ants {
            let solution = construct_solution(data, &pheromones, &mut rng);
            let (objective_score, total_time, final_stocks) = evaluate_solution(data, &solution);
            solutions.push((solution.clone(), objective_score, total_time));

            if objective_score > best_score || (objective_score == best_score && total_time < best_time) {
                best_score = objective_score;
                best_time = total_time;
                best_solution = solution;
                best_stocks = final_stocks;
            }
        }

        update_pheromones(&mut pheromones, &solutions);
    }

    let elapsed = start.elapsed();

    println!("Ant Colony Optimitzation algo executed in: {}.{:03} seconds\n", elapsed.as_secs(), elapsed.subsec_millis());

    (best_solution, best_time, best_stocks)
}