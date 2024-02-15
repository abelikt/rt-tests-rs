
# Cyclictest written in Rust

The goal is to have a similar tool like cyclictest to be able to measure the
wakeup and effective context switch latency. The original cyclictest serves as
template and as sources for best practices.

https://wiki.linuxfoundation.org/realtime/documentation/howto/tools/cyclictest/start

See [../README.md](../README.md) for a introduction about why and why Rust
is used here.


# State of implementation

State:

* So far, it started to work and to show similar latencies like the original
cyclictest.
* The implementation is still limited to one thread and has no good reporting.
* Many places are still unsafe and can probably replaced

To-Dos:

* Extend to multiple threads
* Record histograms
* Plot nice histograms (maybe like in the
    [latency-farm](https://www.osadl.org/Create-a-latency-plot-from-cyclictest-hi.bash-script-for-latency-plot.0.html?&no_cache=1&sword_list[0]=script))
* Generate background load for tests
* Check multiple Platforms


# Using the original cyclictest

Get an overview of current latency

    sudo cyclictest -l 200 -m -S -p99 -i10000

    sudo cyclictest -q -l 200 -m -S -p99 -i100000 -h 400 > amd_ryzen_rt_$(date +"%Y_%m_%d_%H:%M:%S").txt

Observe real-time prio with:

    ps  -m -C cyclictest -o pid,pri,rtprio,uid,cputime,cmd


# Run Tests

Run With:

    cargo build && sudo target/debug/cyclictest-rs  --nanosleep
    cargo build --release && sudo target/release/cyclictest-rs  --sleep
    cargo build --release && sudo target/release/cyclictest-rs  --nanosleep
    cargo build --release && sudo target/release/cyclictest-rs  --nanosleepgettime

Observe rt prio:

    ps  -m -C cyclictest-rs -o pid,pri,rtprio,uid,cputime,cmd

Note: not sure if the rtprio setting in ps works right. It reports no rtprio
even though we are using them (Happens on 5.10.0-27-rt-amd64).


# Some measurements

From my developement system (1K samples per line))

Results are comparable to original cyclictests results, but still a bit worse.
Sometimes there are samples about 30µs, this needs more investigation.

## Without any real-time settings, simple std::thread::sleep:

    $ cargo build --release && sudo target/release/cyclictest-rs  --sleep
    ...
    Average Latency 61.095µs Maximal Latency 129.054µs
    Average Latency 63.433µs Maximal Latency 92.947µs
    Average Latency 55.066µs Maximal Latency 91.985µs
    Average Latency 56.036µs Maximal Latency 68.941µs
    Average Latency 54.85µs Maximal Latency 67.71µs
    Average Latency 55.891µs Maximal Latency 89.831µs
    Average Latency 55.387µs Maximal Latency 67.228µs
    Average Latency 56.008µs Maximal Latency 64.793µs
    Average Latency 53.779µs Maximal Latency 65.245µs
    Average Latency 56.335µs Maximal Latency 81.936µs


## Sleeping with clock_nanosleep and having the right settings in place:

    $ cargo build --release && sudo target/release/cyclictest-rs  --nanosleep
    ...
    Average Latency 2.797µs Maximal Latency 7.946µs
    Average Latency 2.739µs Maximal Latency 6.744µs
    Average Latency 2.75µs Maximal Latency 7.576µs
    Average Latency 2.835µs Maximal Latency 13.437µs
    Average Latency 2.772µs Maximal Latency 6.453µs
    Average Latency 2.783µs Maximal Latency 6.354µs
    Average Latency 2.757µs Maximal Latency 6.113µs
    Average Latency 2.704µs Maximal Latency 8.077µs
    Average Latency 2.713µs Maximal Latency 8.988µs
    Average Latency 2.62µs Maximal Latency 6.865µs


## Sleeping with clock_nanosleep, measuring time with clock_gettime and having the right settings in place:

    $ cargo build --release && sudo target/release/cyclictest-rs  --nanosleepgettime
    ...
    Average Latency 2.908 Maximal Latency 10.632
    Average Latency 2.882 Maximal Latency 8.668
    Average Latency 2.767 Maximal Latency 6.554
    Average Latency 2.781 Maximal Latency 9.57
    Average Latency 2.784 Maximal Latency 9.419
    Average Latency 2.853 Maximal Latency 7.526
    Average Latency 2.811 Maximal Latency 8.057
    Average Latency 2.799 Maximal Latency 7.445
    Average Latency 2.86 Maximal Latency 7.196
    Average Latency 2.774 Maximal Latency 7.606


# Test Systems

Development:

* OS: Debian Bookworm 12.4
* cpuinfo : `AMD Ryzen 5 2600 Six-Core Processor`
* uname : `6.1.0-17-rt-amd64 #1 SMP PREEMPT_RT Debian 6.1.69-1 (2023-12-30) x86_64 GNU/Linux`
* Type: Development system; mate desktop and other stuff is running
