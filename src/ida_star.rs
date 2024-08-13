use crate::Data;
use crate::Process;
use crate::delay;
use crate::stock_scores;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

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

impl StockState {
    fn calculate_heuristic(&self, objectives: &[String], heuristic_scores: &HashMap<String, u64>) -> u64 {
        objectives.iter().map(|obj| heuristic_scores.get(obj).cloned().unwrap_or(0)).sum()
    }
}

#[derive(Clone, Eq, PartialEq)]
struct State {
    time: u64,
    stocks: StockState,
    objectives: HashMap<String, u64>,
    log: Vec<(String, u64, u64)>,
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.time.hash(state);
        self.stocks.hash(state);
        for (key, value) in &self.objectives {
            key.hash(state);
            value.hash(state);
        }
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
    let heuristic_scores = stock_scores::precompute_stock_scores(&data);
    let mut visited_global = HashSet::new();
    let timer_flag = delay::start_timer(std::time::Duration::from_secs(delay as u64));
    let start = Instant::now();

    let initial_state = State::new(0, data.stocks.clone(), &data.objectives, vec![]);
    let mut threshold = initial_state.stocks.calculate_heuristic(&data.objectives, &heuristic_scores)
        .max(initial_state.time);
    let mut best_state: Option<State> = None;

    loop {
        if timer_flag.load(AtomicOrdering::SeqCst) {
            println!("Timer elapsed, stopping optimization");
            break;
        }

        visited_global.clear();

        let (result, new_threshold) = depth_limited_search(
            initial_state.clone(),
            &data,
            &heuristic_scores,
            threshold,
            &mut best_state,
            &mut visited_global,
            &timer_flag
        );

        if let Some(final_state) = result {
            best_state = Some(final_state);
        }

        if new_threshold == u64::MAX || new_threshold == threshold {
            break;
        }

        threshold = new_threshold;
    }

    let elapsed = start.elapsed();
    println!("IDA* executed in: {}.{:03} seconds\n", elapsed.as_secs(), elapsed.subsec_millis());

    best_state.map(|state| (state.time, state.stocks.0, state.log))
}

fn depth_limited_search(
    state: State,
    data: &Data,
    heuristic_scores: &HashMap<String, u64>,
    limit: u64,
    best_state: &mut Option<State>,
    visited_global: &mut HashSet<(StockState, u64)>,
    timer_flag: &Arc<AtomicBool>
) -> (Option<State>, u64) {
    let f_value = state.time + state.stocks.calculate_heuristic(&data.objectives, heuristic_scores);

    if f_value > limit {
        return (None, f_value);
    }

    if timer_flag.load(AtomicOrdering::SeqCst) {
        return (None, u64::MAX);
    }

    if visited_global.contains(&(state.stocks.clone(), state.time)) {
        return (None, u64::MAX);
    }

    visited_global.insert((state.stocks.clone(), state.time));

    if let Some(ref best) = best_state {
        if state.objectives.values().sum::<u64>() > best.objectives.values().sum::<u64>() {
            *best_state = Some(state.clone());
        } else if state.objectives.values().sum::<u64>() == best.objectives.values().sum::<u64>() && state.time < best.time {
            *best_state = Some(state.clone());
        }
    } else {
        *best_state = Some(state.clone());
    }

    let mut min_threshold = u64::MAX;
    let mut local_best_state = None;

    for new_state in state.apply_processes(&data.processes, &data.objectives) {
        let (result, threshold) = depth_limited_search(new_state.clone(), data, heuristic_scores, limit, best_state, visited_global, timer_flag);

        if let Some(r) = result {
            local_best_state = Some(r);
        }

        if threshold < min_threshold {
            min_threshold = threshold;
        }
    }

    (local_best_state.or(best_state.clone()), min_threshold)
}
