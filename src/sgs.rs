use crate::Data;
use crate::Process;
use crate::delay;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::time::Instant;

pub fn sgs_algorithm(mut data: Data, delay: u64) -> (Data, u64, Vec<(String, u64, u64)>) {
    let timer_flag = delay::start_timer(std::time::Duration::from_secs(delay as u64));
    let start = Instant::now();

    fn score_process(process: &Process, stocks: &HashMap<String, u64>, objectives: &Vec<String>) -> i64 {
        let mut score = 0;

        for (output_item, output_amount) in &process.output {
            if objectives.contains(output_item) {
                score += *output_amount as i64;
            }
        }

        for (input_item, input_amount) in &process.input {
            if objectives.contains(input_item) {
                score -= *input_amount as i64;
            } else {
                score -= (*input_amount as i64) / 2;
            }
        }

        score
    }

    let mut total_simulated_time = 0;
    let mut execution_log = Vec::new();

    while !timer_flag.load(AtomicOrdering::SeqCst) {
        let mut eligible_processes = vec![];
        for process in &data.processes {
            let mut is_eligible = true;
            for (input_item, input_amount) in &process.input {
                if let Some(stock) = data.stocks.get(input_item) {
                    if stock < input_amount {
                        is_eligible = false;
                        break;
                    }
                } else {
                    is_eligible = false;
                    break;
                }
            }
            if is_eligible {
                eligible_processes.push(process.clone());
            }
        }

        if eligible_processes.is_empty() {
            break;
        }

        eligible_processes.sort_by_key(|process| -score_process(process, &data.stocks, &data.objectives));

        let selected_process = eligible_processes.first().unwrap();

        let mut max_executions = u64::MAX;
        for (input_item, input_amount) in &selected_process.input {
            if let Some(stock) = data.stocks.get(input_item) {
                max_executions = max_executions.min(stock / input_amount);
            }
        }

        for (input_item, input_amount) in &selected_process.input {
            if let Some(stock) = data.stocks.get_mut(input_item) {
                *stock -= input_amount * max_executions;
            }
        }

        for (output_item, output_amount) in &selected_process.output {
            *data.stocks.entry(output_item.clone()).or_insert(0) += output_amount * max_executions;
        }

        execution_log.push((selected_process.id.clone(), max_executions, total_simulated_time));

        total_simulated_time += selected_process.time;

    }

    let elapsed = start.elapsed();

    println!("SGS* executed in: {}.{:03} seconds\n", elapsed.as_secs(), elapsed.subsec_millis());

    (data, total_simulated_time, execution_log)
}

