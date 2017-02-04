use std::sync::{Arc, Mutex, Condvar};
use std::thread;

pub struct Timer<'a> {
    pub is_alive: bool,
    pub was_reset: bool,
    timed_out: &'a Condvar,
    cv: Condvar,
    cv_m: Mutex<bool>,
    step: u64,
    jitter: u64,
    thread: Option<std::thread::Thread>,
}

impl<'a> Timer<'a> {
    pub fn new(timed_out: &'a Condvar) -> Timer {
        Timer {
            is_alive: false,
            was_reset: false,
            timed_out: timed_out,
            step: 2,
            jitter: 0,
            thread: None,
            cv_m: Mutex::new(false),
            cv: Condvar::new(),

        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Condvar;
    use Timer;

    #[test]
    fn it_works() {
        let cv = Condvar::new();
        let t = Timer::new(&cv);
        assert!(t.is_alive == false);
    }
}
