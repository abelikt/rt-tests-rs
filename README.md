

# Real Time Linux Tests in Rust

The idea behind this project is to provide some real-time tests written in Rust.

So far there is only cyclictest, see cyclictest-rs folder.

The target is not to rewrite rt-tests, I want to find out how a real-time
program written in Rust has to look like, what is allowed in rt contexts and
what spoils rt behaviour.

To test, a Linux System with a PREEMPT_RT Kernel is needed. Some distributions
offer prepacked Kernels (e.g. Debian), for others you need to patch and 
compile yourself.

Also, this is a learners project, don't expect production ready software.


# Links

Real-Time tests for Linux:

* https://git.kernel.org/pub/scm/utils/rt-tests/rt-tests.git/
* https://wiki.linuxfoundation.org/realtime/documentation/start
* https://wiki.linuxfoundation.org/realtime/documentation/howto/tools/cyclictest/start

OSADL qa monitoring Farm:

https://www.osadl.org/Continuous-latency-monitoring.qa-farm-monitoring.0.html

