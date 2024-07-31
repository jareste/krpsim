use std::collections::{HashMap, VecDeque};
use crate::Data;
use crate::Process;


impl Data {
    fn new() -> Data {
        Data {
            stocks: HashMap::new(),
            processes: Vec::new(),
            objectives: Vec::new(),
        }
    }

    fn objective_value(&self) -> u64 {
        self.objectives.iter().map(|obj| *self.stocks.get(obj).unwrap_or(&0)).sum()
    }

    fn can_execute(&self, process: &Process) -> bool {
        process.input.iter().all(|(stock, amount)| {
            self.stocks.get(stock).unwrap_or(&0) >= amount
        })
    }

    fn execute_process(&mut self, process: &Process) {
        for (stock, amount) in &process.input {
            *self.stocks.get_mut(stock).unwrap() -= *amount;
        }
        for (stock, amount) in &process.output {
            *self.stocks.entry(stock.clone()).or_insert(0) += *amount;
        }
    }

    fn generate_neighbors(&self) -> Vec<(Data, u64)> {
        let mut neighbors = Vec::new();
        for process in &self.processes {
            if self.can_execute(process) {
                let mut new_state = self.clone();
                new_state.execute_process(process);
                neighbors.push((new_state, process.time));
            }
        }
        neighbors
    }
}

pub fn tabu_search(data: &Data, max_iterations: usize, tabu_list_size: usize) -> (Data, u64) {
    let mut best_solution = data.clone();
    let mut current_solution = data.clone();
    let mut best_time = 0;
    let mut current_time = 0;
    let mut tabu_list = VecDeque::new();
    let mut iterations = 0;

    while iterations < max_iterations {
        let neighbors = current_solution.generate_neighbors();
        
        let mut best_neighbor = None;
        let mut best_neighbor_value = 0;
        let mut best_neighbor_time = 0;

        for (neighbor, time) in &neighbors {
            if !tabu_list.contains(&neighbor.stocks) {
                let neighbor_value = neighbor.objective_value();
                if neighbor_value > best_neighbor_value {
                    best_neighbor_value = neighbor_value;
                    best_neighbor_time = *time;
                    best_neighbor = Some(neighbor);
                }
            }
        }

        if let Some(best) = best_neighbor {
            current_solution = best.clone();
            current_time += best_neighbor_time;
            if current_solution.objective_value() > best_solution.objective_value() {
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

    (best_solution, best_time)
}