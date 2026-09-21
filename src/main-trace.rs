// modified code used for perfetto tracing
use std::io::{PipeReader, PipeWriter, Read, Write, pipe};
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration};
use clap::Parser;
use nix::libc;
use nix::unistd::{ForkResult, fork, Pid};
use nix::sched::{sched_setaffinity, CpuSet};
use nix::sys::mman::{MapFlags, mmap_anonymous, ProtFlags};
use perfetto_sdk::producer::*;
use perfetto_sdk::track_event::*;
use perfetto_sdk::{track_event_begin, track_event_end};

perfetto_sdk::track_event_categories! {
    pub mod categories {
        ("ipc", "IPC benchmark events", []),
        ("write", "writing events", []),
        ("read", "reading events", []),
        ("ack", "ack shared meme management", [])
    }
}

use categories as perfetto_te_ns;

fn init_perfetto() {
    Producer::init(
        ProducerInitArgsBuilder::new()
            .backends(Backends::SYSTEM)
            .build(),
    );

    TrackEvent::init();
    categories::register().unwrap();
}

fn pin_to_cpu(cpu: usize) {
    let mut cpuset = CpuSet::new();
    cpuset.set(cpu).unwrap();

    sched_setaffinity(Pid::from_raw(0), &cpuset).unwrap();

    eprintln!(
        "PID {} pinned to CPU {}",
        std::process::id(),
        cpu
    );
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "0")]
    warmup_repetitions: usize,
    #[arg(short, long, default_value = "1")]
    repetitions: usize,
    #[arg(short, long, default_value = "1024")]
    message_size: usize,
}

const MESSAGE: [u8; 512 * 1024] = [67; 512 * 1024];

fn main() {
    let cli = Cli::parse();
    assert!(cli.message_size <= MESSAGE.len(), "message size exceeds MESSAGE");

    let (reader, writer) = pipe().unwrap();

    // TODO: will accesses be optimized out?
    let ack = unsafe {
        &mut *(mmap_anonymous(
            None,
            NonZeroUsize::new_unchecked(1),
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
        ).unwrap().as_ptr() as *mut AtomicBool)
    };
    ack.store(false, SeqCst);
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child: _ }) => {
            pin_to_cpu(2);
            init_perfetto();
            std::thread::sleep(Duration::from_millis(200));
            pusher(writer, ack, cli.warmup_repetitions, cli.repetitions, cli.message_size)
        },
        Ok(ForkResult::Child) => {
            pin_to_cpu(3);
            init_perfetto();
            std::thread::sleep(Duration::from_millis(200));
            puller(reader, ack, cli.warmup_repetitions, cli.repetitions, cli.message_size);
        },
        Err(_) => println!("fork failed!")
    }
}

fn pusher(
    mut writer: PipeWriter,
    ack: &mut AtomicBool,
    warmup_repetitions: usize,
    repetitions: usize,
    message_size: usize,
) -> () {
    for _ in 0..(warmup_repetitions + repetitions) {
        track_event_begin!("ipc", "send");

        track_event_begin!("write", "write_all");
        if let Err(e) = writer.write_all(&MESSAGE[..message_size]) {
            std::hint::cold_path();
            println!("{}", e);
        }
        track_event_end!("write");

        track_event_begin!("ack", "ack spin_loop");
        while !ack.load(SeqCst) {
            std::hint::spin_loop();
        }
        track_event_end!("ack");
        track_event_begin!("ack", "ack reset");
        ack.store(false, SeqCst);
        track_event_end!("ack");
        track_event_end!("ipc");
    }
}

fn puller(
    mut reader: PipeReader,
    ack: &mut AtomicBool,
    warmup_repetitions: usize,
    repetitions: usize,
    message_size: usize,
) {
    for _ in 0..(warmup_repetitions + repetitions) {
        track_event_begin!("ipc", "recieve");
        track_event_begin!("write", "msg buf alloc (user)");
        let mut message = vec![0_u8; message_size];
        track_event_end!("write");

        track_event_begin!("read", "read_exact");
        if let Err(e) = reader.read_exact(&mut message) {
            std::hint::cold_path();
            println!("{}", e);
        }
        track_event_end!("read");

        track_event_begin!("ack", "ack store");
        ack.store(true, SeqCst);
        track_event_end!("ack");

        track_event_begin!("read", "drop read buff (user)");
        drop(message);
        track_event_end!("read");
        track_event_end!("ipc");
    }
}
