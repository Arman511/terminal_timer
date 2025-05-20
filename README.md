# ⏱️ Terminal Timer with Music

This is a simple terminal-based timer written in Rust. It shows a progress bar for a given duration and plays a random embedded audio track once the timer ends. You can interrupt the timer with `Ctrl+C` or stop the music by pressing `Enter`.

## 📦 Features

-   Custom duration input (e.g. 1h 30m 45s)
-   Embedded audio files for portability
-   Randomized music playback after the timer ends
-   Graceful `Ctrl+C` handling
-   Stylish progress bar using `indicatif`

## Windows MSI Installer

You can now install Terminal Timer easily on Windows using the MSI installer. Make sure if you already have it installed uninstall the old one.

### Installation

1. Download the latest `terminal_timer_windows.msi` file from the [Releases page](https://github.com/Arman511/terminal_timer/releases).
2. Double-click the `.msi` file and follow the installation wizard.
3. The installer will add the Terminal Timer install directory to your user PATH environment variable.

### Running the Installed App

Once installed, you can open a new Command Prompt or PowerShell window and simply run:

```powershell
terminal_timer_windows.exe 90s
```

or just

```powershell
terminal_timer_windows 90s
```

## Examples

**Bash:**

```bash
terminal_timer 90s     # 90 seconds
terminal_timer 2m      # 2 minutes
terminal_timer 1h 10s  # 1 hour, 10 seconds
```

**PowerShell:**

```powershell
terminal_timer 90s     # 90 seconds
terminal_timer 2m      # 2 minutes
terminal_timer 1h 10s  # 1 hour, 10 seconds
```

**Command Prompt:**

```CMD
terminal_timer.exe 90s     # 90 seconds
terminal_timer.exe 2m      # 2 minutes
terminal_timer.exe 1h 10s  # 1 hour, 10 seconds
```

## 🚀 How to Build

1. Clone the repository:

    ```
    git clone https://github.com/Arman511/terminal_timer.git
    ```

2. Run

    ```
    cd terminal_timer
    cargo build --release
    ```

3. This will create an optimized binary at `target/release/terminal_timer` or `target/release/terminal_timer.exe` for Windows.

## ▶️ How to Run

### Run from Source

```bash
cargo run 1h 15m 30s
```

You can also omit the duration and input it interactively:

```bash
cargo run
# Then enter: 2m 10s
```

### Run Built Binary

```bash
./target/release/terminal_timer 45s
```

```powershell
.\target\release\terminal_timer.exe 45s
```

#### (Optional) Add to PATH

If you want to run `terminal_timer` from anywhere, add the binary directory to your system `PATH`:

**On Linux/macOS:**

```bash
echo 'export PATH="$PATH:/path/to/terminal_timer"' >> ~/.bashrc
source ~/.bashrc
```

**On Windows (PowerShell):**

```powershell
$env:Path += ";\path\to\terminal_timer"
# To make it permanent, add the path to your user environment variables.
```

After this, you can simply run:

```bash
terminal_timer 45s
```

or

```powershell
terminal_timer.exe 45s
```

## 📂 Audio Files

Audio files must be stored in the `audio/` directory and will be embedded at compile-time using `include_bytes!`.

E.g.

```
audio/
├── 1.ogg
├── 2.ogg
├── 3.ogg
└── 4.ogg
```

You can replace these with any audio files you like or add more just make sure to number to add them in the code.

## 🛠️ Dependencies

-   **clap** – Command-line parsing
-   **ctrlc** – Ctrl+C signal handling
-   **indicatif** – Progress bars
-   **rand** – Random number generation
-   **rodio** – Audio playback
-   **regex** – Input parsing

Install them automatically with `cargo build`.

## Music Credits

The timer music used in this project is sourced from Pixabay.

-   Music by <a href="https://pixabay.com/users/lesiakower-25701529/?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=168821">Lesiakower</a> from <a href="https://pixabay.com/music//?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=168821">Pixabay</a>
-   Music by <a href="https://pixabay.com/users/dferun-20599211/?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=332384">dferun</a> from <a href="https://pixabay.com//?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=332384">Pixabay</a>
-   Music by <a href="https://pixabay.com/users/ezioblade4life-37897481/?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=155671">Ellis Checkley</a> from <a href="https://pixabay.com//?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=155671">Pixabay</a>
