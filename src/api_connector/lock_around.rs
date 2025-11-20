use parking_lot::{RwLock, RwLockWriteGuard};

#[derive(Debug, Copy, Clone)]
pub enum LockBehavior {
    /// Hold lock during operation
    HoldDuringOperation,

    /// Hold lock only after operation
    HoldAfterOperation,
}

pub fn lock_around<'a, F, T, R>(
    rw: &'a RwLock<T>,
    when: LockBehavior,
    func: F,
) -> (RwLockWriteGuard<'a, T>, R)
where
    F: FnOnce() -> R,
{
    match when {
        LockBehavior::HoldDuringOperation => {
            let lock = rw.write();
            let result = func();
            (lock, result)
        }
        LockBehavior::HoldAfterOperation => {
            let result = func();
            let lock = rw.write();
            (lock, result)
        }
    }
}
