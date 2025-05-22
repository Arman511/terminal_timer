# ⏱️ Terminal Timer with Music

## 📖 Table of Contents

1. [Features](#-features)
2. [Windows MSI Installer](#-windows-msi-installer)
    - [Installation](#installation)
    - [Running the Installed App](#running-the-installed-app)
3. [Usage Examples](#-usage-examples)
    - [With a Message](#with-a-message)
    - [Without a Message](#without-a-message)
    - [Interactive Mode](#interactive-mode)
    - [PowerShell](#powershell)
    - [Command Prompt](#command-prompt)
4. [How to Build](#-how-to-build)
    - [Run from Source](#run-from-source)
    - [Run Built Binary](#run-built-binary)
5. [Audio Files](#-audio-files)
6. [Dependencies](#-dependencies)
7. [Music Credits](#-music-credits)
8. [Additional Tests](#-additional-tests)

## 📦 Features

-   Custom duration input (e.g. `1h 30m 45s`)
-   Optional message/reminder after timer ends (e.g. `--message "Wash car"`)
-   Embedded audio files for portability
-   Randomized music playback after the timer ends
-   Graceful `Ctrl+C` handling
-   Stylish progress bar using `indicatif`

## Windows MSI Installer

You can now install Terminal Timer easily on Windows using the MSI installer. If you already have it installed, uninstall the old one first.

### Installation

1. Download the latest `terminal_timer_windows.msi` from the [Releases page](https://github.com/Arman511/terminal_timer/releases).
2. Double-click the `.msi` file and follow the installation wizard.
3. The installer will add the Terminal Timer install directory to your user PATH environment variable.

### Running the Installed App

Once installed, open a new Command Prompt or PowerShell window and simply run:

```powershell
terminal_timer.exe 90s -m "Take a break!"
```

or just

```powershell
terminal_timer 90s
```

## 📝 Usage Examples

### With a Message

```bash
terminal_timer 1h --message "Wash car"
terminal_timer 20m -m "Take a break!"
```

### Without a Message

```bash
terminal_timer 90s
terminal_timer 2m
terminal_timer 1h 10s
```

### Interactive Mode

```bash
terminal_timer
# Enter duration (e.g. 1h 20m 30s): 10m
# Enter message (optional, press Enter to skip): Make tea
```

### PowerShell

```powershell
terminal_timer 1h 10s -m "Check laundry"
```

### Command Prompt

```cmd
terminal_timer.exe 2m --message "Stretch!"
```

## 🚀 How to Build

1.  Clone the repository:

    ```powershell
    git clone https://github.com/Arman511/terminal_timer.git
    cd terminal_timer
    ```

2.  Build the project:

    ```powershell
    cargo build --release
    ```

3.  The optimized binary will be at `target/release/terminal_timer` (Linux/macOS) or `target/release/terminal_timer.exe` (Windows).

### Run from Source

```bash
cargo run 1h 15m 30s --message "Start dinner"
```

Or omit the duration and input it interactively:

```bash
cargo run
# Enter duration (e.g. 2m 10s)
# Enter message (optional, press Enter to skip):
```

### Run Built Binary

```bash
./target/release/terminal_timer 45s -m "Feed the cat"
```

```powershell
.\target\release\terminal_timer.exe 45s --message "Feed the cat"
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
terminal_timer 45s -m "Do pushups"
```

or

```powershell
terminal_timer.exe 45s --message "Do pushups"
```

## 📂 Audio Files

Audio files must be stored in the `audio/` directory and will be embedded at compile-time using `include_bytes!`.

```
audio/
├── 1.ogg
├── 2.ogg
├── 3.ogg
└── 4.ogg
```

You can replace these with any audio files you like or add more—just make sure to update the code if you add more.

## 🛠️ Dependencies

-   **clap** – Command-line parsing
-   **ctrlc** – Ctrl+C signal handling
-   **indicatif** – Progress bars
-   **rand** – Random number generation
-   **rodio** – Audio playback
-   **regex** – Input parsing

Install them automatically with `cargo build`.

## 🎵 Music Credits

The timer music used in this project is sourced from Pixabay.

-   Music by [Lesiakower](https://pixabay.com/users/lesiakower-25701529/?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=168821) from [Pixabay](https://pixabay.com/music//?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=168821)
-   Music by [dferun](https://pixabay.com/users/dferun-20599211/?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=332384) from [Pixabay](https://pixabay.com//?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=332384)
-   Music by [Ellis Checkley](https://pixabay.com/users/ezioblade4life-37897481/?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=155671) from [Pixabay](https://pixabay.com//?utm_source=link-attribution&utm_medium=referral&utm_campaign=music&utm_content=155671)

## Additional Tests

### Unit Tests

-   **`parse_duration`**:

    -   Handles valid inputs with hours, minutes, and seconds.
    -   Handles plain numeric inputs as seconds.
    -   Rejects invalid formats and empty inputs.
    -   Handles large values for hours, minutes, and seconds.

-   **`format_duration`**:
    -   Formats durations with hours, minutes, and seconds.
    -   Handles zero duration.
    -   Handles large durations.

### Example Commands

-   Start a timer for 1 hour, 20 minutes, and 30 seconds:

    ```bash
    cargo run -- --time "1h 20m 30s"
    ```

-   Start a timer for 45 seconds with a custom message:

    ```bash
    cargo run -- --time "45" --message "Break time!"
    ```

-   Interactive mode:
    ```bash
    cargo run
    ```
    Enter the duration and message when prompted.
