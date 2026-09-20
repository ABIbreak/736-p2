use std::io::{PipeReader, PipeWriter, Read, Write, pipe};
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration, Instant};
use std::process;
use clap::Parser;
use nix::unistd::{ForkResult, fork};
use nix::sys::mman::{MapFlags, mmap_anonymous, ProtFlags};

const MESSAGE: [u8; 512 * 1024] = [67; 512 * 1024];

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "0")]
    warmup_repetitions: usize,
    #[arg(short, long, default_value = "1")]
    repetitions: usize,
    #[arg(short, long, default_value = "1024")]
    message_size: usize,
}

fn main() {
    let cli = Cli::parse();

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
        Ok(ForkResult::Parent { child }) => {
            println!("parent id pid {}", process::id());
            println!("child is pid {}", child);
            for (i, time) in pusher(writer, ack, cli.warmup_repetitions, cli.repetitions, cli.message_size).iter().enumerate() {
                println!("{}: {} us ({} ns)", i, time.as_micros(), time.as_nanos());
            }
        },
        Ok(ForkResult::Child) => {
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
) -> Vec<Duration> {
    let mut timing = Vec::with_capacity(repetitions);

    for i in 0..(warmup_repetitions + repetitions) {
        //println!("parent iter {}", i);
        let start = Instant::now();

        if let Err(e) = writer.write_all(&MESSAGE[..message_size]) {
            std::hint::cold_path();
            println!("{}", e);
        }

        while !ack.load(SeqCst) {
            std::hint::spin_loop();
        }
        if i >= warmup_repetitions {
            timing.push(Instant::now() - start);
        }
        ack.store(false, SeqCst);
    }

    timing
}

fn puller(
    mut reader: PipeReader,
    ack: &mut AtomicBool,
    warmup_repetitions: usize,
    repetitions: usize,
    message_size: usize,
) {
    for i in 0..(warmup_repetitions + repetitions) {
        //println!("child iter {}", i);
        let mut message = vec![0_u8; message_size];

        if let Err(e) = reader.read_exact(&mut message) {
            std::hint::cold_path();
            println!("{}", e);
        }

        ack.store(true, SeqCst);
    }
}
