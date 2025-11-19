use parking_lot::{RwLock, RwLockWriteGuard};

#[derive(Debug, Copy, Clone)]
pub enum FetchBehavior {
    /// Fetch invalidates the old token - must hold lock during fetch
    FetchInvalidatesToken,

    /// Old token remains valid - only lock when writing new token
    OldTokenRemainsValid,
}

// TODO find a better name
pub fn lock_around<'a, F, T, R>(
    rw: &'a RwLock<T>,
    when: FetchBehavior,
    func: F,
) -> (RwLockWriteGuard<'a, T>, R)
where
    F: FnOnce() -> R,
{
    match when {
        FetchBehavior::FetchInvalidatesToken => {
            let lock = rw.write();
            let result = func();
            (lock, result)
        }
        FetchBehavior::OldTokenRemainsValid => {
            let result = func();
            let lock = rw.write();
            (lock, result)
        }
    }
}
