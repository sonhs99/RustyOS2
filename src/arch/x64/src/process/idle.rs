use crate::println;

use super::{yield_next, PROCESS_POOL, SCHEDULER};

pub fn idle_process() {
    loop {
        if unsafe { SCHEDULER.wait.count() != 0 } {
            while let Some(wait) = unsafe { SCHEDULER.wait.pop_front() } {
                println!("IDLE: Task ID [0x{wait:X}] ended");
                PROCESS_POOL.lock().dealloc(wait);
            }
        }
        yield_next();
    }
}
