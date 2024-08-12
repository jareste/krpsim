use std::collections::{HashMap, HashSet, VecDeque};
use crate::Data;

pub fn precompute_stock_scores(data: &Data) -> HashMap<String, u64> {
    let mut stock_scores: HashMap<String, u64> = HashMap::new();
    let mut to_visit: VecDeque<(String, u64)> = VecDeque::new();
    let mut visited: HashSet<String> = HashSet::new();

    for objective in &data.objectives {
        to_visit.push_back((objective.clone(), 0));
    }

    while let Some((current_stock, score)) = to_visit.pop_front() {
        if !visited.insert(current_stock.clone()) {
            continue;
        }
        stock_scores.insert(current_stock.clone(), score);

        for process in &data.processes {
            for (output, _) in &process.output {
                if output == &current_stock {
                    for (input, _) in &process.input {
                        if !visited.contains(input) {
                            to_visit.push_back((input.clone(), score + 1));
                        }
                    }
                }
            }
        }
    }

    stock_scores
}