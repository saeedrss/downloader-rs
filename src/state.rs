use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone)]
pub struct DynState {
    pub paused: Arc<AtomicBool>,
    pub resume_notify: Arc<Notify>,
    pub config_notify: Arc<Notify>,
    pub max_connections: Arc<AtomicU32>,
    pub timeout: Arc<AtomicU32>,
    pub connect_timeout: Arc<AtomicU32>,
    #[allow(dead_code)]
    pub read_timeout: Arc<AtomicU32>,
}

impl DynState {
    pub fn new(max_connections: u32, timeout: u32, connect_timeout: u32, read_timeout: u32) -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            resume_notify: Arc::new(Notify::new()),
            config_notify: Arc::new(Notify::new()),
            max_connections: Arc::new(AtomicU32::new(max_connections)),
            timeout: Arc::new(AtomicU32::new(timeout)),
            connect_timeout: Arc::new(AtomicU32::new(connect_timeout)),
            read_timeout: Arc::new(AtomicU32::new(read_timeout)),
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Acquire)
    }

    /// Set pause state. Returns true if state actually changed.
    pub fn set_paused(&self, paused: bool) -> bool {
        let was_paused = self.paused.swap(paused, Ordering::AcqRel);
        if was_paused && !paused {
            self.resume_notify.notify_waiters();
        }
        was_paused != paused
    }

    pub async fn wait_while_paused(&self) {
        while self.is_paused() {
            let notified = self.resume_notify.notified();
            if !self.is_paused() {
                break;
            }
            notified.await;
        }
    }
}

pub struct ProxyStats {
    pub successes: AtomicU64,
    pub total_time_ns: AtomicU64,
}

impl ProxyStats {
    pub fn new() -> Self {
        Self {
            successes: AtomicU64::new(0),
            total_time_ns: AtomicU64::new(0),
        }
    }

    pub fn record(&self, elapsed_secs: f64) {
        self.successes.fetch_add(1, Ordering::Release);
        self.total_time_ns
            .fetch_add((elapsed_secs * 1_000_000_000.0) as u64, Ordering::Release);
    }

    pub fn avg_time(&self) -> f64 {
        let s = self.successes.load(Ordering::Acquire);
        if s == 0 {
            return f64::MAX;
        }
        let ns = self.total_time_ns.load(Ordering::Acquire);
        (ns as f64) / (s as f64) / 1_000_000_000.0
    }
}

pub struct DynamicSemaphore {
    active: Arc<AtomicU32>,
    notify: Arc<Notify>,
    extra_notify: Option<Arc<Notify>>,
    max_getter: Option<Box<dyn Fn() -> u32 + Send + Sync>>,
    initial_max: u32,
}

impl DynamicSemaphore {
    #[allow(dead_code)]
    pub fn new(initial_max: u32) -> Self {
        Self {
            active: Arc::new(AtomicU32::new(0)),
            notify: Arc::new(Notify::new()),
            extra_notify: None,
            max_getter: None,
            initial_max,
        }
    }

    pub fn with_getter<F>(initial_max: u32, getter: F) -> Self
    where
        F: Fn() -> u32 + Send + Sync + 'static,
    {
        Self {
            active: Arc::new(AtomicU32::new(0)),
            notify: Arc::new(Notify::new()),
            extra_notify: None,
            max_getter: Some(Box::new(getter)),
            initial_max,
        }
    }

    pub fn with_getter_and_extra<F>(initial_max: u32, extra: Arc<Notify>, getter: F) -> Self
    where
        F: Fn() -> u32 + Send + Sync + 'static,
    {
        Self {
            active: Arc::new(AtomicU32::new(0)),
            notify: Arc::new(Notify::new()),
            extra_notify: Some(extra),
            max_getter: Some(Box::new(getter)),
            initial_max,
        }
    }

    fn effective_max(&self) -> u32 {
        match &self.max_getter {
            Some(g) => (g)(),
            None => self.initial_max,
        }
    }

    pub async fn acquire(&self) -> AcquireGuard<'_> {
        loop {
            let current = self.active.load(Ordering::Acquire);
            let max = self.effective_max();
            if current < max
                && self
                    .active
                    .compare_exchange_weak(current, current + 1, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                return AcquireGuard { sem: self };
            }
            match &self.extra_notify {
                Some(n) => {
                    tokio::select! {
                        _ = self.notify.notified() => {}
                        _ = n.notified() => {}
                    }
                }
                None => self.notify.notified().await,
            }
        }
    }

    fn release(&self) {
        self.active.fetch_sub(1, Ordering::Release);
        self.notify.notify_one();
    }
}

pub struct AcquireGuard<'a> {
    sem: &'a DynamicSemaphore,
}

impl Drop for AcquireGuard<'_> {
    fn drop(&mut self) {
        self.sem.release();
    }
}
