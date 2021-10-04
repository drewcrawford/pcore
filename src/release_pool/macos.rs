use objr::bindings::{AutoreleasePool, ActiveAutoreleasePool};
use std::ops::Deref;

///This type can be deferenced to get a platform-specific pool type.
pub struct ReleasePool(AutoreleasePool);

///Creates an autoreleasepool.
pub fn autoreleasepool<F: FnOnce(&ReleasePool) -> R,R>(f: F) -> R {
    let a = unsafe{ ReleasePool::new() };
    f(&a)
}

impl ReleasePool {
    ///Creates a new pool.  The pool will be dropped when this type is dropped.
    ///
    /// # Safety
    /// Autorelease pools must be dropped in reverse order to when they are created. If you don't want to maintain
    /// this invariant yourself, see the [autoreleasepool] safe wrapper.
    pub unsafe fn new() -> Self {
        ReleasePool(AutoreleasePool::new())
    }
}

impl Deref for ReleasePool {
    type Target = ActiveAutoreleasePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
