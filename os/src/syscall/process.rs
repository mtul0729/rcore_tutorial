//! Process management syscalls
use core::mem::size_of;
use core::ptr;

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{translated_byte_buffer, MapPermission, VirtAddr},
    task::{
        append_map_area, change_program_brk, current_user_token, exit_current_and_run_next,
        get_current_task_status, get_init_time, get_syscall_times, remove_map_area,
        suspend_current_and_run_next, TaskStatus,
    },
    timer::{get_time_ms, get_time_us},
};

// 16字节对齐，避免跨页
#[repr(C, align(16))]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let timeval = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let mut src = &timeval as *const _ as *const u8;
    let bufs = translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    for buf in bufs {
        let buf_len = buf.len();
        unsafe {
            ptr::copy_nonoverlapping(src, buf as *mut _ as *mut u8, size_of::<TimeVal>());
            src = src.add(buf_len);
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");

    let taskinfo = TaskInfo {
        status: get_current_task_status(),
        syscall_times: get_syscall_times(),
        time: get_time_ms() - get_init_time(),
    };
    let mut src = &taskinfo as *const _ as *const u8;
    let bufs = translated_byte_buffer(current_user_token(), ti as *const _, size_of::<TaskInfo>());
    for buf in bufs {
        let buf_len = buf.len();
        unsafe {
            ptr::copy_nonoverlapping(src, buf as *mut _ as *mut u8, buf_len);
            src = src.add(buf_len);
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if (port & !0x7 != 0) || (port & 0x7 == 0) {
        return -1;
    }
    let port = port << 1;
    let permission = MapPermission::from_bits(port as u8).unwrap() | MapPermission::U;
    let start_va: VirtAddr = start.into();
    if !start_va.aligned() {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    match append_map_area(start_va, (start + len).into(), permission) {
        Ok(_) => 0,
        _ => -1,
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    // 找到完全对应的Maparea并删除，返回0，否则返回-1
    match remove_map_area(start.into(), (start + len).into()) {
        Ok(_) => 0,
        _ => -1,
    }
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
