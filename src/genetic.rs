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
struct Individual {
    chromosome: Vec<String>,
    fitness: u64,
}

impl Individual {
    fn new(chromosome: Vec<String>) -> Self {
        Individual {
            chromosome,
            fitness: 0,
        }
    }
}

pub fn genetic_algorithm(data: Data, delay: u32) -> Option<(u64, HashMap<String, u64>)> {
    let stock_scores = precompute_stock_scores(&data);
    println!("Stock score: {:?}", stock_scores);
    
    let population_size = 150;
    let generations = 200;
    let mutation_rate = 0.02;

    let mut population: Vec<Individual> = (0..population_size)
        .map(|_| {
            let chromosome = data.processes.iter()
                .map(|p| p.id.clone())
                .collect();
            Individual::new(chromosome)
        })
        .collect();

    let timer_flag = delay::start_timer(std::time::Duration::from_secs(delay as u64));
    let start = Instant::now();

    while true {

        if timer_flag.load(AtomicOrdering::SeqCst) {
            println!("Timer elapsed, stopping optimization");
            break;
        }

        for individual in &mut population {
            individual.fitness = evaluate_fitness(&data, &individual.chromosome, &stock_scores);
        }

        population.sort_by_key(|ind| std::cmp::Reverse(ind.fitness));
        population.truncate(population_size / 2);

        let mut rng = rand::thread_rng();
        let mut new_population = population.clone();
        while new_population.len() < population_size {
            let parent1 = &population[rng.gen_range(0..population.len())];
            let parent2 = &population[rng.gen_range(0..population.len())];
            let crossover_point = rng.gen_range(0..parent1.chromosome.len());
            let mut offspring_chromosome = parent1.chromosome[..crossover_point].to_vec();
            offspring_chromosome.extend_from_slice(&parent2.chromosome[crossover_point..]);
            new_population.push(Individual::new(offspring_chromosome));
        }

        for individual in &mut new_population {
            if rng.gen_bool(mutation_rate) {
                let mutation_point = rng.gen_range(0..individual.chromosome.len());
                let new_process = data.processes[rng.gen_range(0..data.processes.len())].id.clone();
                individual.chromosome[mutation_point] = new_process;
            }
        }

        population = new_population;
    }

    let best_individual = population.iter().max_by_key(|ind| ind.fitness).unwrap();
    let final_stock = simulate(&data, &best_individual.chromosome);
    Some((best_individual.fitness, final_stock))
}

fn evaluate_fitness(data: &Data, chromosome: &[String], stock_scores: &HashMap<String, u64>) -> u64 {
    let stock = simulate(data, chromosome);
    stock.iter().map(|(item, qty)| stock_scores.get(item).unwrap_or(&0) * qty).sum()
}

fn simulate(data: &Data, chromosome: &[String]) -> HashMap<String, u64> {
    let mut current_stock = data.stocks.clone();
    let mut total_time = 0;

    for process_id in chromosome {

        if let Some(process) = data.processes.iter().find(|p| &p.id == process_id) {
            if process.input.iter().all(|(item, qty)| current_stock.get(item).unwrap_or(&0) >= qty) {
                for (item, qty) in &process.input {
                    *current_stock.get_mut(item).unwrap() -= qty;
                }
                for (item, qty) in &process.output {
                    *current_stock.entry(item.clone()).or_insert(0) += qty;
                }
                total_time += process.time as u32;
            }
        }
    }

    current_stock
}

