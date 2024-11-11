//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{
    block_current_and_run_next, current_process, current_task, wakeup_task, TaskControlBlock,
};
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore structure
pub struct Semaphore {
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    /// tid and tcb
    pub wait_queue: VecDeque<(usize, Arc<TaskControlBlock>)>,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self, _tid: usize, sem_id: usize) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        inner.count += 1;
        if inner.count <= 0 {
            if let Some((tid, task)) = inner.wait_queue.pop_front() {
                wakeup_task(task);
                let process = current_process();
                process.request_up(tid, sem_id);
            }
        }
    }

    /// down operation of semaphore
    pub fn down(&self, tid: usize, sem_id: usize) {
        trace!("kernel: Semaphore::down");
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        let process = current_process();
        if inner.count < 0 {
            println!(
                "request added: tid {},sem_id {}, sem_count {}",
                tid, sem_id, inner.count
            );
            process.request_down(tid, sem_id);
            inner.wait_queue.push_back((tid, current_task().unwrap()));
            drop(inner);
            block_current_and_run_next();
        } else {
            let task = process.get_task(tid).unwrap();
            let mut current_task_inner = task.inner_exclusive_access();
            let allocation = &mut current_task_inner.allocated;
            if allocation.len() <= sem_id {
                allocation.resize(sem_id + 1, 0);
            }
            allocation[sem_id] += 1;
        }
    }
    /// get the count of semaphore
    pub fn get_count(&self) -> isize {
        let inner = self.inner.exclusive_access();
        inner.count
    }
}
