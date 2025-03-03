use crate::Data;
use crate::Process;
use crate::delay;
use crate::stock_scores;
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;

#[derive(Clone, Eq, PartialEq, Debug)]
struct StockState(HashMap<String, u64>);

impl Hash for StockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in &self.0 {
            key.hash(state);
            value.hash(state);
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct State {
    time: u64,
    stocks: StockState,
    objectives: HashMap<String, u64>,
    log: Vec<(String, u64, u64)>,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.objectives.values().sum::<u64>()
            .cmp(&self.objectives.values().sum::<u64>())
            .then_with(|| self.time.cmp(&other.time))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl State {
    fn new(time: u64, stocks: HashMap<String, u64>, objectives: &[String], log: Vec<(String, u64, u64)>) -> Self {
        let mut objectives_map = HashMap::new();
        for obj in objectives {
            objectives_map.insert(obj.clone(), *stocks.get(obj).unwrap_or(&0));
        }
        State { time, stocks: StockState(stocks), objectives: objectives_map, log }
    }

    fn apply_processes(&self, processes: &[Process], objectives: &[String]) -> Vec<Self> {
        let mut new_states = Vec::new();

        for process in processes {
            let max_executable_times = process
                .input
                .iter()
                .map(|(input_item, input_amount)| {
                    self.stocks
                        .0
                        .get(input_item)
                        .map_or(0, |&available| available / input_amount)
                })
                .min()
                .unwrap_or(0);

            for count in 1..=max_executable_times {
                let mut new_stocks = self.stocks.0.clone();
                let mut new_log = self.log.clone();
                let mut valid_state = true;

                for (input_item, input_amount) in &process.input {
                    if *new_stocks.get(input_item).unwrap() < input_amount * count {
                        valid_state = false;
                        break;
                    }
                    *new_stocks.get_mut(input_item).unwrap() -= input_amount * count;
                }

                if !valid_state {
                    continue;
                }

                for (output_item, output_amount) in &process.output {
                    *new_stocks.entry(output_item.clone()).or_insert(0) += output_amount * count;
                }

                new_log.push((process.id.clone(), count, self.time + process.time * count));

                let mut new_objectives = self.objectives.clone();
                for obj in objectives {
                    *new_objectives.entry(obj.clone()).or_insert(0) = *new_stocks.get(obj).unwrap_or(&0);
                }

                new_states.push(State {
                    time: self.time + process.time * count,
                    stocks: StockState(new_stocks),
                    objectives: new_objectives,
                    log: new_log,
                });
            }
        }

        new_states
    }
}

pub fn optimize(data: Data, delay: u32) -> Option<(u64, HashMap<String, u64>, Vec<(String, u64, u64)>)> {
    let mut heap = BinaryHeap::new();
    let mut visited = HashSet::new();
    let mut best_time = u64::MAX;
    let mut best_stocks = None;
    let mut best_log = None;
    let mut best_objective_sum = 0;

    let heuristic_scores = stock_scores::precompute_stock_scores(&data);

    let timer_flag = delay::start_timer(std::time::Duration::from_secs(delay as u64));
    let start = Instant::now();

    heap.push(State::new(0, data.stocks.clone(), &data.objectives, vec![]));

    while let Some(state) = heap.pop() {

        /* delay checker */
        if timer_flag.load(AtomicOrdering::SeqCst) {
            println!("Timer elapsed, stopping optimization");
            break;
        }

        if visited.contains(&(state.stocks.clone(), state.time)) {
            continue;
        }

        visited.insert((state.stocks.clone(), state.time));

        let current_objective_sum: u64 = state.objectives.values().sum();

        if current_objective_sum > best_objective_sum {
            best_objective_sum = current_objective_sum;
            best_time = state.time;
            best_stocks = Some(state.stocks.0.clone());
            best_log = Some(state.log.clone());
        } else if current_objective_sum == best_objective_sum && state.time < best_time {
            best_time = state.time;
            best_stocks = Some(state.stocks.0.clone());
            best_log = Some(state.log.clone());
        }

        let new_states = state.apply_processes(&data.processes, &data.objectives);
        for mut new_state in new_states {
            // Calculate the heuristic estimate (h) based on the remaining objectives
            let heuristic_estimate = new_state.objectives.keys().map(|obj| {
                heuristic_scores.get(obj).cloned().unwrap_or(0)
            }).sum::<u64>();

            // Update the state with the estimated total cost (g + h)
            let total_cost_estimate = new_state.time + heuristic_estimate;
            
            // Push the state to the heap with the total cost estimate
            heap.push(State {
                time: total_cost_estimate,  // Use estimated total cost for priority
                ..new_state
            });
        }
    }

    let elapsed = start.elapsed();

    println!("A* executed in: {}.{:03} seconds\n", elapsed.as_secs(), elapsed.subsec_millis());

    best_stocks.map(|stocks| (best_time, stocks, best_log.unwrap_or_default()))
}
