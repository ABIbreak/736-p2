use std::arch::x86_64::{__cpuid, _rdtsc};
use std::hint::black_box;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const WARMUP_REPETITIONS: usize = 0;//10;
const MEASUREMENT_REPETITIONS: usize = 20;//10;

static TSC_MHZ: OnceLock<u64> = OnceLock::new();

fn main() {
    TSC_MHZ.set(get_tsc_mhz()).unwrap();
    println!("TSC_MHZ = {} mhz", tsc_mhz());

    let differences = difference_between_repetitions(
        WARMUP_REPETITIONS,
        MEASUREMENT_REPETITIONS,
    );
    let rdtsc_measurements =
        rdtsc_measurements(WARMUP_REPETITIONS, MEASUREMENT_REPETITIONS);
    let serializing_rdtsc_measurements = serializing_rdtsc_measurements(
        WARMUP_REPETITIONS,
        MEASUREMENT_REPETITIONS,
    );

    let average_rdtsc_latency = get_average_rdtsc_latency();
    let average_cpuid_0_latency = get_average_cpuid_0_latency();

    println!(
        "average_rdtsc_latency = {} ticks = {} ns",
        average_rdtsc_latency,
        ticks_to_ns(average_rdtsc_latency)
    );
    println!(
        "average cpuid(0) latency = {} ticks = {} ns",
        average_cpuid_0_latency,
        ticks_to_ns(average_cpuid_0_latency)
    );

    println!("difference between repetitions");
    for difference in differences {
        println!("{} ns", difference.as_nanos());
    }

    println!("rdtsc measurement");
    for ticks in rdtsc_measurements {
        //let ticks = ticks.saturating_sub(average_rdtsc_latency);
        println!("{} ns", ticks_to_ns(ticks));
    }

    println!("serializing rdtsc measurement");
    //let overhead = average_rdtsc_latency + 2 * average_cpuid_0_latency;
    for ticks in serializing_rdtsc_measurements {
        //let ticks = ticks.saturating_sub(overhead);
        println!("{} ns", ticks_to_ns(ticks));
    }
}

fn tsc_mhz() -> u64 {
    *TSC_MHZ.get().unwrap()
}

fn ticks_to_ns(ticks: u64) -> u64 {
    ticks * 1_000 / tsc_mhz()
}

fn difference_between_repetitions(
    warmup_repetitions: usize,
    repetitions: usize,
) -> Box<[Duration]> {
    let total = warmup_repetitions + repetitions;
    let mut measurements = Vec::with_capacity(total);
    for _ in 0..total {
        measurements.push(Instant::now());
    }

    measurements[warmup_repetitions..]
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect()
}

fn rdtsc_measurements(
    warmup_repetitions: usize,
    repetitions: usize,
) -> Box<[u64]> {
    let total = warmup_repetitions + repetitions;
    let mut measurements = Vec::with_capacity(total);
    for _ in 0..total {
        measurements.push(rdtsc_measurement());
    }

    measurements[warmup_repetitions..].into()
}

fn serializing_rdtsc_measurements(
    warmup_repetitions: usize,
    repetitions: usize,
) -> Box<[u64]> {
    let total = warmup_repetitions + repetitions;
    let mut measurements = Vec::with_capacity(total);
    for _ in 0..total {
        measurements.push(serializing_rdtsc_measurement());
    }

    measurements[warmup_repetitions..].into()
}

fn serializing_rdtsc_measurement() -> u64 {
    black_box(__cpuid(0));
    let before = unsafe { _rdtsc() };
    black_box(__cpuid(0));
    black_box(Instant::now());
    black_box(__cpuid(0));
    let after = unsafe { _rdtsc() };
    black_box(__cpuid(0));

    after - before
}

fn rdtsc_measurement() -> u64 {
    let before = unsafe { _rdtsc() };
    black_box(Instant::now());
    let after = unsafe { _rdtsc() };

    after - before
}

// Specifically for Skylake (Xeon Gold 6142)
fn get_tsc_mhz() -> u64 {
    __cpuid(0x16).eax as u64
}

fn get_average_rdtsc_latency() -> u64 {
    const N: u64 = 128;

    let mut accumulator = 0;

    for _ in 0..N {
        let first = black_box(unsafe { _rdtsc() });
        let second = black_box(unsafe { _rdtsc() });
        accumulator += second - first;
    }

    accumulator / N
}

fn get_average_cpuid_0_latency() -> u64 {
    const N: u64 = 128;

    _ = black_box(__cpuid(0));
    let before = unsafe { _rdtsc() };
    for _ in 0..N {
        _ = black_box(__cpuid(0));
    }
    let after = unsafe { _rdtsc() };
    _ = black_box(__cpuid(0));

    (after - before) / N
}
