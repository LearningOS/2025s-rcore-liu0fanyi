//! Process management syscalls
use crate::{
    mm::translated,
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, mmap, munmap,
        read_sys_call_counter, suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    // trace!("kernel: sys_get_time");
    // -1

    let phy_ts = translated::<TimeVal>(current_user_token(), ts as usize, 0).unwrap();

    let us = get_time_us();
    *phy_ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
// pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
//     trace!("kernel: sys_trace");
//     -1
// }

pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    let mut phy_addr = None;
    if trace_request == 0 || trace_request == 1 {
        phy_addr = translated::<isize>(current_user_token(), id, trace_request);
        println!("phy_a :{:?}", phy_addr);
        if phy_addr.is_none() {
            return -1;
        }
    }

    match trace_request {
        // 0 => unsafe { *(id as *const u8) as isize },
        0 => {
            return *phy_addr.unwrap();
        }
        1 => {
            *(phy_addr.unwrap()) = data as isize;
            0
        }
        2 => read_sys_call_counter(id),
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    mmap(start, len, prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    munmap(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
