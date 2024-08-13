use std::collections::{HashMap, HashSet, VecDeque};
use crate::Data;

pub fn precompute_stock_scores(data: &Data) -> HashMap<String, u64> {
    let mut stock_scores: HashMap<String, u64> = HashMap::new();
    let mut to_visit: VecDeque<(String, u64)> = VecDeque::new();
    let mut visited: HashSet<String> = HashSet::new();

    let max_score = data.stocks.len() as u64;

    for objective in &data.objectives {
        to_visit.push_back((objective.clone(), max_score)); // Assign max score to objectives
    }

    while let Some((current_stock, score)) = to_visit.pop_front() {
        if let Some(&existing_score) = stock_scores.get(&current_stock) {
            if score <= existing_score {
                continue;  // Only update if the new score is lower (better)
            }
        }

        stock_scores.insert(current_stock.clone(), score);

        for process in &data.processes {
            for (output, _) in &process.output {
                if output == &current_stock {
                    for (input, _) in &process.input {
                        if !visited.contains(input) {
                            to_visit.push_back((input.clone(), score.saturating_sub(1)));  // Decrease the score as you move away
                        }
                    }
                }
            }
        }

        visited.insert(current_stock);
    }

    stock_scores
}