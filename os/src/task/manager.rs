//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::binary_heap::BinaryHeap;
use alloc::sync::Arc;
use core::cmp::Ordering;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: BinaryHeap<StrideTask>,
}

// #[derive(Debug)]
pub struct StrideTask(Arc<TaskControlBlock>);

impl PartialOrd for StrideTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrideTask {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_stride = self.0.inner().stride;
        let other_stride = other.0.inner().stride;

        let diff = self_stride.wrapping_sub(other_stride) as i8;
        if diff < 0 {
            Ordering::Less.reverse()
        } else if diff > 0 {
            Ordering::Greater.reverse()
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for StrideTask {
    fn eq(&self, other: &Self) -> bool {
        self.0.inner().stride == other.0.inner().stride
    }
}

impl Eq for StrideTask {}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(StrideTask(task));
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop().map(|s| s.0)
    }
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
