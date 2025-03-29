//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

use alloc::vec::Vec;
pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;

// use lazy_static::*;

// lazy_static! {
//     /// sync/resource_manager.rs
//     pub static ref RES_MANAGER: UPSafeCell<ResourceManager> =
//         unsafe { UPSafeCell::new(ResourceManager::new()) };
// }

/// ResType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResType {
    /// 互斥
    Mutex = 0,
    /// 信号量
    Semaphore = 1,
}

// #[macro_export]
macro_rules! vec {
    // 匹配 vec![1, 2, 3] 这种形式
    ($($elem:expr),* $(,)?) => {
        {
            let mut v = Vec::new();
            $( v.push($elem); )*
            v
        }
    };

    // 匹配 vec![0; 5] 这种形式
    ($elem:expr; $count:expr) => {
        {
            let count = $count; // 避免多次计算
            let mut v = Vec::with_capacity(count);
            v.resize(count, $elem);
            v
        }
    };
}

/// ResourceManager
pub struct ResourceManager {
    /// 可用资源向量 Available[m] (m为资源类型数量)
    pub available: Vec<i32>,

    // n线程已经分配的资源数
    /// 分配矩阵 Allocation[n][m] (n=线程数, m=资源数)
    pub allocation: Vec<Vec<usize>>,

    // n线程还需要的资源
    /// 需求矩阵 need[tid][res_id] = 数量
    pub need: Vec<Vec<usize>>,

    /// 资源类型标记 (0=mutex, 1=semaphore)
    pub res_types: Vec<ResType>,
}

impl ResourceManager {
    /// new
    pub fn new() -> Self {
        Self {
            available: Vec::new(),
            allocation: Vec::new(),
            need: Vec::new(),
            res_types: Vec::new(),
        }
    }

    /// 添加新资源时调用
    pub fn add_resource(&mut self, res_id: usize, tid: usize, res_type: ResType, total: usize) {
        // 确保res_id在向量范围内
        if res_id >= self.available.len() {
            self.available.resize(res_id + 1, 0);
            self.res_types.resize(res_id + 1, ResType::Mutex);
        }

        if res_id >= self.allocation[tid].len() {
            self.allocation[tid].resize(res_id + 1, 0);
            self.need[tid].resize(res_id + 1, 0);

            info!(
                "add_resource tid allocation need update, tid:{}, allocation:{}, need:{}",
                tid,
                self.allocation[tid].len(),
                self.need[tid].len(),
            );
        }

        self.res_types[res_id] = res_type;
        self.available[res_id] = total as i32;
        info!(
            "init res_id:{}, tid:{}, type:{:?}, available:{}",
            res_id, tid, self.res_types[res_id], self.available[res_id]
        );
    }

    /// 添加新线程的时候初始化
    pub fn add_thread(&mut self, tid: usize) {
        info!("tid allocation before:{}", tid);
        if tid >= self.allocation.len() {
            // let m = self.available.len();
            let m = self.available.len() + 1;
            self.allocation.resize(tid + 1, vec![0; m]);
            self.need.resize(tid + 1, vec![0; m]);
            info!(
                "tid allocation, tid:{}, allocation:{}, need:{}",
                tid,
                self.allocation[tid].len(),
                self.need[tid].len(),
            );
        }
    }

    /// 安全检查
    pub fn is_safe(&self, _tid: usize, res_id: usize) -> bool {
        // 克隆当前状态用于模拟
        let mut work = self.available.clone();
        let alloc = self.allocation.clone();
        let need = self.need.clone();

        // 预检查资源是否足够
        if self.res_types[res_id] == ResType::Mutex && work[res_id] == 0 {
            return false;
        }

        // info!(
        //     "after tid:{}, id:{}, work:{:?}, alloc:{:?}, need:{:?}",
        //     tid, res_id, work, alloc, need,
        // );

        // 初始化完成标记, false代表没有完成
        let mut finish = vec![false; self.allocation.len()];

        // 银行家算法核心
        loop {
            let mut found = false;

            // 线程
            for i in 0..finish.len() {
                if !finish[i] && (0..work.len()).all(|j| need[i][j] as i32 <= work[j]) {
                    // 将当前进程的资源释放给work

                    for j in 0..work.len() {
                        work[j] += alloc[i][j] as i32;
                    }
                    // 当前进程结束
                    finish[i] = true;
                    // 找到了一个finish进程
                    found = true;
                }
            }

            if !found {
                break;
            }
        }
        // info!("finish:{:?}", finish);

        // 全部完成才是安全状态
        finish.iter().all(|&f| f)
    }
}
