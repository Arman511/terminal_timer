use clap::Parser;
use ctrlc;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use once_cell::sync::Lazy;
use rand::prelude::*;
use rand::rng;
use regex::Regex;
use rodio::{Decoder, OutputStream, Sink};
use std::sync::Arc;
use std::{
    io::{self, Write},
    thread,
    time::{Duration, Instant},
};
/// terminal timer
#[derive(Parser, Debug, Clone)]
#[command(
    name = "Terminal Timer",
    author,
    version,
    about = "A simple terminal timer with song notification",
    long_about = None,
    help_template = "{name} {version}\n{author}\n{about}\n{usage-heading} {usage}\n\n{all-args}"
)]
struct Args {
    /// Timer duration (e.g. 1h 20m 30s)
    #[arg(value_name = "TIME", required = false)]
    time: Vec<String>,
    /// Message to print after timer ends
    #[arg(short, long, value_name = "MESSAGE", default_value = "''")]
    message: String,
}

static DURATION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+h)?\s*(\d+m)?\s*(\d+s)?\s*(\d+)?$|^\d+$").unwrap());

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
    let mut rng = rng();
    list.choose(&mut rng)
        .expect("Audio list should not be empty")
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
    let source = match Decoder::new(cursor) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("Audio decode error: {e}. Skipping playback.");
            return;
        }
    };
    sink.lock().unwrap().append(source);

    let is_done = Arc::new(AtomicBool::new(false));
    let is_done_clone = Arc::clone(&is_done);
    let sink_clone = Arc::clone(&sink);

    // Thread to wait for audio to finish
    thread::spawn(move || {
        sink_clone.lock().unwrap().sleep_until_end();
        is_done_clone.store(true, Ordering::SeqCst);
    });

    // Thread to wait for user input (Enter)
    let is_done_clone = Arc::clone(&is_done);
    thread::spawn(move || {
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
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
    println!("\n");
}

pub fn parse_duration(input: &str) -> (u64, u64, u64) {
    let mut h;
    let mut m;
    let mut s;
    let mut duration_input = input.to_string();

    loop {
        let mut valid = true;
        if !DURATION_REGEX.is_match(duration_input.trim()) {
            valid = false;
        }

        h = 0;
        m = 0;
        s = 0;
        for part in duration_input.split_whitespace() {
            parse_time_part(part, &mut h, &mut m, &mut s, &mut valid);
        }

        if valid && (h > 0 || m > 0 || s > 0) {
            break;
        } else {
            duration_input = get_user_input(
                "Invalid input or zero duration. Please provide a valid duration (e.g., 1h 20m 30s): ",
            );
        }
    }

    (h, m, s)
}

fn parse_time_part(part: &str, h: &mut u64, m: &mut u64, s: &mut u64, valid: &mut bool) {
    if let Some(val) = part.strip_suffix('h') {
        match val.parse() {
            Ok(val) => *h = val,
            Err(_) => *valid = false,
        }
    } else if let Some(val) = part.strip_suffix('m') {
        match val.parse() {
            Ok(val) => *m = val,
            Err(_) => *valid = false,
        }
    } else if let Some(val) = part.strip_suffix('s') {
        match val.parse() {
            Ok(val) => *s = val,
            Err(_) => *valid = false,
        }
    } else {
        match part.parse() {
            Ok(val) => *s = val,
            Err(_) => *valid = false,
        }
    }
}

fn get_duration_and_message(args: &Args) -> ((u64, u64, u64), String) {
    let input = if args.time.is_empty() {
        get_user_input("Enter duration (e.g. 1h 20m 30s): ")
    } else {
        args.time.join(" ")
    };

    let message = if args.message.is_empty() {
        get_user_input("Enter message (optional, press Enter to skip): ")
    } else {
        args.message.clone()
    };

    let (hours, minutes, seconds) = parse_duration(&input);
    ((hours, minutes, seconds), message)
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn main() {
    let args = Args::parse();
    let global_abort = Arc::new(AtomicBool::new(false));
    {
        let global_abort = Arc::clone(&global_abort);
        ctrlc::set_handler(move || {
            println!("\nInterrupt received! Exiting...");
            global_abort.store(true, Ordering::SeqCst);
            std::process::exit(0);
        })
        .expect("Error setting Ctrl+C handler");
    }
    let ((hours, minutes, seconds), message) = get_duration_and_message(&args);
    let duration = hours * 3600 + minutes * 60 + seconds;
    println!("\nTimer started for {}", format_duration(duration));
    show_progress_bar(duration, Arc::clone(&global_abort));
    if global_abort.load(Ordering::SeqCst) {
        return;
    }
    if !message.trim().is_empty() {
        // Print colored heading and message using ANSI escape codes
        println!("\x1b[1;34mMessage:\x1b[0m \x1b[1;32m{}\x1b[0m\n", message);
    }
    println!("Playing a random song... (press Enter to stop)");
    play_song_with_interrupt(global_abort);
}

pub fn format_duration(duration: u64) -> String {
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

#[cfg(test)]
mod tests;
