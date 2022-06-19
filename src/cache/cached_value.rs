use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Wrapper over any value used in cache core
///
/// Can be cloned cheaply multiple times and locks only on "write"
pub struct CachedValue<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> CachedValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.inner.write().await
    }
}

impl<T> Clone for CachedValue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Deref for CachedValue<T> {
    type Target = Arc<RwLock<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// imports for the CachedValueBlocking
use std::sync::{
    RwLock as RwLockBlocking, RwLockReadGuard as RwLockReadGuardBlocking,
    RwLockWriteGuard as RwLockWriteGuardBlocking,
};

/// Wrapper over any value used in cache core
///
/// Can be cloned cheaply multiple times and locks only on "write"
///
/// In contrast to `CachedValue`, this wrapper doesn't use
/// asynchronous operations and can block the thread
///
/// `read()` and `write()` can also panic if the lock is poisoned
pub struct CachedValueBlocking<T> {
    inner: Arc<RwLockBlocking<T>>,
}

impl<T> CachedValueBlocking<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLockBlocking::new(value)),
        }
    }

    pub fn read(&self) -> RwLockReadGuardBlocking<'_, T> {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuardBlocking<'_, T> {
        self.inner.write().unwrap()
    }
}

impl<T> Clone for CachedValueBlocking<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Deref for CachedValueBlocking<T> {
    type Target = Arc<RwLockBlocking<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
