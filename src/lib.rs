extern crate rand;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, Condvar};
use std::time::Duration;

/// A countdown timer.
///
/// A countdown timer counts down from the specified `step` parameter. While
/// counting down, it is possible to reset the timer.
///
/// If the count down timer expires, i.e., if `step` many nanoseconds expires,
/// the `timed_out` condition variable is signalled.
///
pub struct Timer {
    // Internal condition variable used to implement a timer.
    cv: Arc<Condvar>,
    // Internal mutex for `cv` used to implement a timer.
    m: Arc<Mutex<bool>>,
    // Internal thread handle to join on shutdown.
    handle: Option<std::thread::JoinHandle<()>>,
    // Condition variable signalled if/when timer expires.
    pub timed_out: Arc<Condvar>,
    // The amount of time to count down from.
    pub step: Duration,
    // The amount of time, if any, to randomize the count down from.
    pub jitter: Duration,
    // True if the timer is counting down.
    pub alive: Arc<AtomicBool>,
    /// Number of times this timer has expired.
    pub expiries: Arc<AtomicUsize>,
}

impl Timer {
    /// Create a new timer.
    ///
    /// # Arguments
    ///
    /// * `timed_out` - Condition to signal if the timer expires.
    ///
    pub fn new(step: Duration, jitter: Duration, timed_out: Arc<Condvar>) -> Timer {
        Timer {
            handle: None,
            alive: Arc::new(AtomicBool::new(false)),
            cv: Arc::new(Condvar::new()),
            m: Arc::new(Mutex::new(false)),
            timed_out: timed_out,
            step: step,
            jitter: jitter,
            expiries: Arc::new(AtomicUsize::new(0)),
        }
    }
    /// Convert a duration to milliseconds.
    ///
    /// Annoying, right? See https://github.com/rust-lang/rfcs/issues/1545.
    ///
    fn duration_to_millis(d: Duration) -> u64 {
        1000 * d.as_secs() + (d.subsec_nanos() as u64 / 1000000)
    }
    /// Calculate a wait time.
    ///
    fn calculate_wait_duration(step: Duration, jitter: Duration) -> Duration {
        let random = rand::random::<u64>();
        let step_ms = Timer::duration_to_millis(step);
        let jitter_ms = Timer::duration_to_millis(jitter);
        if jitter_ms > 0 {
            return Duration::from_millis(step_ms - (random % jitter_ms))
        } else {
            return Duration::from_millis(step_ms)
        }
    }
    /// Internal timer loop.
    ///
    fn spin(alive: Arc<AtomicBool>,
            cv: Arc<Condvar>,
            m: Arc<Mutex<bool>>,
            timed_out: Arc<Condvar>,
            expiries: Arc<AtomicUsize>,
            step: Duration,
            jitter: Duration) {
        alive.store(true, Ordering::SeqCst);
        while alive.load(Ordering::SeqCst) {
            let wait_duration = Timer::calculate_wait_duration(step, jitter);
            match cv.wait_timeout(m.lock().unwrap(), wait_duration) {
                Ok((_, result)) => {
                    if result.timed_out() {
                        expiries.fetch_add(1, Ordering::SeqCst);
                        timed_out.notify_all();
                    }
                },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
    /// Start the timer.
    ///
    pub fn start(&mut self) {
        let alive = self.alive.clone();
        let expiries = self.expiries.clone();
        let cv = self.cv.clone();
        let m = self.m.clone();
        let timed_out = self.timed_out.clone();
        let step = self.step;
        let jitter = self.jitter;
        self.handle = Some(std::thread::spawn(move || {
            Timer::spin(alive, cv, m, timed_out, expiries, step, jitter);
        }));
    }
    /// Stop the timer.
    ///
    pub fn stop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
        self.handle
            .take().expect("Couldn't stop non-running thread!")
            .join().expect("Couldn't join spawned thread!");
    }
    /// Reset the timer.
    ///
    pub fn reset(&mut self) {
        self.cv.notify_all();
    }
}

#[test]
fn it_works() {
    let cv = Arc::new(Condvar::new());
    let d = Duration::from_secs(5);
    let j = Duration::from_secs(0);
    let t = Timer::new(d, j, cv);
    assert!(t.alive.load(Ordering::SeqCst) == false);
}

#[test]
fn timer_start() {
    let cv = Arc::new(Condvar::new());
    let mut t = Timer::new(Duration::from_millis(50),
                           Duration::from_millis(0),
                           cv);
    t.start();
    // This should cause at least two expiries...
    std::thread::sleep(Duration::from_millis(100));
    t.stop();
    println!("{}", t.expiries.load(Ordering::SeqCst));
    assert!(t.expiries.load(Ordering::SeqCst) >= 2);
    assert!(t.expiries.load(Ordering::SeqCst) < 5);
}

#[test]
fn timer_reset() {
    let cv = Arc::new(Condvar::new());
    let mut t = Timer::new(Duration::from_millis(50),
                           Duration::from_millis(10),
                           cv);
    t.start();
    // This should cause two expiries...
    std::thread::sleep(Duration::from_millis(125));
    // This should catch an expiry before it happens...
    t.reset();
    // This should cause two expiries...
    std::thread::sleep(Duration::from_millis(100));
    t.stop();
    assert!(t.expiries.load(Ordering::SeqCst) >= 4);
    assert!(t.expiries.load(Ordering::SeqCst) < 6);
}