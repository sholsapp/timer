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
    // The amount of time in nanoseconds to count down from.
    pub step: u64,
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
    pub fn new(step: u64, timed_out: Arc<Condvar>) -> Timer {
        Timer {
            handle: None,
            alive: Arc::new(AtomicBool::new(false)),
            cv: Arc::new(Condvar::new()),
            m: Arc::new(Mutex::new(false)),
            timed_out: timed_out,
            step: step,
            expiries: Arc::new(AtomicUsize::new(0)),
        }
    }
    /// Internal timer loop.
    ///
    fn spin(alive: Arc<AtomicBool>, cv: Arc<Condvar>, m: Arc<Mutex<bool>>, timed_out: Arc<Condvar>, expiries: Arc<AtomicUsize>, step: u64) {
        alive.store(true, Ordering::SeqCst);
        while alive.load(Ordering::SeqCst) {
            match cv.wait_timeout(m.lock().unwrap(), Duration::new(step, 0)) {
                Ok((_, result)) => {
                    if result.timed_out() {
                        // println!("Signaling waiters!");
                        expiries.fetch_add(1, Ordering::SeqCst);
                        timed_out.notify_all();
                    } else {
                        // println!("Resetting timer!");
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
        self.handle = Some(std::thread::spawn(move || {
            Timer::spin(alive, cv, m, timed_out, expiries, step);
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
    let t = Timer::new(5, cv);
    assert!(t.alive.load(Ordering::SeqCst) == false);
}

#[test]
fn timer_start() {
    let cv = Arc::new(Condvar::new());
    let mut t = Timer::new(5, cv);
    t.start();
    std::thread::sleep(Duration::new(10, 0));
    std::thread::sleep(Duration::new(1, 0));
    t.reset();
    std::thread::sleep(Duration::new(10, 0));
    t.stop();
    assert!(t.expiries.load(Ordering::SeqCst) == 4);
}

#[test]
fn timer_reset() {

}