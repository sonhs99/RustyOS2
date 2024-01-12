use crate::{print, println};

use super::{get_process_from_id, Scheduler, PROCESS_MAXCOUNT};

const PROCESS_TIME: i64 = 5;
const PROCESS_READYLISTCOUNT: usize = 5;

pub const PRIORITY_HIGHIST: u64 = 0;
pub const PRIORITY_MIDDLE: u64 = 2;
pub const PRIORITY_LOWIST: u64 = 4;
pub const PRIORITY_WAIT: u64 = 0xFF;

static mut RUNQUEUE_POOL: [Option<u64>; PROCESS_MAXCOUNT] = [None; PROCESS_MAXCOUNT];

pub fn get_priority(flag: u64) -> u64 {
    flag & 0xFF
}

pub(crate) fn set_priority(flag: &mut u64, priority: u64) {
    *flag = (*flag & !0xFF) | priority;
}

#[derive(Clone, Copy)]
pub(crate) struct RunQueue {
    head: Option<u64>,
    tail: Option<u64>,
    count: u64,
}

pub struct RRScheduler {
    running: u64,
    processor_time: i64,
    pub(crate) wait: RunQueue,
    ready: [RunQueue; PROCESS_READYLISTCOUNT],
    execute_count: [u64; PROCESS_READYLISTCOUNT],
}

impl RRScheduler {
    pub const fn new(run_id: u64) -> Self {
        Self {
            running: run_id,
            processor_time: PROCESS_TIME,
            wait: RunQueue::new(),
            ready: [RunQueue::new(); PROCESS_READYLISTCOUNT],
            execute_count: [0; PROCESS_READYLISTCOUNT],
        }
    }
}

impl Scheduler for RRScheduler {
    fn running(&self) -> u64 {
        self.running
    }

    fn next(&mut self) -> Option<u64> {
        for _ in 0..2 {
            for idx in 0..PROCESS_READYLISTCOUNT {
                let len = self.ready[idx].count();
                if len > self.execute_count[idx] {
                    self.execute_count[idx] += 1;
                    return self.ready[idx].pop_front();
                } else {
                    self.execute_count[idx] = 0;
                }
            }
        }
        None
    }

    fn set_running(&mut self, run_id: u64) {
        self.running = run_id;
    }

    fn add_ready_list(&mut self, ready_id: u64) -> Result<(), ()> {
        if let Some(ready) = get_process_from_id(ready_id) {
            let priority = get_priority(ready.flags);
            if priority == PRIORITY_WAIT {
                // println!("{:?}", unsafe { RUNQUEUE_POOL[ready_id as usize] });
                // self.ready[4].print();
                self.wait.push_back(ready_id);
            } else if priority < PROCESS_READYLISTCOUNT as u64 {
                self.ready[priority as usize].push_back(ready_id);
            } else {
                return Err(());
            }
        }
        Ok(())
    }

    fn total_count(&self) -> u64 {
        self.ready.iter().fold(0, |acc, x| acc + x.count())
    }

    fn remove_process(&mut self, pid: u64) -> Result<u64, ()> {
        if let Some(process) = super::get_process_from_id(pid) {
            if pid != process.id {
                let priority = get_priority(process.flags);
                self.ready[priority as usize].remove(pid)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    fn change_priority(&mut self, pid: u64, priority: u64) -> Result<(), ()> {
        if let Some(process) = get_process_from_id(pid) {
            set_priority(&mut process.flags, priority);
            if let Ok(_) = self.remove_process(pid) {
                self.add_ready_list(pid)?;
            }
            Ok(())
        } else {
            Err(())
        }
    }

    fn reset_processtime(&mut self) {
        self.processor_time = PROCESS_TIME;
    }
    fn decrease_time(&mut self) {
        if self.processor_time > 0 {
            self.processor_time -= 1;
        }
    }

    fn is_expired(&self) -> bool {
        self.processor_time <= 0
    }
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            count: 0,
        }
    }

    pub fn push_back(&mut self, node: u64) {
        if let Some(process) = self.tail {
            self.tail = Some(node);
            unsafe { RUNQUEUE_POOL[process as usize] = self.tail };
        } else {
            self.head = Some(node);
            self.tail = Some(node);
        }
        self.count += 1;
    }

    pub fn pop_front(&mut self) -> Option<u64> {
        if let Some(process) = self.head {
            if process == self.tail.unwrap() {
                self.head = None;
                self.tail = None;
            } else {
                unsafe { self.head = RUNQUEUE_POOL[process as usize] };
            }
            unsafe { RUNQUEUE_POOL[process as usize] = None };
            self.count -= 1;
            Some(process)
        } else {
            None
        }
    }

    pub const fn count(&self) -> u64 {
        self.count
    }

    pub fn remove(&mut self, pid: u64) -> Result<u64, ()> {
        let node = match self.head {
            Some(head) => {
                if head == pid {
                    self.pop_front();
                    return Ok(pid);
                } else {
                    let mut current = head;
                    while let Some(next) = unsafe { RUNQUEUE_POOL[current as usize] } {
                        if next == pid {
                            break;
                        } else if next == self.tail.unwrap() {
                            return Err(());
                        }
                        current = next;
                    }
                    current
                }
            }
            None => return Err(()),
        };

        if self.tail.unwrap() == pid {
            self.tail = Some(node);
        }
        unsafe {
            let next = RUNQUEUE_POOL[node as usize].unwrap();
            RUNQUEUE_POOL[node as usize] = RUNQUEUE_POOL[next as usize];
            RUNQUEUE_POOL[next as usize] = None;
        };

        Ok(pid)
    }
}
