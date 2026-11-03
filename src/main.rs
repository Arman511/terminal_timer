use clap::Parser;
use console::style;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rand::seq::{IndexedRandom, SliceRandom};
use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::error::Error;
use std::fmt;
use std::io::{self, Cursor, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

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
    #[arg(short, long, value_name = "MESSAGE", default_value = "")]
    message: String,
    /// Don't play a song when the timer ends
    #[arg(short, long)]
    no_audio: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseDurationError {
    Empty,
    Zero,
    InvalidToken(String),
    DuplicateUnit(char),
    BareNumberWithUnits,
    Overflow,
}

impl fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "no duration provided"),
            Self::Zero => write!(f, "duration must be greater than zero"),
            Self::InvalidToken(t) => write!(f, "invalid token '{t}'"),
            Self::DuplicateUnit(u) => write!(f, "unit '{u}' specified more than once"),
            Self::BareNumberWithUnits => {
                write!(
                    f,
                    "a number without a unit cannot be combined with other parts"
                )
            }
            Self::Overflow => write!(f, "duration is too large"),
        }
    }
}

impl Error for ParseDurationError {}

// Embed your audio files as byte arrays:
const AUDIO_1: &[u8] = include_bytes!("audio/1.ogg");
const AUDIO_2: &[u8] = include_bytes!("audio/2.ogg");
const AUDIO_3: &[u8] = include_bytes!("audio/3.ogg");
const AUDIO_4: &[u8] = include_bytes!("audio/4.ogg");

const AUDIO_LIST: [&[u8]; 4] = [AUDIO_1, AUDIO_2, AUDIO_3, AUDIO_4];

const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Folder next to the executable that may hold custom songs.
const CUSTOM_AUDIO_DIR: &str = "tt_audio";
const CUSTOM_AUDIO_EXTENSIONS: [&str; 4] = ["ogg", "mp3", "wav", "flac"];

fn custom_audio_files() -> Vec<PathBuf> {
    let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join(CUSTOM_AUDIO_DIR)))
    else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| CUSTOM_AUDIO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        })
        .collect()
}

/// Picks a random decodable song from `tt_audio`, falling back to the embedded songs.
fn pick_audio() -> Option<Decoder<Cursor<Vec<u8>>>> {
    let mut rng = rand::rng();
    let mut custom = custom_audio_files();
    custom.shuffle(&mut rng);
    for path in custom {
        let decoded = std::fs::read(&path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| Decoder::new(Cursor::new(bytes)).map_err(|e| e.to_string()));
        match decoded {
            Ok(src) => return Some(src),
            Err(e) => eprintln!("Skipping {}: {e}", path.display()),
        }
    }
    let data = AUDIO_LIST.choose(&mut rng)?;
    Decoder::new(Cursor::new(data.to_vec())).ok()
}

/// While a prompt is blocking on stdin the main thread cannot observe the abort flag,
/// so the Ctrl+C handler exits directly in that case.
static PROMPTING: AtomicBool = AtomicBool::new(false);

/// Restores the terminal when dropped, even on early returns.
struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> Option<Self> {
        terminal::enable_raw_mode().ok().map(|()| Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

/// Waits up to `timeout` for Enter or Ctrl+C. Returns true if playback should stop.
fn stop_requested(timeout: Duration) -> bool {
    match event::poll(timeout) {
        Ok(true) => matches!(
            event::read(),
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release
                && (key.code == KeyCode::Enter
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)))
        ),
        Ok(false) => false,
        Err(_) => {
            // No usable keyboard input (e.g. stdin is not a terminal): just wait for the audio.
            thread::sleep(timeout);
            false
        }
    }
}

fn play_song_with_interrupt(global_abort: &AtomicBool) {
    let mut device_sink = match DeviceSinkBuilder::open_default_sink() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Audio output error: {e}. Skipping playback.");
            return;
        }
    };
    device_sink.log_on_drop(false);
    let sink = Player::connect_new(device_sink.mixer());

    let source = match pick_audio() {
        Some(src) => src,
        None => {
            eprintln!("Audio decode error. Skipping playback.");
            return;
        }
    };
    sink.append(source);
    sink.play();

    let _raw_mode = RawModeGuard::enable();
    while !sink.empty() && !global_abort.load(Ordering::Relaxed) {
        if stop_requested(POLL_INTERVAL) {
            break;
        }
    }
    sink.stop();
}

fn show_progress_bar(duration: Duration, global_abort: &AtomicBool) {
    let total_ms = u64::try_from(duration.as_millis()).unwrap_or(u64::MAX);
    let pb = ProgressBar::new(total_ms);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% ({eta})",
        )
        .expect("valid progress template")
        .with_key(
            "eta",
            |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                let _ = write!(w, "{}", format_duration(state.eta().as_secs()));
            },
        )
        .progress_chars("#>-"),
    );
    pb.enable_steady_tick(POLL_INTERVAL);

    let start = Instant::now();
    loop {
        if global_abort.load(Ordering::Relaxed) {
            pb.finish_with_message("Timer interrupted!");
            return;
        }
        let elapsed = start.elapsed();
        if elapsed >= duration {
            break;
        }
        pb.set_position(u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX));
        thread::sleep(POLL_INTERVAL.min(duration - elapsed));
    }
    pb.finish_with_message("Time's up!");
    println!();
}

/// Parses a duration such as `1h 20m 30s`, `1h20m` or a bare number of seconds.
pub fn parse_duration(input: &str) -> Result<Duration, ParseDurationError> {
    let input = input.trim().to_lowercase();
    if input.is_empty() {
        return Err(ParseDurationError::Empty);
    }

    let mut parts: [Option<u64>; 3] = [None; 3];
    let mut bare: Option<u64> = None;
    let mut token_count = 0;
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let mut digits = String::new();
        while let Some(&d) = chars.peek() {
            if d.is_ascii_digit() {
                digits.push(d);
                chars.next();
            } else {
                break;
            }
        }
        let unit = match chars.peek() {
            Some(&u) if matches!(u, 'h' | 'm' | 's') && !digits.is_empty() => {
                chars.next();
                Some(u)
            }
            _ => None,
        };
        // A token must end at whitespace, end of input, or directly before the next number.
        let malformed = digits.is_empty()
            || matches!(chars.peek(), Some(&n) if !n.is_whitespace() && !n.is_ascii_digit());
        if malformed {
            let mut bad = digits;
            if let Some(u) = unit {
                bad.push(u);
            }
            while let Some(&n) = chars.peek() {
                if n.is_whitespace() {
                    break;
                }
                bad.push(n);
                chars.next();
            }
            return Err(ParseDurationError::InvalidToken(bad));
        }
        let value: u64 = digits.parse().map_err(|_| ParseDurationError::Overflow)?;
        token_count += 1;
        match unit {
            Some(u) => {
                let idx = "hms".find(u).expect("unit is h, m or s");
                if parts[idx].replace(value).is_some() {
                    return Err(ParseDurationError::DuplicateUnit(u));
                }
            }
            None => {
                if bare.replace(value).is_some() {
                    return Err(ParseDurationError::BareNumberWithUnits);
                }
            }
        }
    }

    let total = match bare {
        Some(secs) if token_count == 1 => secs,
        Some(_) => return Err(ParseDurationError::BareNumberWithUnits),
        None => {
            let [h, m, s] = parts.map(|p| p.unwrap_or(0));
            h.checked_mul(3600)
                .and_then(|h| m.checked_mul(60).and_then(|m| h.checked_add(m)))
                .and_then(|hm| hm.checked_add(s))
                .ok_or(ParseDurationError::Overflow)?
        }
    };

    // The progress bar works in milliseconds.
    if total.checked_mul(1000).is_none() {
        return Err(ParseDurationError::Overflow);
    }
    if total == 0 {
        return Err(ParseDurationError::Zero);
    }
    Ok(Duration::from_secs(total))
}

fn get_user_input(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    PROMPTING.store(true, Ordering::Relaxed);
    let read = io::stdin().read_line(&mut input);
    PROMPTING.store(false, Ordering::Relaxed);
    if read? == 0 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "input closed"));
    }
    Ok(input.trim().to_string())
}

fn get_duration_and_message(args: &Args) -> Result<(Duration, String), Box<dyn Error>> {
    let duration = if args.time.is_empty() {
        // Interactive: keep asking until the input is valid.
        let mut input = get_user_input("Enter duration (e.g. 1h 20m 30s): ")?;
        loop {
            match parse_duration(&input) {
                Ok(d) => break d,
                Err(e) => {
                    eprintln!("Invalid duration: {e}.");
                    input = get_user_input("Please provide a valid duration (e.g., 1h 20m 30s): ")?;
                }
            }
        }
    } else {
        // Supplied on the command line: an invalid value is a hard error.
        parse_duration(&args.time.join(" "))?
    };

    let message = if args.message.is_empty() {
        match get_user_input("Enter message (optional, press Enter to skip): ") {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => String::new(),
            Err(e) => return Err(e.into()),
        }
    } else {
        args.message.clone()
    };

    Ok((duration, message))
}

fn main() -> ExitCode {
    let args = Args::parse();
    let global_abort = Arc::new(AtomicBool::new(false));
    {
        let global_abort = Arc::clone(&global_abort);
        let handler = move || {
            global_abort.store(true, Ordering::Relaxed);
            if PROMPTING.load(Ordering::Relaxed) {
                println!("\nInterrupt received! Exiting...");
                std::process::exit(130);
            }
        };
        if let Err(e) = ctrlc::set_handler(handler) {
            eprintln!("Error setting Ctrl+C handler: {e}");
            return ExitCode::FAILURE;
        }
    }

    let (duration, message) = match get_duration_and_message(&args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };
    if global_abort.load(Ordering::Relaxed) {
        println!("\nInterrupt received! Exiting...");
        return ExitCode::from(130);
    }

    println!(
        "\nTimer started for {}",
        format_duration(duration.as_secs())
    );
    show_progress_bar(duration, &global_abort);
    if global_abort.load(Ordering::Relaxed) {
        println!("\nInterrupt received! Exiting...");
        return ExitCode::from(130);
    }
    if !message.trim().is_empty() {
        println!(
            "{} {}\n",
            style("Message:").blue().bold(),
            style(&message).green().bold()
        );
    }
    if !args.no_audio {
        println!("Playing a random song... (press Enter to stop)");
        play_song_with_interrupt(&global_abort);
    }
    ExitCode::SUCCESS
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
