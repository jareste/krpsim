use std::fs::File;
use std::io::{self, Write};
use std::collections::HashMap;
use std::thread;

pub fn run_in_thread(
    filename: String,
    initial_stocks: HashMap<String, u64>,
    final_stocks: HashMap<String, u64>,
    log: Vec<(String, u64, u64)>,
    finish_time: u64,
) {
    thread::spawn(move || {
        if let Err(e) = generate_log_file(filename, initial_stocks, final_stocks, log, finish_time) {
            eprintln!("Failed to generate log file: {}", e);
        }
    });
}

pub fn generate_log_file(
    filename: String,
    initial_stocks: HashMap<String, u64>,
    final_stocks: HashMap<String, u64>,
    log: Vec<(String, u64, u64)>,
    finish_time: u64,
) -> io::Result<()> {
    let mut file = File::create(filename)?;

    writeln!(file, "Initial stocks:")?;
    for (stock, amount) in &initial_stocks {
        writeln!(file, "{}:{}", stock, amount)?;
    }
    writeln!(file)?;

    for (process, count, time) in &log {
        for _ in 0..*count {
            writeln!(file, "{}:{}", time, process)?;
        }
    }
    writeln!(file)?;

    writeln!(file, "Finished at time {}", finish_time)?;
    writeln!(file)?;

    writeln!(file, "Final stocks:")?;
    for (stock, amount) in &final_stocks {
        writeln!(file, "{}:{}", stock, amount)?;
    }

    Ok(())
}