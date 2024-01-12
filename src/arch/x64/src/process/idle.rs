use crate::{interrupt, println};

use super::{yield_next, PROCESS_POOL, SCHEDULER};
use crate::assembly;

pub(super) static mut IDLE_COUNT: u64 = 0;
pub(super) static mut TICK_COUNT: u64 = 0;
static mut PROCESS_LOAD: u64 = 0;

pub fn idle_process() {
    let mut idle_count = unsafe { IDLE_COUNT };
    let mut tick_count = unsafe { TICK_COUNT };
    loop {
        let current_idle_count = unsafe { IDLE_COUNT };
        let current_tick_count = unsafe { TICK_COUNT };
        unsafe {
            PROCESS_LOAD = if current_tick_count - tick_count == 0 {
                0
            } else {
                100 - (current_idle_count - idle_count) * 100 / (current_tick_count - tick_count)
            }
        };
        idle_count = current_idle_count;
        tick_count = current_tick_count;
        halting(unsafe { PROCESS_LOAD });
        if interrupt::without_interrupt(|| SCHEDULER.lock().wait.count()) != 0 {
            while let Some(wait) =
                interrupt::without_interrupt(|| SCHEDULER.lock().wait.pop_front())
            {
                println!("IDLE: Task ID [0x{wait:X}] ended");
                PROCESS_POOL.lock().dealloc(wait);
            }
        }
        yield_next();
    }
}

fn halting(load: u64) {
    if load < 40 {
        assembly::halt();
        assembly::halt();
        assembly::halt();
    } else if load < 80 {
        assembly::halt();
        assembly::halt();
    } else if load < 95 {
        assembly::halt();
    }
}

pub fn process_load() -> u64 {
    unsafe { PROCESS_LOAD }
}
