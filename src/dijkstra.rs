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
        let mut max_time = 0;
        let mut combined_log = self.log.clone();
        let mut combined_stocks = self.stocks.0.clone();
        let mut executed_any = false;

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

            if max_executable_times > 0 {
                executed_any = true;

                for (input_item, input_amount) in &process.input {
                    *combined_stocks.get_mut(input_item).unwrap() -= input_amount * max_executable_times;
                }

                for (output_item, output_amount) in &process.output {
                    *combined_stocks.entry(output_item.clone()).or_insert(0) += output_amount * max_executable_times;
                }

                max_time = max_time.max(process.time);

                combined_log.push((process.id.clone(), max_executable_times, self.time + process.time));
            }
        }

        if executed_any {
            let mut new_objectives = self.objectives.clone();
            for obj in objectives {
                *new_objectives.entry(obj.clone()).or_insert(0) = *combined_stocks.get(obj).unwrap_or(&0);
            }

            new_states.push(State {
                time: self.time + max_time,
                stocks: StockState(combined_stocks),
                objectives: new_objectives,
                log: combined_log,
            });
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
        for new_state in new_states {
            heap.push(new_state);
        }
    }

    let elapsed = start.elapsed();

    println!("Dijkstra executed in: {}.{:03} seconds\n", elapsed.as_secs(), elapsed.subsec_millis());

    best_stocks.map(|stocks| (best_time, stocks, best_log.unwrap_or_default()))
}
