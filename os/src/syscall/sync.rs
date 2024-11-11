use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    if process_inner.deadlock_detect_enabled && mutex.is_locking() {
        return -0xDEAD;
    }
    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let current_tid = get_tid();
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    if process_inner.deadlock_detect_enabled {
        // allocate resource
        let current_task = current_task().unwrap();
        let mut current_task_inner = current_task.inner_exclusive_access();
        let allocation = &mut current_task_inner.allocated;
        if allocation.len() <= sem_id {
            allocation.resize(sem_id + 1, 0);
        }
        allocation[sem_id] += 1;
    }
    drop(process_inner);
    sem.up(current_tid, sem_id);
    0
}
fn get_tid() -> usize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let current_tid = get_tid();
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let detecting = process_inner.deadlock_detect_enabled;
    if detecting {
        // 查询当前进程下所有线程的资源和请求情况
        let resourse_num = process_inner.semaphore_list.len();

        let mut work = alloc::vec![0;resourse_num];
        for (sem_id, sem) in process_inner.semaphore_list.iter().enumerate() {
            if let Some(sem) = sem {
                let count = sem.get_count().max(0) as usize;
                work[sem_id] = count;
            }
        }

        let tasks_num = process_inner.tasks.len();
        let mut allocations = alloc::vec![alloc::vec![0; resourse_num]; tasks_num];
        let mut requests = process_inner.requests.clone();
        requests.resize(tasks_num, alloc::vec![]);
        for req in requests.iter_mut() {
            req.resize(resourse_num, 0);
        }
        let mut finish = alloc::vec![false; tasks_num];

        for (tid, task) in process_inner.tasks.iter().enumerate() {
            let Some(task) = task else {
                continue;
            };

            let task = Arc::clone(task);
            let task_inner = task.inner_exclusive_access();
            if task_inner.res.is_none() {
                continue;
            }
            for (sem_id, sem_alloc) in task_inner.allocated.iter().enumerate() {
                allocations[tid][sem_id] = *sem_alloc;
            }
        }

        let current_task = current_task().unwrap();
        let mut current_task_inner = current_task.inner_exclusive_access();
        // 当前线程请求资源
        requests[current_tid][sem_id] += 1;

        let mut change = true;
        while change {
            change = false;
            for (tid, finished) in finish.iter_mut().enumerate() {
                let mut enough = true;
                for (req, work) in requests[tid].iter().zip(work.iter()) {
                    if *req > *work {
                        enough = false;
                        break;
                    }
                }
                if !*finished && enough {
                    *finished = true;
                    change = true;
                    for (work, alloc) in work.iter_mut().zip(allocations[tid].iter()) {
                        *work += *alloc;
                    }
                }
            }
        }

        for is_finished in finish {
            if !is_finished {
                return -0xDEAD;
            }
        }

        // allocate resource

        let allocation = &mut current_task_inner.allocated;
        if allocation.len() <= sem_id {
            allocation.resize(sem_id + 1, 0);
        }
        allocation[sem_id] += 1;
    }
    let sem = process_inner.semaphore_list[sem_id]
        .as_ref()
        .map(Arc::clone)
        .unwrap();
    drop(process_inner);
    sem.down(current_tid, sem_id);
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();

    process_inner.deadlock_detect_enabled = true;

    0
}
