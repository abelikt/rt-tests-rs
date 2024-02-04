
# Real-Time Linux Tests in Rust

The idea behind this project is to provide some real-time tests written in Rust.
The goal is to provide similar test like the can be found in
[rt-tests](https://wiki.linuxfoundation.org/realtime/documentation/howto/tools/rt-tests).

So far there is only cyclictest-rs, see cyclictest-rs folder
[./cyclictest-rs/README.md](./cyclictest-rs/README.md).

The target is not to rewrite rt-tests, I want to find out how a real-time
program written in Rust has to look like, what is allowed in rt contexts and
what spoils rt behaviour.

The question is how Rust programs are performing regarding real-time operations.
How latencies look like an if we want to, for example want to write control
systems like PLCs or CNCs with Rust? What crated are needed or should be avoided?

To test, a Linux system with a PREEMPT_RT Kernel is needed. Some distributions
offer prepacked Kernels (e.g. Debian), for others you need to patch and 
compile yourself.

Also, this is a learners project, don't expect production ready software.


# Rust for Real-Time Systems?

Rust seems to have everything needed for (soft)-real-time systems.
Especially, a deterministic memory management i.e. no garbage collection and
an execution performance comparable to C++.

C and Assembler are probably the holy grail for rt software.
C++ seems also to be fine, as long as no memory management is triggered in an
rt-context. This means to avoid malloc, free, new and so on or to replace calls
with a deterministic version that works on a preallocated memory region [1].

For Rust, I simply want to find out what we need to avoid.
Dynamic memory management for sure but is there something else?

See also:
* [1] C.M. Kormanyos, Real-Time C++: Efficient Object-Oriented and Template
    Microcontroller Programming, ISBN 9783662629956.
    Also: https://github.com/ckormanyos/real-time-cpp


## Open Questions

* What is the performance impact to trigger memory management?
* What is the performance impact to use trait objects for error handling, like `Box<dyn Error>`?
* Can we use parts of the std library or can we only use core?


# System preparation

Under Debian Bookworm e.g.:

    apt install rt-tests
    apt install linux-image-6.1.0-17-rt-amd64

Reboot into the rt Kernel.

# Links

Real-Time tests for Linux:

* https://wiki.linuxfoundation.org/realtime/start
* https://wiki.linuxfoundation.org/realtime/documentation/start
* https://wiki.linuxfoundation.org/realtime/documentation/howto/tools/start
* https://wiki.linuxfoundation.org/realtime/documentation/howto/tools/rt-tests
* https://wiki.linuxfoundation.org/realtime/documentation/howto/tools/cyclictest/start
* https://git.kernel.org/pub/scm/utils/rt-tests/rt-tests.git/

OSADL QA monitoring Farm:

* https://www.osadl.org/Continuous-latency-monitoring.qa-farm-monitoring.0.html

More about scheduling

* https://www.kernel.org/doc/html/latest/scheduler/index.html
* https://www.kernel.org/doc/html/latest/scheduler/sched-rt-group.html
