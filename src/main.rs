use clap::Parser;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use std::{
    fs::File,
    io::{self, BufReader, Write},
    thread,
    time::{Duration, Instant},
};

/// Simple terminal timer
#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    time: Option<u64>,
}
use std::io::Cursor;

// Embed your audio files as byte arrays:
const AUDIO_1: &[u8] = include_bytes!("audio/1.ogg");
const AUDIO_2: &[u8] = include_bytes!("audio/2.ogg");
const AUDIO_3: &[u8] = include_bytes!("audio/3.ogg");
const AUDIO_4: &[u8] = include_bytes!("audio/4.ogg");

fn play_song_with_interrupt() {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use std::thread;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(std::sync::Mutex::new(Sink::try_new(&stream_handle).unwrap()));

    let n = rand::thread_rng().gen_range(1..=4);
    
    // Select the embedded audio bytes based on random number
    let audio_data = match n {
        1 => AUDIO_1,
        2 => AUDIO_2,
        3 => AUDIO_3,
        4 => AUDIO_4,
        _ => AUDIO_1,
    };

    // Create a Cursor over the byte slice so Decoder can read it
    let cursor = Cursor::new(audio_data);
    let source = Decoder::new(cursor).unwrap();
    sink.lock().unwrap().append(source);

    let is_done = Arc::new(AtomicBool::new(false));
    let is_done_clone = Arc::clone(&is_done);
    let sink_clone = Arc::clone(&sink);

    thread::spawn(move || {
        sink_clone.lock().unwrap().sleep_until_end();
        is_done_clone.store(true, Ordering::SeqCst);
    });

    let is_done_clone = Arc::clone(&is_done);
    thread::spawn(move || {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        is_done_clone.store(true, Ordering::SeqCst);
    });

    let sink = sink.lock().unwrap();
    sink.play();
    while !is_done.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }
    sink.stop();
}


fn show_progress_bar(seconds: u64) {
    let total_ms = seconds * 1000;
    let pb = ProgressBar::new(total_ms);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );

    let start = Instant::now();
    while pb.position() < total_ms {
        let elapsed = start.elapsed().as_millis() as u64;
        pb.set_position(elapsed.min(total_ms));
        thread::sleep(Duration::from_millis(50));
    }

    pb.finish_with_message("Time's up!");
}

fn main() {
    let args = Args::parse();

    let duration = match args.time {
        Some(t) => t,
        None => {
            print!("How many seconds should the timer be? ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().parse().expect("Please enter a valid number")
        }
    };

    println!("Timer started for {} seconds.", duration);
    show_progress_bar(duration);

    println!("Playing a random song... (press Enter to stop)");
    play_song_with_interrupt();


}