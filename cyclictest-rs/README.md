
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

* Find equivalent of pthread_attr_setschedpolicy (https://wiki.linuxfoundation.org/realtime/documentation/howto/applications/application_base)
* DONE Extend to multiple threads
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

Note: not sure if the rtprio setting in ps is displayed right. It reports no rtprio
even though we are using them (happens on 5.10.0-27-rt-amd64, 6.1.0-18-rt-amd64).


# Some measurements

From my developement system (1K samples per line))

Results are comparable to original cyclictests results, but still a bit worse.
Sometimes there are samples about 30µs, this needs more investigation.

## Without any real-time settings, simple std::thread::sleep:

    $ cargo build --release && sudo target/release/cyclictest-rs  --sleep
    ...
    Average Latency 68.894µs Maximal Latency 321.632µs
    Average Latency 73.944µs Maximal Latency 412.127µs
    Average Latency 73.317µs Maximal Latency 333.374µs
    Average Latency 74.609µs Maximal Latency 474.898µs
    Average Latency 71.179µs Maximal Latency 251.477µs
    Average Latency 69.87µs Maximal Latency 318.315µs
    Average Latency 73.449µs Maximal Latency 502.901µs
    Average Latency 70.285µs Maximal Latency 340.889µs
    Average Latency 71.172µs Maximal Latency 278.278µs
    Average Latency 67.685µs Maximal Latency 299.499µs
    Average Latency 60.739µs Maximal Latency 201.68µs
    Average Latency 71.372µs Maximal Latency 500.387µs



## Sleeping with clock_nanosleep and having the right settings in place:

    $ cargo build --release && sudo target/release/cyclictest-rs  --nanosleep
    ...
    Histogram: Rows:Latency_us; Columns:Threads
    0     0     0     0     0     0     0     0     0     0     0     0     0 
    1     2     1    14    17    12    10     6     3     0    11     4     5 
    2  3487  3350  6496  6721  6620  7189  4891  6349  4811  5211  5020  5115 
    3  5812  5964  2946  2754  2757  2315  4695  3375  4809  4463  4554  4475 
    4   442   433   349   320   410   302   270   176   252   212   305   282 
    5   140   143   125   112   101    92    86    48    67    52    61    68 
    6    58    59    48    51    55    47    24    23    31    27    25    24 
    7    30    27    13    14    28    26    12     6    19    13    13    13 
    8    15    11     5     5     6    12    10    10     5     8     7     5 
    9     6     2     0     3     5     3     2     7     3     1     5     7 
    10     3     3     1     0     3     2     1     1     1     2     2     3 
    11     2     2     0     1     0     1     3     1     1     0     1     0 
    12     1     3     1     2     1     0     0     1     1     0     0     0 
    13     2     0     1     0     0     0     0     0     0     0     0     1 
    14     0     1     1     0     1     0     0     0     0     0     1     0 
    Ov     0     1     0     0     1     1     0     0     0     0     2     2 
    Stats
    T0 µs: Min    2.0  Avg    3.2  Max   13.7  Overflows      0
    T1 µs: Min    2.0  Avg    3.2  Max   17.4  Overflows      1
    T2 µs: Min    1.9  Avg    3.0  Max   14.8  Overflows      0
    T3 µs: Min    1.9  Avg    2.9  Max   12.6  Overflows      0
    T4 µs: Min    1.9  Avg    3.0  Max   18.0  Overflows      1
    T5 µs: Min    1.9  Avg    2.9  Max   15.1  Overflows      1
    T6 µs: Min    1.9  Avg    3.1  Max   11.7  Overflows      0
    T7 µs: Min    1.9  Avg    2.9  Max   12.2  Overflows      0
    T8 µs: Min    2.0  Avg    3.1  Max   12.9  Overflows      0
    T9 µs: Min    1.9  Avg    3.0  Max   10.1  Overflows      0
    T10 µs: Min    2.0  Avg    3.1  Max   21.5  Overflows      2
    T11 µs: Min    1.9  Avg    3.1  Max   19.4  Overflows      2



## Sleeping with clock_nanosleep, measuring time with clock_gettime and having the right settings in place:

    $ cargo build --release && sudo target/release/cyclictest-rs  --nanosleepgettime
    ...
    Histogram: Rows:Latency_us; Columns:Threads
    0     0     0     0     0     0     0     0     0     0     0     0     0 
    1    15     9    25    18     2     1    11     9     4     2     4     0 
    2  4740  4541  6009  6494  4297  5698  5622  6524  6144  6589  3726  3792 
    3  4754  4999  3400  3103  5099  3894  4022  3154  3466  3107  5843  5819 
    4   369   332   389   237   392   239   221   205   257   182   322   295 
    5    72    75   109    76   111    85    69    55    79    79    67    61 
    6    26    24    36    41    59    45    30    29    26    28    23    18 
    7    13    12    22    17    23    22    14    13    14     5     6     7 
    8     5     5     4     9     7     7     5     6    10     3     5     3 
    9     4     3     5     3     4     7     2     0     0     3     3     2 
    10     0     0     0     1     1     0     2     3     0     0     1     2 
    11     1     0     1     0     1     0     0     1     0     1     0     1 
    12     0     0     0     0     1     0     1     1     0     1     0     0 
    13     0     0     0     1     0     0     0     0     0     0     0     0 
    14     1     0     0     0     0     0     1     0     0     0     0     0 
    Ov     0     0     0     0     3     2     0     0     0     0     0     0 
    Stats
    T0 µs: Min    1.9  Avg    3.1  Max   14.5  Overflows      0
    T1 µs: Min    1.9  Avg    3.1  Max    9.7  Overflows      0
    T2 µs: Min    1.9  Avg    3.0  Max   11.3  Overflows      0
    T3 µs: Min    1.9  Avg    2.9  Max   14.0  Overflows      0
    T4 µs: Min    2.0  Avg    3.2  Max   27.1  Overflows      3
    T5 µs: Min    2.0  Avg    3.0  Max   20.6  Overflows      2
    T6 µs: Min    1.9  Avg    3.0  Max   14.5  Overflows      0
    T7 µs: Min    1.9  Avg    2.9  Max   12.2  Overflows      0
    T8 µs: Min    1.9  Avg    3.0  Max    8.7  Overflows      0
    T9 µs: Min    2.0  Avg    2.9  Max   12.7  Overflows      0
    T10 µs: Min    2.0  Avg    3.2  Max   10.2  Overflows      0
    T11 µs: Min    2.0  Avg    3.2  Max   11.3  Overflows      0



# Test Systems

Development:

* OS: Debian Bookworm 12.5
* cpuinfo : `AMD Ryzen 5 2600 Six-Core Processor`
* uname : `6.1.0-18-rt-amd64 #1 SMP PREEMPT_RT Debian 6.1.76-1 (2024-02-01) x86_64 GNU/Linux`
* Type: Development system; mate desktop and other stuff is running
