# Pipe Performance Analysis Accompanyment

This repository accompanies the submission for CS736-P2 and provides associated artifacts (traces, source files) discussed within the write-up.

## System Information

CPU : Intel(R) Xeon(R) Gold 6142 CPU @ 2.60GHz
MEM : DDR4 @ 2666 MHz
OS  : Ubuntu 22.04.2 LTS (GNU/Linux 5.15.0-187-generic x86_64)

## Organization

- `perfetto_traces` contains raw perfetto traces on executions at different message sizes. These files are not meant to be human readable but are intended to be visualized via perfetto.
- `trace_function_graph` contains human readable per-CPU `ftrace` output for per-CPU kernel function call graph information. 
- `trace_calls_events` contains human readable aggregate `ftrace` output for specfic pipe related function calls alongside scheduling and syscall events. 