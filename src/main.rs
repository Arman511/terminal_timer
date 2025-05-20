use clap::Parser;
use ctrlc;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use std::sync::Arc;
use std::{
    io::{self, Write},
    thread,
    time::{Duration, Instant},
};
/// Simple terminal timer
#[derive(Parser)]
struct Args {
    #[clap(value_parser)]
    time: Vec<String>,
}

use regex::Regex;
use std::io::Cursor;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
// Embed your audio files as byte arrays:
const AUDIO_1: &[u8] = include_bytes!("audio/1.ogg");
const AUDIO_2: &[u8] = include_bytes!("audio/2.ogg");
const AUDIO_3: &[u8] = include_bytes!("audio/3.ogg");
const AUDIO_4: &[u8] = include_bytes!("audio/4.ogg");

const AUDIO_LIST: [&[u8]; 4] = [AUDIO_1, AUDIO_2, AUDIO_3, AUDIO_4];

fn random_choice<'a, T>(list: &'a [T]) -> &'a T {
    let mut rng = rand::rng();
    let idx = rng.random_range(0..list.len());
    &list[idx]
}

fn play_song_with_interrupt(global_abort: Arc<AtomicBool>) {
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Audio output error: {e}. Skipping playback.");
            return;
        }
    };

    let sink = match Sink::try_new(&stream_handle) {
        Ok(s) => Arc::new(std::sync::Mutex::new(s)),
        Err(e) => {
            eprintln!("Sink creation error: {e}. Skipping playback.");
            return;
        }
    };

    let audio_data = random_choice(&AUDIO_LIST);

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
    while !is_done.load(Ordering::SeqCst) && !global_abort.load(Ordering::SeqCst) && !sink.empty() {
        thread::sleep(Duration::from_millis(100));
    }
    sink.stop();
}

fn show_progress_bar(seconds: u64, global_abort: Arc<AtomicBool>) {
    let total_ms = seconds * 1000;
    let pb = ProgressBar::new(total_ms);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% ({eta})",
        )
        .unwrap()
        .with_key(
            "eta",
            |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                let secs = state.eta().as_secs();
                let formatted = format_duration(secs);
                write!(w, "{}", formatted).unwrap();
            },
        )
        .progress_chars("#>-"),
    );

    let start = Instant::now();
    while pb.position() < total_ms {
        if global_abort.load(Ordering::SeqCst) {
            pb.finish_with_message("Timer interrupted!");
            return;
        }

        let elapsed = start.elapsed().as_millis() as u64;
        pb.set_position(elapsed.min(total_ms));
        thread::sleep(Duration::from_millis(50));
    }

    pb.finish_with_message("Time's up!");
}

fn parse_duration(input: &str) -> (u64, u64, u64) {
    let mut h;
    let mut m;
    let mut s;
    let mut string_input = input.to_string();

    let re = Regex::new(r"^(\d+h)?\s*(\d+m)?\s*(\d+s)?\s*$").unwrap();
    loop {
        let mut valid = true;

        if !re.is_match(string_input.trim()) {
            valid = false;
        }
        h = 0;
        m = 0;
        s = 0;
        for part in string_input.split_whitespace() {
            if let Some(val) = part.strip_suffix('h') {
                match val.parse() {
                    Ok(val) => h = val,
                    Err(_) => valid = false,
                }
            } else if let Some(val) = part.strip_suffix('m') {
                match val.parse() {
                    Ok(val) => m = val,
                    Err(_) => valid = false,
                }
            } else if let Some(val) = part.strip_suffix('s') {
                match val.parse() {
                    Ok(val) => s = val,
                    Err(_) => valid = false,
                }
            } else {
                match part.parse() {
                    Ok(val) => s = val,
                    Err(_) => valid = false,
                }
            }
        }
        if valid && (h > 0 || m > 0 || s > 0) {
            break;
        } else {
            print!("Invalid format, timer cannot be 0. Please enter duration (e.g. 1h 20m 30s): ");
            io::stdout().flush().unwrap();
            let mut new_input = String::new();
            io::stdin().read_line(&mut new_input).unwrap();
            string_input = new_input.trim().to_string();
        }
    }

    (h, m, s)
}

fn main() {
    let args = Args::parse();

    let global_abort = Arc::new(AtomicBool::new(false)); // <-- Track Ctrl+C
    {
        let global_abort = Arc::clone(&global_abort);
        ctrlc::set_handler(move || {
            println!("\nInterrupt received! Exiting...");
            global_abort.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl+C handler");
    }

    let (hours, minutes, seconds) = get_duration(args);

    let duration = hours * 3600 + minutes * 60 + seconds;
    println!("Timer started for {}", format_duration(duration));
    show_progress_bar(duration, Arc::clone(&global_abort));

    if global_abort.load(Ordering::SeqCst) {
        return;
    }

    println!("Playing a random song... (press Enter to stop)");
    play_song_with_interrupt(global_abort);
}

fn format_duration(duration: u64) -> String {
    let h = duration / 3600;
    let m = (duration % 3600) / 60;
    let s = duration % 60;
    if h > 0 {
        return format!("{}h {}m {}s", h, m, s);
    } else if m > 0 {
        return format!("{}m {}s", m, s);
    }
    format!("{}s", s)
}

fn get_duration(args: Args) -> (u64, u64, u64) {
    let input = if args.time.is_empty() {
        print!("Enter duration (e.g. 1h 20m 30s): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    } else {
        args.time.join(" ")
    };

    let (hours, minutes, seconds) = parse_duration(&input);
    (hours, minutes, seconds)
}
