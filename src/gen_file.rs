use std::fs::File;
use std::io::{self, Write};
use std::collections::HashMap;

pub fn generate_log_file(filename: &str, initial_stocks: &HashMap<String, u64>, final_stocks: &HashMap<String, u64>, log: &[(String, u64, u64)], finish_time: u64) -> io::Result<()> {
    let mut file = File::create(filename)?;

    // Write initial stocks
    writeln!(file, "Initial stocks:")?;
    for (stock, amount) in initial_stocks {
        writeln!(file, "{}:{}", stock, amount)?;
    }
    writeln!(file)?;

    // Write the log of actions
    for (process, count, time) in log {
        for _ in 0..*count {
            writeln!(file, "{}:{}", time, process)?;
        }
    }
    writeln!(file)?;

    // Write finish time
    writeln!(file, "Finished at time {}", finish_time)?;
    writeln!(file)?;

    // Write final stocks
    writeln!(file, "Final stocks:")?;
    for (stock, amount) in final_stocks {
        writeln!(file, "{}:{}", stock, amount)?;
    }

    Ok(())
}