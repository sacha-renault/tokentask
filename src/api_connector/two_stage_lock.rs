use parking_lot::{RwLock, RwLockWriteGuard};

#[derive(Debug, Copy, Clone)]
pub enum LockStrategy {
    BeforeFetch,
    AfterFetch,
}

// TODO find a better name
pub fn lock_around<'a, F, T, R>(
    rw: &'a RwLock<T>,
    when: LockStrategy,
    func: F,
) -> (RwLockWriteGuard<'a, T>, R)
where
    F: FnOnce() -> R,
{
    match when {
        LockStrategy::BeforeFetch => {
            let lock = rw.write();
            let result = func();
            (lock, result)
        }
        LockStrategy::AfterFetch => {
            let result = func();
            let lock = rw.write();
            (lock, result)
        }
    }
}
