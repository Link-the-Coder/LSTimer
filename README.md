# LSTimer-Linux

# **Overview**
![Timer Screenshot](screenshots/overview.png)

**LSTimer** is a minimal and efficient Rubik's Cube timer for Linux, built in Rust.

## 🚀 Features
- Fast and responsive terminal-based interface
- Accurate WCA-style timing
- Scramble generator for each event
- Session stats tracking (coming soon)
- Bluetooth (coming soon)

## 🛠 Requirements
- Linux (tested on Arch with Wayland, KDE Plasma 6 and Hyprland)
- Rust (1.87+)

# **Custom Settings**
![Timer Screenshot](screenshots/settings.png)

# **Multiple events**
![Timer Screenshot](screenshots/multiple-events.png)

# **Detailed Stats**
![Timer Screenshot](screenshots/detailed-stats.png)

## Information
**When closing, it may say app unresponding, ignore that, click terminate!!**

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
Add this to the .desktop file.
**Make sure to change yourname to your actual username!**
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

    Wait for 2 seconds before releasing spacebar

    Automatically generates a scramble on each solve

    Scramble changes for each event

🔒 License

All rights reserved.
You may view the source code but may not redistribute, reuse, or modify it.

Made by Link-the-Coder 💻

