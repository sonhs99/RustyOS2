use core::{hint::black_box, mem::size_of, ptr::NonNull};

use spin::{Lazy, Mutex};

use crate::{
    descriptor::{GDT_KERNELCODESEGMENT, GDT_KERNELDATASEGMENT, IST_SIZE, IST_STARTADDRESS},
    interrupt, println,
    utility::{memcpy, memset},
};

use self::round_robin::{RRScheduler, PRIORITY_WAIT};

pub use idle::{idle_process, process_load};
pub use round_robin::{get_priority, PRIORITY_HIGHIST, PRIORITY_LOWIST, PRIORITY_MIDDLE};

mod idle;
mod round_robin;

const PROCESS_REGISTERCOUNT: usize = 5 + 19;
pub(crate) const PROCESS_MAXCOUNT: usize = 1024;
const PROCESS_POOLADDRESS: u64 = 0x800000;

const PROCESS_STACKADDRESS: u64 =
    PROCESS_POOLADDRESS + (size_of::<Process>() * PROCESS_MAXCOUNT) as u64;
const PROCESS_STACKSIZE: u64 = 8192;

const PROCESS_INVALIDID: u64 = 0xFFFFFFFFFFFFFFFF;

// flags
const PROCESS_FLAG_ENDTASK: u64 = 0x8000000000000000;
pub const PROCESS_FLAG_IDLETASK: u64 = 0x0800000000000000;

#[repr(C, packed(1))]
pub struct Context {
    pub(crate) registers: [u64; PROCESS_REGISTERCOUNT],
}

extern "C" {
    pub fn context_switch(current: &Context, next: &Context);
}

pub struct Process {
    pub context: Context,
    pub id: u64,
    pub flags: u64,

    stack: u64,
    stack_size: u64,
}

pub struct ProcessPool<'a> {
    pool: &'a mut [Process; PROCESS_MAXCOUNT],
    max_count: usize,
    use_count: usize,
    alloc_count: usize,
}

impl Process {
    const GS: usize = 0;
    const FS: usize = 1;
    const ES: usize = 2;
    const DS: usize = 3;
    const RBP: usize = 18;
    const RIP: usize = 19;
    const CS: usize = 20;
    const RFLAGS: usize = 21;
    const RSP: usize = 22;
    const SS: usize = 23;

    pub fn new(flags: u64, entry_point: u64, stack: u64, stack_size: u64) -> Self {
        let mut process = Self {
            context: Context::empty(),
            id: 0,
            flags,
            stack,
            stack_size,
        };
        process.set(flags, entry_point, stack, stack_size);
        process
    }

    pub fn set(&mut self, flags: u64, entry_point: u64, stack: u64, stack_size: u64) {
        memset(
            &mut self.context as *mut Context as *mut u8,
            0,
            size_of::<Context>() as isize,
        );
        self.context.registers[Process::RSP] = stack + stack_size;
        self.context.registers[Process::RBP] = stack + stack_size;

        self.context.registers[Process::CS] = GDT_KERNELCODESEGMENT as u64;
        self.context.registers[Process::DS] = GDT_KERNELDATASEGMENT as u64;
        self.context.registers[Process::ES] = GDT_KERNELDATASEGMENT as u64;
        self.context.registers[Process::FS] = GDT_KERNELDATASEGMENT as u64;
        self.context.registers[Process::GS] = GDT_KERNELDATASEGMENT as u64;
        self.context.registers[Process::SS] = GDT_KERNELDATASEGMENT as u64;

        self.context.registers[Process::RIP] = entry_point;

        self.context.registers[Process::RFLAGS] |= 0x0200;

        self.flags = flags;
        self.stack = stack;
        self.stack_size = stack_size;
    }
}

impl Context {
    pub const fn empty() -> Self {
        Self {
            registers: [0u64; PROCESS_REGISTERCOUNT],
        }
    }
}

impl<'a> ProcessPool<'a> {
    unsafe fn new(address: u64) -> Self {
        let address = address as *mut [Process; PROCESS_MAXCOUNT];
        memset(
            address as *mut u8,
            0,
            (size_of::<Process>() * PROCESS_MAXCOUNT) as isize,
        );
        for (idx, process) in (*address).iter_mut().enumerate() {
            process.id = idx as u64;
        }
        Self {
            pool: &mut *address,
            max_count: PROCESS_MAXCOUNT,
            use_count: 0,
            alloc_count: 1,
        }
    }

    pub fn get(&mut self, idx: usize) -> *mut Process {
        &mut self.pool[idx] as *mut Process
    }

    pub fn alloc(&mut self) -> Option<*mut Process> {
        let mut i = 0;
        if self.max_count == self.use_count {
            return None;
        }
        for (idx, process) in self.pool.iter().enumerate() {
            if (process.id >> 32) == 0 {
                i = idx;
                break;
            }
        }
        let target = &mut self.pool[i];
        target.id = ((self.alloc_count << 32) | i) as u64;
        self.alloc_count = self.alloc_count.overflowing_add(1).0;
        self.use_count += 1;
        Some(target as *mut Process)
    }

    pub fn dealloc(&mut self, id: u64) -> Option<()> {
        let idx = id & 0xFFFFFFFF;
        memset(
            &mut self.pool[idx as usize].context as *mut Context as *mut u8,
            0,
            size_of::<Context>() as isize,
        );
        self.pool[idx as usize].id = idx;
        self.use_count -= 0;
        Some(())
    }
}

unsafe impl<'a> Send for ProcessPool<'a> {}

pub(crate) static PROCESS_POOL: Lazy<Mutex<ProcessPool>> =
    Lazy::new(|| Mutex::new(unsafe { ProcessPool::new(PROCESS_POOLADDRESS) }));

pub trait Scheduler {
    fn next(&mut self) -> Option<u64>;
    fn add_ready_list(&mut self, pid: u64) -> Result<(), ()>;
    fn running(&self) -> u64;
    fn set_running(&mut self, pid: u64);
    fn decrease_time(&mut self);
    fn reset_processtime(&mut self);
    fn change_priority(&mut self, pid: u64, priority: u64) -> Result<(), ()>;
    fn is_expired(&self) -> bool;
    fn remove_process(&mut self, pid: u64) -> Result<u64, ()>;
    fn total_count(&self) -> u64;
}

pub(crate) static SCHEDULER: Lazy<Mutex<RRScheduler>> = Lazy::new(|| {
    let first = unsafe { &mut *PROCESS_POOL.lock().alloc().unwrap() };
    first.flags = PRIORITY_HIGHIST;
    Mutex::new(RRScheduler::new(first.id & 0xFFFFFFFF))
});

pub fn init_scheduler() {
    black_box(SCHEDULER.lock());
}

pub fn create_task(flags: u64, entry: u64) -> Result<u64, ()> {
    if let Some(process) = PROCESS_POOL.lock().alloc() {
        let pid = unsafe { (*process).id & 0xFFFFFFFF };
        let stack_address = PROCESS_STACKADDRESS + (PROCESS_STACKSIZE * pid as u64);
        unsafe {
            (*process).set(flags, entry, stack_address, PROCESS_STACKSIZE);
            if let Err(_) = interrupt::without_interrupt(|| SCHEDULER.lock().add_ready_list(pid)) {
                return Err(());
            }
        }
        Ok(pid)
    } else {
        Err(())
    }
}

pub fn schedule() {
    unsafe {
        if let Some(next_id) = interrupt::without_interrupt(|| SCHEDULER.lock().next()) {
            let context_address =
                (IST_STARTADDRESS + IST_SIZE) as u64 - size_of::<Context>() as u64;
            let current_id = interrupt::without_interrupt(|| SCHEDULER.lock().running());
            let current = get_process_from_id(current_id).unwrap();
            let next = get_process_from_id(next_id).unwrap();

            interrupt::without_interrupt(|| SCHEDULER.lock().set_running(next_id));

            if current.flags & PROCESS_FLAG_ENDTASK == 0 {
                if current.flags & PROCESS_FLAG_IDLETASK != 0 {
                    idle::IDLE_COUNT += 1;
                }
                idle::TICK_COUNT += 1;

                memcpy(
                    &mut current.context as *mut Context as *mut u8,
                    context_address as *mut u8,
                    size_of::<Context>() as isize,
                );
            }

            interrupt::without_interrupt(|| SCHEDULER.lock().add_ready_list(current_id));

            memcpy(
                context_address as *mut u8,
                &mut next.context as *mut Context as *mut u8,
                size_of::<Context>() as isize,
            );
        }
        interrupt::without_interrupt(|| SCHEDULER.lock().reset_processtime());
    }
}

pub fn yield_next() {
    if let Some(next_id) = interrupt::without_interrupt(|| SCHEDULER.lock().next()) {
        let current_id = interrupt::without_interrupt(|| SCHEDULER.lock().running());
        let current = get_process_from_id(current_id).unwrap();
        let next = get_process_from_id(next_id).unwrap();

        interrupt::without_interrupt(|| SCHEDULER.lock().set_running(next_id));
        interrupt::without_interrupt(|| SCHEDULER.lock().add_ready_list(current_id));
        if current.flags & PROCESS_FLAG_ENDTASK != 0 {
            unsafe { context_switch(&*(0 as *const Context), &next.context) };
        } else {
            if current.flags & PROCESS_FLAG_IDLETASK != 0 {
                unsafe { idle::IDLE_COUNT += 1 };
            }
            unsafe { idle::TICK_COUNT += 1 };
            unsafe { context_switch(&current.context, &next.context) };
        }
    }
    interrupt::without_interrupt(|| SCHEDULER.lock().reset_processtime());
}

pub fn decrease_time() {
    interrupt::without_interrupt(|| SCHEDULER.lock().decrease_time())
}

pub fn is_expired() -> bool {
    interrupt::without_interrupt(|| SCHEDULER.lock().is_expired())
}

pub fn get_pid() -> u64 {
    interrupt::without_interrupt(|| SCHEDULER.lock().running())
}

pub fn get_process_from_id<'a>(pid: u64) -> Option<&'a mut Process> {
    let pid = pid & 0xFFFFFFFF;
    if pid > PROCESS_MAXCOUNT as u64 {
        None
    } else {
        unsafe {
            let address = &mut *(PROCESS_POOLADDRESS as *mut [Process; PROCESS_MAXCOUNT]);
            Some(&mut address[pid as usize])
        }
    }
}

pub fn end_process(pid: u64) {
    let target = get_process_from_id(pid);
    if interrupt::without_interrupt(|| SCHEDULER.lock().running()) == pid {
        let target = target.unwrap();
        target.flags |= PROCESS_FLAG_ENDTASK;
        round_robin::set_priority(&mut target.flags, PRIORITY_WAIT);
        yield_next();
        loop {}
    } else if let Some(target) = target {
        interrupt::without_interrupt(|| SCHEDULER.lock().remove_process(pid));
        target.flags |= PROCESS_FLAG_ENDTASK;
        round_robin::set_priority(&mut target.flags, PRIORITY_WAIT);
        interrupt::without_interrupt(|| SCHEDULER.lock().add_ready_list(pid));
    }
}

pub fn change_priority(pid: u64, priority: u64) -> Result<(), ()> {
    interrupt::without_interrupt(|| SCHEDULER.lock().change_priority(pid, priority))
}

pub fn exit() {
    end_process(interrupt::without_interrupt(|| SCHEDULER.lock().running()));
}

pub fn process_count() -> u64 {
    interrupt::without_interrupt(|| SCHEDULER.lock().total_count())
}

pub fn is_process_exist(pid: u64) -> bool {
    match get_process_from_id(pid) {
        Some(process) => process.id != pid,
        None => false,
    }
}
