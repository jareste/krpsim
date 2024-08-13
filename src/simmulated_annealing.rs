extern crate rand;
use rand::prelude::*;
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
pub struct ProcessExecution {
    process: Process,
    execution_time: u64,
}

#[derive(Debug, Clone)]
pub struct State {
    processes: Vec<ProcessExecution>,
    total_time: u64,
    stock_quantities: HashMap<String, u64>,
}

impl State {
    fn new(initial_stocks: &HashMap<String, u64>) -> Self {
        Self {
            processes: Vec::new(),
            total_time: 0,
            stock_quantities: initial_stocks.clone(),
        }
    }

    fn calculate_energy(&self, objectives: &Vec<String>, lambda: f64) -> f64 {
        let mut energy = self.total_time as f64;

        for obj in objectives {
            if let Some(&qty) = self.stock_quantities.get(obj) {
                energy -= lambda * qty as f64;
            }
        }
        // println!("Energy: {}", energy);

        energy
    }

    fn apply_process(&mut self, process: &Process) -> bool {
        for (need, qty) in &process.input {
            if *self.stock_quantities.get(need).unwrap_or(&0) < *qty {
                return false;
            }
        }

        for (need, qty) in &process.input {
            *self.stock_quantities.get_mut(need).unwrap() -= qty;
        }

        for (result, qty) in &process.output {
            *self.stock_quantities.entry(result.clone()).or_insert(0) += qty;
        }

        self.total_time += process.time;
        self.processes.push(ProcessExecution {
            process: process.clone(),
            execution_time: self.total_time,
        });

        true
    }

    fn random_neighbor(&self, data: &Data, rng: &mut ThreadRng) -> Self {
        let mut new_state = self.clone();
        let mut feasible_processes: Vec<(&Process, u64)> = Vec::new();
    
        for process in &data.processes {
            let mut max_runs = u64::MAX;
    
            for (need, qty) in &process.input {
                if let Some(&available_qty) = self.stock_quantities.get(need) {
                    let possible_runs = available_qty / qty;
                    if possible_runs < max_runs {
                        max_runs = possible_runs;
                    }
                } else {
                    max_runs = 0;
                }
            }
    
            if max_runs > 0 {
                feasible_processes.push((process, max_runs));
            }
        }
    
        if feasible_processes.is_empty() {
            return self.clone();
        }
    
        let selected_process = feasible_processes.choose(rng).unwrap();
        let process = selected_process.0;
        let max_runs = selected_process.1;
    
        let times_to_run = rng.gen_range(1..=max_runs);
    
        for _ in 0..times_to_run {
            if !new_state.apply_process(process) {
                break;
            }
        }
    
        new_state
    }
}

pub fn simulated_annealing(data: &Data, initial_temp: f64, lambda: f64, alpha: f64, max_delay: u64) -> (HashMap<String, u64>, u64, Vec<ProcessExecution>) {
    let mut rng = rand::thread_rng();
    let mut current_state = State::new(&data.stocks);
    let mut best_state = current_state.clone();
    let mut temp = initial_temp;

    let timer_flag = delay::start_timer(std::time::Duration::from_secs(max_delay as u64));
    let start = Instant::now();
    

    while true {

        if timer_flag.load(AtomicOrdering::SeqCst) {
            println!("Timer elapsed, stopping optimization");
            break;
        }
        let new_state = current_state.random_neighbor(data, &mut rng);

        let current_energy = current_state.calculate_energy(&data.objectives, lambda);
        let new_energy = new_state.calculate_energy(&data.objectives, lambda);

        if new_energy < current_energy || rng.gen::<f64>() < ((current_energy - new_energy) / temp).exp() {
            current_state = new_state.clone();
        }

        if new_state.calculate_energy(&data.objectives, lambda) < best_state.calculate_energy(&data.objectives, lambda) {
            best_state = new_state.clone();
        }

        temp *= alpha;
    }

    (best_state.stock_quantities.clone(), best_state.total_time.clone(), best_state.processes.clone())
}
