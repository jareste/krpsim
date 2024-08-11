use crate::Data;
use crate::Process;
use crate::delay;
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
    heuristic: u64,
    log: Vec<(String, u64, u64)>,  // Added field to store logs
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.objectives.values().sum::<u64>() + other.heuristic)
            .cmp(&(self.objectives.values().sum::<u64>() + self.heuristic))
            .then_with(|| self.time.cmp(&other.time)) // Prioritize more objectives, then less time
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl State {
    fn new(time: u64, stocks: HashMap<String, u64>, objectives: &[String], heuristic: u64, log: Vec<(String, u64, u64)>) -> Self {
        let mut objectives_map = HashMap::new();
        for obj in objectives {
            objectives_map.insert(obj.clone(), *stocks.get(obj).unwrap_or(&0));
        }
        State { time, stocks: StockState(stocks), objectives: objectives_map, heuristic, log }
    }

    fn apply_process(&self, process: &Process, objectives: &[String]) -> Option<Vec<Self>> {
        let mut new_states = Vec::new();
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

        for times in 1..=max_executable_times {
            let mut new_stocks = self.stocks.0.clone();

            for (input_item, input_amount) in &process.input {
                *new_stocks.get_mut(input_item).unwrap() -= input_amount * times;
            }

            for (output_item, output_amount) in &process.output {
                *new_stocks.entry(output_item.clone()).or_insert(0) += output_amount * times;
            }

            let mut new_objectives = self.objectives.clone();
            for obj in objectives {
                *new_objectives.entry(obj.clone()).or_insert(0) = *new_stocks.get(obj).unwrap_or(&0);
            }

            let heuristic = calculate_heuristic(&new_stocks, objectives);

            // Clone the existing log and add the new process entry
            let mut new_log = self.log.clone();
            new_log.push((process.id.clone(), times, self.time + process.time));

            new_states.push(State {
                time: self.time + process.time,
                stocks: StockState(new_stocks),
                objectives: new_objectives,
                heuristic,
                log: new_log,
            });
        }

        if new_states.is_empty() {
            None
        } else {
            Some(new_states)
        }
    }
}

fn calculate_heuristic(stocks: &HashMap<String, u64>, objectives: &[String]) -> u64 {
    objectives.iter().map(|obj| *stocks.get(obj).unwrap_or(&0)).sum()
}

pub fn optimize(data: Data, delay: u32) -> Option<(u64, HashMap<String, u64>, Vec<(String, u64, u64)>)> {
    let mut heap = BinaryHeap::new();
    let mut visited = HashSet::new();
    let mut best_time = u64::MAX;
    let mut best_stocks = None;
    let mut best_log = None;

    let timer_flag = delay::start_timer(std::time::Duration::from_secs(delay as u64));
    let start = Instant::now();

    let optimize_for_time = false;

    let initial_heuristic = calculate_heuristic(&data.stocks, &data.objectives);
    heap.push(State::new(0, data.stocks.clone(), &data.objectives, initial_heuristic, vec![]));
    while let Some(state) = heap.pop() {

        /* delay checker */
        if timer_flag.load(AtomicOrdering::SeqCst) {
            println!("Timer elapsed, stopping optimization");
            break;
        }

        if visited.contains(&state.stocks) {
            continue;
        }

        visited.insert(state.stocks.clone());

        let current_objective_sum: u64 = state.objectives.values().sum();

        if current_objective_sum > 0 {
            if !optimize_for_time || state.time < best_time || best_stocks.is_none() {
                best_time = state.time;
                best_stocks = Some(state.stocks.0.clone());
                best_log = Some(state.log.clone());
            }
        }

        for process in &data.processes {
            if let Some(new_states) = state.apply_process(process, &data.objectives) {
                for new_state in new_states {
                    heap.push(new_state);
                }
            }
        }
    }

    let elapsed = start.elapsed();

    println!("Dijkstra executed in: {}.{:03} seconds\n", elapsed.as_secs(), elapsed.subsec_millis());

    best_stocks.map(|stocks| (best_time, stocks, best_log.unwrap_or_default()))
}
