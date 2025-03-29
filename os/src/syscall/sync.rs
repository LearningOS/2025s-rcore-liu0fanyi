use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore, RES_MANAGER};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;

macro_rules! with_deadlock_detect {
    ($res_mgr:ident, $code:block) => {{
        // if GLOBAL_DETECT_FLAG.load(core::sync::atomic::Ordering::Relaxed) {
        let mut $res_mgr = RES_MANAGER.exclusive_access();
        $code
    }};
}

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
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid,
    );
    let process = current_process();

    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };

    let mut process_inner = process.inner_exclusive_access();

    let id = if let Some(id) = process_inner
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
    };

    if process_inner.is_deadlock_detect {
        with_deadlock_detect! {res_mgr,{
            // info!("mutex create:id:{}, tid:{}", id, tid);
            res_mgr.add_resource(id as usize, tid, crate::sync::ResType::Mutex, 1);
        }};
    }
    id
}

// ch8_deadlock_mutex1

/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid,
    );

    if process_inner.is_deadlock_detect {
        with_deadlock_detect! {res_mgr,{
            // info!("mutex lock tid:{}, mutex_id: {}", tid, mutex_id);
            res_mgr.need[tid][mutex_id] += 1;

            if !res_mgr.is_safe(tid, mutex_id) {
                // info!("unsafe :tid: {}, mutex_id: {}", tid, mutex_id);
                res_mgr.need[tid][mutex_id] -= 1;
                return -0xDEAD;
            }
        }};
    }

    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.lock();

    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    if process_inner.is_deadlock_detect {
        with_deadlock_detect! {res_mgr,{
            res_mgr.allocation[tid][mutex_id] += 1;
            res_mgr.need[tid][mutex_id] -= 1;
            res_mgr.available[mutex_id] -= 1;
        }};
    }

    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();

    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    if process_inner.is_deadlock_detect {
        with_deadlock_detect! {res_mgr,{
            res_mgr.allocation[tid][mutex_id] -= 1;
            res_mgr.available[mutex_id] += 1;
        }};
    }

    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
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

    if process_inner.is_deadlock_detect {
        with_deadlock_detect! {res_mgr,{
            // info!("add Semaphore resource:id:{}-tid:{}", id, tid);
            res_mgr.add_resource(id as usize, tid, crate::sync::ResType::Semaphore, res_count);
        }};
    }

    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );

    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.up();

    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    if process_inner.is_deadlock_detect {
        with_deadlock_detect! {res_mgr,{
            // info!("up: tid:{}, sem_id:{}, allocation:{:?}", tid, sem_id, res_mgr.allocation);
            res_mgr.allocation[tid][sem_id] -= 1;
            res_mgr.available[sem_id] += 1;
        }};
    }
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    if process_inner.is_deadlock_detect {
        with_deadlock_detect! {res_mgr,{
            info!("sem down :sem_id: {}, tid: {}", sem_id, tid);
            res_mgr.need[tid][sem_id] += 1;

            if !res_mgr.is_safe(tid, sem_id) {
                // info!("sem down not safe ?tid: {}, sem_id: {}", tid, sem_id);
                res_mgr.need[tid][sem_id] -= 1;
                return -0xDEAD;
            }
        }};
    }

    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    // info!("!!!!+down before?:sem_id{},tid:{}", sem_id, tid);
    sem.down();

    // info!("!!!!=down done?:sem_id{},tid:{}", sem_id, tid);
    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    if process_inner.is_deadlock_detect {
        with_deadlock_detect! {res_mgr,{
            res_mgr.allocation[tid][sem_id] += 1;
            res_mgr.available[sem_id] -= 1;
            res_mgr.need[tid][sem_id] -= 1;
        }};
    }
    // info!("!!!!-down done2?:sem_id{},tid:{}", sem_id, tid);

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
// static GLOBAL_DETECT_FLAG: AtomicBool = AtomicBool::new(false);

/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.is_deadlock_detect = enabled != 0;
    0
}
