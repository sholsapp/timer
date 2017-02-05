# timer

<a href='https://travis-ci.org/sholsapp/timer'>
<img src='https://img.shields.io/travis/sholsapp/timer/master.svg?style=flat-square' />
</a>

# about

A countdown timer counts down from the specified `step` parameter. While
counting down, it is possible to reset the timer.

If the count down timer expires, i.e., if `step` many nanoseconds expires, the
`timed_out` condition variable is signalled.

# usage

```
extern crate timer;
use std::sync::Condvar;
use std::time::Duration;
let cv = Condvar::new();
let d = Duration::from_millis(100);
let mut t = Timer::new(d, cv);
t.start();
// ...
t.stop();
```
