
# Cyclictest written in Rust

The goal is to have a similar tool like cyclictest to be able to measure the
wakeup and effective context switch latency. The original cyclictest serves as
template and as sources for best practices.

https://wiki.linuxfoundation.org/realtime/documentation/howto/tools/cyclictest/start

See [../README.md](../README.md) for a introduction about why and why Rust
is used here.


# State of implementation

* So far, it started to work and to show similar latencies like the original
cyclictest.
* The implementation is still limited to one thread and has no good reporting.
* Many places are still unsafe and can probably replaced

To-Dos:

* Extend to multiple threads
* Record histograms
* Plot nice histograms (maybe https://www.osadl.org/Create-a-latency-plot-from-cyclictest-hi.bash-script-for-latency-plot.0.html?&no_cache=1&sword_list[0]=script)
* Generate background load for tests
* Check multiple Platforms


# Using the original cyclictest

Get an overview of current latency

    sudo cyclictest -l 200 -m -S -p99 -i10000

    sudo cyclictest -q -l 200 -m -S -p99 -i100000 -h 400 > amd_ryzen_rt_$(date +"%Y_%m_%d_%H:%M:%S").txt

Observe real-time prio with:

    ps  -m -C cyclictest-rs -o pid,pri,rtprio,uid,cputime,cmd


# Run Tests

Run With:

    cargo build && sudo target/debug/cyclictest-rs  --nanosleep
    cargo build --release && sudo target/release/cyclictest-rs  --nanosleep


# Test Systems

Development:
* OS: Debian Bookworm 12.4
* cpuinfo : `AMD Ryzen 5 2600 Six-Core Processor`
* uname : `6.1.0-17-rt-amd64 #1 SMP PREEMPT_RT Debian 6.1.69-1 (2023-12-30) x86_64 GNU/Linux`
