# LSTimer-Linux

**LSTimer** is a minimal and efficient Rubik's Cube timer for Linux, built in Rust.

## 🚀 Features
- Fast and responsive terminal-based interface
- Accurate WCA-style timing
- Scramble generator
- Session stats tracking (coming soon)
- Bluetooth (coming soon)

## 🛠 Requirements
- Linux (tested on Arch)
- Rust (1.87+)

## Information
When closing, it may say app unresponding but you can ignore that, just click terminate.

## 📦 Installation

```bash
git clone https://github.com/Link-the-Coder/LSTimer-Linux.git
cd LSTimer-Linux
cargo run --release
```

## Create Desktop Shortcut

```bash
nano ~/.local/share/applications/LSTimer.desktop
```
Add this to the .desktop file
**Make sure to change yourname to your name**
```INI
[Desktop Entry]
Name=LSTimer
Comment=Launch LSTimer Rust App
Exec=/home/yourname/LSTimer-Linux/target/release/LSTimer # Your path to the file
Terminal=false
Type=Application
Categories=Utility;
StartupNotify=true
Icon=/home/yourname/LSTimer-Linux/icon.png
```

🧠 Usage

    Press Space to start/stop the timer

    Wait for the green light before starting

    Automatically generates a scramble on each solve

🔒 License

All rights reserved.
You may view the source code but may not redistribute, reuse, or modify it.

Made by Link-the-Coder 💻

