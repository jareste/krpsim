use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Once};
use std::thread;
use std::time::Duration;
use ctrlc;

static INIT: Once = Once::new();

pub fn start_timer(duration: Duration) -> Arc<AtomicBool> {
    let timer_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&timer_flag);

    let ctrlc_flag = Arc::clone(&timer_flag);
    INIT.call_once(|| {
        ctrlc::set_handler(move || {
            ctrlc_flag.store(true, Ordering::SeqCst);
            println!("\x1b[33m\nCtrl-C detected, stopping optimization\n\x1b[0m");
        }).expect("Error setting Ctrl-C handler");
    });

    thread::spawn(move || {
        thread::sleep(duration);
        flag_clone.store(true, Ordering::SeqCst);
    });

    timer_flag
}