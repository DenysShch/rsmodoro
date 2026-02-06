# Rsmodoro Timer

A simple Pomodoro timer application built with Rust and Slint.

## Features

- **Timer**: Pomodoro timer with customizable work and rest durations
- **Alarm**: Set alarms to notify you at specific times

## Usage

Run the application:
```bash
cargo run
```

The timer includes work sessions, rest breaks, and plays sound notifications when phases complete.

## WM hooks
Hyperland `windows.conf`
```conf
windowrule = float on, match:title rsmodoro
windowrule = pin on, match:title rsmodoro
windowrule = no_initial_focus on, match:title rsmodoro
windowrule = no_dim on, match:title rsmodoro
windowrule = move (monitor_w-window_w-40) (monitor_h/2-window_h/2), match:title rsmodoro
```

