use std::collections::{HashMap, VecDeque};
use crate::Data;
use crate::Process;
use crate::delay;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;

pub fn new_data() -> Data {
    Data {
        stocks: HashMap::new(),
        processes: Vec::new(),
        objectives: Vec::new(),
    }
}

pub fn objective_value(data: &Data) -> u64 {
    data.objectives.iter().map(|obj| *data.stocks.get(obj).unwrap_or(&0)).sum()
}

pub fn can_execute(data: &Data, process: &Process, count: u64) -> bool {
    process.input.iter().all(|(stock, amount)| {
        data.stocks.get(stock).unwrap_or(&0) >= &(amount * count)
    })
}

pub fn execute(data: &mut Data, process: &Process, count: u64) {
    for (stock, amount) in &process.input {
        *data.stocks.get_mut(stock).unwrap() -= amount * count;
    }
    for (stock, amount) in &process.output {
        *data.stocks.entry(stock.clone()).or_insert(0) += amount * count;
    }
}

pub fn generate_neighbors(data: &Data) -> Vec<(Data, u64)> {
    let mut neighbors = Vec::new();
    for process in &data.processes {
        let max_count = process.input.iter().map(|(stock, amount)| {
            data.stocks.get(stock).unwrap() / amount
        }).min().unwrap_or(0);

        if max_count > 0 {
            let mut new_state = data.clone();
            execute(&mut new_state, process, max_count);
            neighbors.push((new_state, process.time));
        }
    }
    neighbors
}

pub fn tabu_search(data: &Data, max_iterations: usize, tabu_list_size: usize, delay: u32) -> (Data, u64) {
    let mut best_solution = data.clone();
    let mut current_solution = data.clone();
    let mut best_time = 0;
    let mut current_time = 0;
    let mut tabu_list = VecDeque::new();
    let mut iterations = 0;

    let timer_flag = delay::start_timer(std::time::Duration::from_secs(delay as u64));
    let start = Instant::now();

    while iterations < max_iterations {
        if timer_flag.load(AtomicOrdering::SeqCst) {
            println!("Timer elapsed, stopping optimization\n");
            break;
        }

        let neighbors = generate_neighbors(&current_solution);
        
        let mut best_neighbor = None;
        let mut best_neighbor_value = 0;
        let mut best_neighbor_time = 0;

        for (neighbor, time) in &neighbors {
            if !tabu_list.contains(&neighbor.stocks) {
                let neighbor_value = objective_value(neighbor);
                if neighbor_value > best_neighbor_value || (neighbor_value == best_neighbor_value && *time < best_neighbor_time) {
                    best_neighbor_value = neighbor_value;
                    best_neighbor_time = *time;
                    best_neighbor = Some(neighbor);
                }
            }
        }

        if let Some(best) = best_neighbor {
            current_solution = best.clone();
            current_time += best_neighbor_time;
            if objective_value(&current_solution) > objective_value(&best_solution) {
                best_solution = current_solution.clone();
                best_time = current_time;
            } else if objective_value(&current_solution) == objective_value(&best_solution) && current_time < best_time {
                best_solution = current_solution.clone();
                best_time = current_time;
            }

        } else if !neighbors.is_empty() {
            let (neighbor, time) = &neighbors[0];
            current_solution = neighbor.clone();
            current_time += *time;
        }

        tabu_list.push_back(current_solution.stocks.clone());
        if tabu_list.len() > tabu_list_size {
            tabu_list.pop_front();
        }

        iterations += 1;
    }

    let elapsed = start.elapsed();

    println!("Tabu Search executed in: {}.{:03} seconds\n", elapsed.as_secs(), elapsed.subsec_millis());

    (best_solution, best_time)
}
