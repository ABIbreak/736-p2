# Pipe Performance Analysis Accompanyment

This repository accompanies the submission for CS736-P2 and provides associated artifacts (traces, source files) discussed within the write-up.

## System Information

- CPU : Intel(R) Xeon(R) Gold 6142 CPU @ 2.60GHz
- MEM : DDR4 @ 2666 MHz
- OS  : Ubuntu 22.04.2 LTS (GNU/Linux 5.15.0-187-generic x86_64)

## Organization

- `perfetto_traces` contains raw perfetto traces on executions at different message sizes. These files are not meant to be human readable but are intended to be visualized via perfetto.
- `trace_function_graph` contains human readable per-CPU `ftrace` output for per-CPU kernel function call graph information. 
- `trace_calls_events` contains human readable aggregate `ftrace` output for specfic pipe related function calls alongside scheduling and syscall events. 

## Benchmark Results

The exact values for the graph displayed in the write-up are as follows:

| Bytes | Time (ns) | Throughput (MiB/s) |
|------:|----------:|-------------------:|
| 4 | 1,614 | 2.36 |
| 16 | 1,562 | 9.77 |
| 64 | 1,634 | 37.35 |
| 256 | 1,670 | 146.19 |
| 1,024 | 1,816 | 537.75 |
| 4,096 | 2,472 | 1,580.20 |
| 16,384 | 6,690 | 2,335.58 |
| 32,768 | 11,724 | 2,665.47 |
| 65,536 | 21,319 | 2,931.66 |
| 65,537 | 32,445 | 1,926.37 |
| 131,072 | 50,628 | 2,468.99 |
| 262,144 | 104,836 | 2,384.68 |
| 524,288 | 216,795 | 2,306.33 |
