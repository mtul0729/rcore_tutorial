//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }

    // =======
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch4, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let next_task = &mut inner.tasks[0];
        next_task.task_status = TaskStatus::Running;
        next_task.init_time = get_time_ms();
        let next_task_cx_ptr = &next_task.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut _, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Ready;
        // =======
        /// Generally, the first task in task list is an idle task (we call it zero process later).
        /// But in ch4, we load apps statically, so the first task is a real app.
        fn run_first_task(&self) -> ! {
            let mut inner = self.inner.exclusive_access();
            let next_task = &mut inner.tasks[0];
            next_task.task_status = TaskStatus::Running;
            next_task.init_time = get_time_ms();
            let next_task_cx_ptr = &next_task.task_cx as *const TaskContext;
            drop(inner);
            let mut _unused = TaskContext::zero_init();
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(&mut _unused as *mut _, next_task_cx_ptr);
            }
            panic!("unreachable in run_first_task!");
        }

        /// Change the status of current `Running` task into `Ready`.
        fn mark_current_suspended(&self) {
            let mut inner = self.inner.exclusive_access();
            let cur = inner.current_task;
            inner.tasks[cur].task_status = TaskStatus::Ready;
        }

        /// Change the status of current `Running` task into `Exited`.
        fn mark_current_exited(&self) {
            let mut inner = self.inner.exclusive_access();
            let cur = inner.current_task;
            inner.tasks[cur].task_status = TaskStatus::Exited;
        }

        /// Find next task to run and return task id.
        ///
        /// In this case, we only return the first `Ready` task in task list.
        fn find_next_task(&self) -> Option<usize> {
            let inner = self.inner.exclusive_access();
            let current = inner.current_task;
            (current + 1..current + self.num_app + 1)
                .map(|id| id % self.num_app)
                .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
        }

        /// Get the current 'Running' task's token.
        fn get_current_token(&self) -> usize {
            let inner = self.inner.exclusive_access();
            inner.tasks[inner.current_task].get_user_token()
        }

        /// Get the current 'Running' task's trap contexts.
        fn get_current_trap_cx(&self) -> &'static mut TrapContext {
            let inner = self.inner.exclusive_access();
            inner.tasks[inner.current_task].get_trap_cx()
        }

        /// Change the current 'Running' task's program break
        pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
            let mut inner = self.inner.exclusive_access();
            let cur = inner.current_task;
            inner.tasks[cur].change_program_brk(size)
        }

        /// Switch current `Running` task to the task we have found,
        /// or there is no `Ready` task and we can exit with all applications completed
        fn run_next_task(&self) {
            if let Some(next) = self.find_next_task() {
                let mut inner = self.inner.exclusive_access();
                let current = inner.current_task;
                inner.tasks[next].task_status = TaskStatus::Running;
                if inner.tasks[next].init_time == 0 {
                    inner.tasks[next].init_time = get_time_ms();
                }
                inner.current_task = next;
                let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
                let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
                drop(inner);
                // before this, we should drop local variables that must be dropped manually
                unsafe {
                    __switch(current_task_cx_ptr, next_task_cx_ptr);
                }
                // go back to user mode
            } else {
                panic!("All applications completed!");
            }
        }
        fn get_current_task_status(&self) -> TaskStatus {
            let inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[current].task_status
        }
        fn count_syscall(&self, syscall_id: usize) {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[current].syscall_times[syscall_id] += 1;
        }
        fn get_syscall_times(&self) -> [u32; MAX_SYSCALL_NUM] {
            let inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[current].syscall_times
        }
        fn get_init_time(&self) -> usize {
            let inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[current].init_time
        }
        fn append_map_area(
            &self,
            start_va: VirtAddr,
            end_va: VirtAddr,
            permission: MapPermission,
        ) -> Result<(), MemErr> {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            let memory_set = &mut inner.tasks[current].memory_set;
            memory_set.insert_framed_area(start_va, end_va, permission)
        }
        fn remove_map_area(&self, start_va: VirtAddr, end_va: VirtAddr) -> Result<(), MemErr> {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            let memory_set = &mut inner.tasks[current].memory_set;
            memory_set.remove_area(start_va, end_va)
        }
        // >>>>>>> 193ba5f (rebase)
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// Change the current 'Running' task's program break
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            if inner.tasks[next].init_time == 0 {
                inner.tasks[next].init_time = get_time_ms();
            }
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }
    fn get_current_task_status(&self) -> TaskStatus {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status
    }
    fn count_syscall(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times[syscall_id] += 1;
    }
    fn get_syscall_times(&self) -> [u32; MAX_SYSCALL_NUM] {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times
    }
    fn get_init_time(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].init_time
    }
    fn append_map_area(
        &self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) -> Result<(), MemErr> {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let memory_set = &mut inner.tasks[current].memory_set;
        memory_set.insert_framed_area(start_va, end_va, permission)
    }
    fn remove_map_area(&self, start_va: VirtAddr, end_va: VirtAddr) -> Result<(), MemErr> {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let memory_set = &mut inner.tasks[current].memory_set;
        memory_set.remove_area(start_va, end_va)
    }
    // >>>>>>> 193ba5f (rebase)
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}

/// Get the status of the current 'Running' task
pub fn get_current_task_status() -> TaskStatus {
    TASK_MANAGER.get_current_task_status()
}

/// Increase syscall counter of the current 'Running' task
pub fn count_syscall(syscall_id: usize) {
    TASK_MANAGER.count_syscall(syscall_id);
}

/// Get syscall times
pub fn get_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    TASK_MANAGER.get_syscall_times()
}

/// Get the init_time of the current 'Running' task
pub fn get_init_time() -> usize {
    TASK_MANAGER.get_init_time()
}

/// Append a MapArea to current 'Running' task
pub fn append_map_area(
    start_va: VirtAddr,
    end_va: VirtAddr,
    permission: MapPermission,
) -> Result<(), MemErr> {
    TASK_MANAGER.append_map_area(start_va, end_va, permission)
}

/// Remove a MapArea to current 'Running' task
pub fn remove_map_area(start_va: VirtAddr, end_va: VirtAddr) -> Result<(), MemErr> {
    TASK_MANAGER.remove_map_area(start_va, end_va)
}
