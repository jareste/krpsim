extern crate rand;
use crate::stock_scores;
use crate::Data;
use crate::Process;
use crate::delay;
use rand::Rng;
use std::collections::HashMap;
use crate::stock_scores::precompute_stock_scores;
use std::cmp::Ordering;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;


#[derive(Debug, Clone)]
struct Solution {
    sequence: Vec<String>,
    fitness: i32,
    time: u64,
}

impl Solution {
    fn new(sequence: Vec<String>, fitness: i32, time: u64) -> Self {
        Solution { sequence, fitness, time }
    }
}

pub fn simmulated_annealing(data: Data, max_delay: u32) -> Option<(u64, HashMap<String, u64>)> {
    let stock_scores = precompute_stock_scores(&data);

    let mut rng = rand::thread_rng();
    let initial_temp = 200.0;
    let final_temp = 0.1;
    let alpha = 0.96;
    let mut temp = initial_temp;

    // Initial solution
    let mut current_solution = generate_initial_solution(&data);
    (current_solution.fitness, current_solution.time) = evaluate_fitness(&data, &current_solution.sequence, &stock_scores);

    let mut best_solution = current_solution.clone();

    while temp > final_temp {
        let mut new_solution = perturb_solution(&data, &current_solution);
        (new_solution.fitness, new_solution.time) = evaluate_fitness(&data, &new_solution.sequence, &stock_scores);

        if acceptance_probability(current_solution.fitness, new_solution.fitness, temp) > rng.gen::<f64>() {
            current_solution = new_solution;
        }

        if current_solution.fitness > best_solution.fitness {
            best_solution = current_solution.clone();
        }

        temp *= alpha;
    }

    let (final_stock, time) = simulate(&data, &best_solution.sequence);
    Some((time, final_stock))
}

fn generate_initial_solution(data: &Data) -> Solution {
    let sequence = data.processes.iter().map(|p| p.id.clone()).collect();
    Solution::new(sequence, 0, 0)
}

fn perturb_solution(data: &Data, solution: &Solution) -> Solution {
    let mut rng = rand::thread_rng();
    let mut new_sequence = solution.sequence.clone();
    let idx = rng.gen_range(0..new_sequence.len());
    new_sequence[idx] = data.processes[rng.gen_range(0..data.processes.len())].id.clone();
    Solution::new(new_sequence, 0, 0)
}

fn acceptance_probability(old_fitness: i32, new_fitness: i32, temp: f64) -> f64 {
    if new_fitness > old_fitness {
        1.0
    } else {
        ((new_fitness as f64 - old_fitness as f64) / temp).exp()
    }
}

fn evaluate_fitness(data: &Data, sequence: &[String], stock_scores: &HashMap<String, u64>) -> (i32, u64) {
    let (stock, time) = simulate(data, sequence);
    let fit: u64 = (stock.iter().map(|(item, qty)| stock_scores.get(item).unwrap_or(&0) * qty).sum());
    (-1 * fit as i32, time)
}

fn simulate(data: &Data, sequence: &[String]) -> (HashMap<String, u64>, u64) {
    let mut current_stock = data.stocks.clone();
    let mut total_time = 0;

    for process_id in sequence {

        if let Some(process) = data.processes.iter().find(|p| &p.id == process_id) {
            if process.input.iter().all(|(item, qty)| current_stock.get(item).unwrap_or(&0) >= qty) {
                for (item, qty) in &process.input {
                    *current_stock.get_mut(item).unwrap() -= qty;
                }
                for (item, qty) in &process.output {
                    *current_stock.entry(item.clone()).or_insert(0) += qty;
                }
                total_time += process.time;
            }
        }
    }
    (current_stock, total_time)
}