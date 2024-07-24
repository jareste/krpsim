use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn start_timer(duration: Duration) -> Arc<AtomicBool> {
    let timer_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&timer_flag);

    thread::spawn(move || {
        thread::sleep(duration);
        flag_clone.store(true, Ordering::SeqCst);
    });

    timer_flag
}