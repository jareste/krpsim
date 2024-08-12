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

pub fn genetic_algorithm(data: Data, max_delay: u32) -> Option<(u64, HashMap<String, u64>)> {
    let stock_scores = precompute_stock_scores(&data);
    
    let population_size = 20000;
    let generations = 100;
    let mutation_rate = 0.1;

    let mut population: Vec<Solution> = (0..population_size)
        .map(|_| {
            let sequence = data.processes.iter()
                .map(|p| p.id.clone())
                .collect();
            Solution::new(sequence, 0, 0)
        })
        .collect();

    let timer_flag = delay::start_timer(std::time::Duration::from_secs(max_delay as u64));
    let start = Instant::now();
    
    while true {

        if timer_flag.load(AtomicOrdering::SeqCst) {
            println!("Timer elapsed, stopping optimization");
            break;
        }

        // Evaluate fitness
        for individual in &mut population {
            (individual.fitness, individual.time) = evaluate_fitness(&data, &individual.sequence, &stock_scores);
        }

        // Selection
        population.sort_by_key(|ind| std::cmp::Reverse(ind.fitness));
        population.truncate(population_size / 2);

        // Crossover
        let mut rng = rand::thread_rng();
        let mut new_population = population.clone();
        while new_population.len() < population_size {
            let parent1 = &population[rng.gen_range(0..population.len())];
            let parent2 = &population[rng.gen_range(0..population.len())];
            let crossover_point = rng.gen_range(0..parent1.sequence.len());
            let mut offspring_sequence = parent1.sequence[..crossover_point].to_vec();
            offspring_sequence.extend_from_slice(&parent2.sequence[crossover_point..]);
            new_population.push(Solution::new(offspring_sequence, 0, 0));
        }

        // Mutation
        for individual in &mut new_population {
            if rng.gen_bool(mutation_rate) {
                let mutation_point = rng.gen_range(0..individual.sequence.len());
                let new_process = data.processes[rng.gen_range(0..data.processes.len())].id.clone();
                individual.sequence[mutation_point] = new_process;
            }
        }

        population = new_population;
    }

    let best_individual = population.iter().max_by_key(|ind| ind.fitness).unwrap();
    let (final_stock,time) = simulate(&data, &best_individual.sequence);
    Some((time, final_stock))
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
                total_time += process.time as u64;
            }
        }
    }
    (current_stock, total_time)
}