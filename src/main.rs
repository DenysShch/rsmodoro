slint::include_modules!();

mod audio;
mod config;

use chrono::{Local, Timelike};
use config::Config;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, PartialEq, Copy)]
enum TimerPhase {
    Idle,
    Work,
    Rest,
    Done,
}

struct Timer {
    phase: TimerPhase,
    timer_duration_mins: u32,
    rest_duration_mins: u32,
    remaining: u32,
    running: bool,
}

impl Timer {
    fn new(timer_duration_mins: u32, rest_duration_mins: u32) -> Self {
        Self {
            phase: TimerPhase::Idle,
            timer_duration_mins,
            rest_duration_mins,
            remaining: timer_duration_mins * 60,
            running: false,
        }
    }
}

struct Alarm {
    hours: u32,
    minutes: u32,
    enabled: bool,
    triggered: bool,
}

impl Alarm {
    fn new(hours: u32, minutes: u32) -> Self {
        Self {
            hours,
            minutes,
            enabled: false,
            triggered: false,
        }
    }

    fn check_and_trigger(&mut self) -> bool {
        if !self.enabled || self.triggered {
            return false;
        }

        let now = Local::now();
        let current_hour = now.hour();
        let current_minute = now.minute();

        if current_hour == self.hours && current_minute == self.minutes {
            self.triggered = true;
            self.enabled = false;
            return true;
        }

        false
    }

    fn set_time(&mut self, hours: u32, minutes: u32) {
        self.hours = hours;
        self.minutes = minutes;
        self.triggered = false;
    }

    fn enable(&mut self) {
        self.enabled = true;
        self.triggered = false;
    }

    fn reset(&mut self) {
        self.enabled = false;
        self.triggered = false;
    }

    /// Returns remaining time in seconds until alarm triggers
    fn remaining_seconds(&self) -> u32 {
        if !self.enabled {
            return 0;
        }

        let now = Local::now();
        let current_hour = now.hour();
        let current_minute = now.minute();
        let current_second = now.second();

        // Calculate target time in seconds from midnight
        let target_seconds = (self.hours * 3600 + self.minutes * 60) as i32;
        // Calculate current time in seconds from midnight
        let current_seconds = (current_hour * 3600 + current_minute * 60 + current_second) as i32;

        // Calculate difference
        let mut diff = target_seconds - current_seconds;

        // If negative, alarm is for tomorrow
        if diff < 0 {
            diff += 24 * 3600; // Add 24 hours
        }

        diff as u32
    }
}

fn main() {
    let ui = AppWindow::new().unwrap();

    // Position window on right side of screen
    let window = ui.window();
    window.set_position(slint::PhysicalPosition::new(
        1920 - 260, // Right edge
        100,        // Near top
    ));

    // Load config
    let config = Config::load();

    // Apply theme from config
    ui.set_theme_bg(parse_color(&config.theme.background_color).into());
    ui.set_theme_input_bg(parse_color(&config.theme.input_bg_color).into());
    ui.set_theme_text(parse_color(&config.theme.text_color).into());
    ui.set_theme_text_dim(parse_color(&config.theme.text_dim_color).into());
    ui.set_theme_icon(parse_color(&config.theme.icon_color).into());
    ui.set_theme_accent(parse_color(&config.theme.accent_color).into());
    ui.set_theme_accent_rest(parse_color(&config.theme.accent_rest_color).into());
    ui.set_theme_transparent(config.theme.transparent);

    let timer = Arc::new(Mutex::new(Timer::new(
        config.timer_duration_minutes,
        config.rest_duration_minutes,
    )));

    // Create alarm instance
    let alarm = Arc::new(Mutex::new(Alarm::new(config.alarm_hour, config.alarm_min)));

    // Update initial display
    {
        let timer_guard = timer.lock().unwrap();
        ui.set_time(format_time(timer_guard.remaining).into());
        ui.set_duration(timer_guard.timer_duration_mins as i32);
        ui.set_rest_duration(timer_guard.rest_duration_mins as i32);
        ui.set_progress(1.0); // Full bar at start
    }

    // Start button
    {
        let t = Arc::clone(&timer);
        let ui_weak = ui.as_weak();
        ui.on_start(move || {
            let mut state = t.lock().unwrap();
            state.running = true;
            state.phase = TimerPhase::Work;
            state.remaining = state.timer_duration_mins * 60;

            let time_str = format_time(state.remaining);
            drop(state);

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_running(true);
                ui.set_finished(false);
                ui.set_is_rest(false);
                ui.set_time(time_str.into());
                ui.set_progress(1.0);
            }
        });
    }

    // Pause button
    {
        let t = Arc::clone(&timer);
        let ui_weak = ui.as_weak();
        ui.on_pause(move || {
            t.lock().unwrap().running = false;
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_running(false);
            }
        });
    }

    // Reset button
    {
        let t = Arc::clone(&timer);
        let ui_weak = ui.as_weak();
        ui.on_reset(move || {
            let mut state = t.lock().unwrap();
            state.running = false;
            state.phase = TimerPhase::Idle;
            state.remaining = state.timer_duration_mins * 60;

            let time_str = format_time(state.remaining);
            drop(state);

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_time(time_str.into());
                ui.set_progress(1.0);
                ui.set_running(false);
                ui.set_finished(false);
                ui.set_is_rest(false);
            }
        });
    }

    // Duration changed (handles both work and rest duration)
    {
        let t = Arc::clone(&timer);
        let ui_weak = ui.as_weak();
        ui.on_duration_changed(move |new_duration, is_rest| {
            let mins = new_duration as u32;

            let should_update_ui = {
                let mut state = t.lock().unwrap();

                if is_rest {
                    state.rest_duration_mins = mins;
                    false // Don't update time display for rest changes
                } else {
                    state.timer_duration_mins = mins;
                    if !state.running {
                        state.remaining = mins * 60;
                        true
                    } else {
                        false
                    }
                }
            }; // Lock released here

            // Load current config, update timer values, and save
            let mut cfg = Config::load();
            let state = t.lock().unwrap();
            cfg.timer_duration_minutes = state.timer_duration_mins;
            cfg.rest_duration_minutes = state.rest_duration_mins;
            drop(state);

            if let Err(e) = cfg.save() {
                eprintln!("Failed to save config: {}", e);
            }

            // Only update time display for work duration changes when not running
            if should_update_ui {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_time(format_time(mins * 60).into());
                    ui.set_progress(1.0);
                }
            }
        });
    }

    // Init alarm
    {
        let alarm_guard = alarm.lock().unwrap();
        ui.set_alarm_hours(alarm_guard.hours as i32);
        ui.set_alarm_minutes(alarm_guard.minutes as i32);
    }

    // Alarm time changed
    {
        let a = Arc::clone(&alarm);
        let ui_weak = ui.as_weak();
        ui.on_alarm_time_changed(move |hours, minutes| {
            let mut alarm_state = a.lock().unwrap();
            alarm_state.set_time(hours as u32, minutes as u32);
            drop(alarm_state);

            let mut cfg = Config::load();
            cfg.alarm_hour = hours as u32;
            cfg.alarm_min = minutes as u32;
            if let Err(e) = cfg.save() {
                eprintln!("Failed to save config: {}", e);
            }
            // Update UI properties so they stay in sync
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_alarm_hours(hours);
                ui.set_alarm_minutes(minutes);
            }
        });
    }

    // Alarm start (enable alarm)
    {
        let a = Arc::clone(&alarm);
        let ui_weak = ui.as_weak();
        ui.on_alarm_start(move || {
            // Get current UI values before enabling
            if let Some(ui) = ui_weak.upgrade() {
                let hours = ui.get_alarm_hours() as u32;
                let minutes = ui.get_alarm_minutes() as u32;

                let mut alarm_state = a.lock().unwrap();
                alarm_state.set_time(hours, minutes);
                alarm_state.enable();
                drop(alarm_state);

                ui.set_alarm_enabled(true);
                ui.set_alarm_triggered(false);
            }
        });
    }

    // Alarm reset
    {
        let a = Arc::clone(&alarm);
        let ui_weak = ui.as_weak();
        ui.on_alarm_reset(move || {
            let mut alarm_state = a.lock().unwrap();
            alarm_state.reset();
            drop(alarm_state);
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_alarm_enabled(false);
                ui.set_alarm_triggered(false);
            }
        });
    }

    // Timer and Alarm thread
    {
        let t = Arc::clone(&timer);
        let a = Arc::clone(&alarm);
        let ui_weak = ui.as_weak();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(1));

                // Check alarm and get remaining time
                let (alarm_triggered, alarm_remaining, alarm_is_enabled) = {
                    let mut alarm_state = a.lock().unwrap();
                    let triggered = alarm_state.check_and_trigger();
                    let remaining = alarm_state.remaining_seconds();
                    let enabled = alarm_state.enabled;
                    (triggered, remaining, enabled)
                };

                // Update alarm UI
                if alarm_is_enabled || alarm_triggered {
                    let remaining_str = format_time_hms(alarm_remaining);
                    let ui_weak = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            if alarm_triggered {
                                ui.set_alarm_enabled(false);
                                ui.set_alarm_triggered(true);
                            }
                            ui.set_alarm_remaining(remaining_str.into());
                        }
                    })
                    .ok();
                }

                if alarm_triggered {
                    audio::play_alarm();
                }

                // Check if timer is running
                let should_tick = t.lock().unwrap().running;
                if !should_tick {
                    continue;
                }

                let (time_str, progress, is_running, phase) = {
                    let mut state = t.lock().unwrap();

                    //Decrement timer
                    if state.remaining > 0 {
                        state.remaining -= 1;
                    }

                    //Check phase
                    if state.remaining == 0 {
                        match state.phase {
                            TimerPhase::Work => {
                                state.phase = TimerPhase::Rest;
                                state.remaining = state.rest_duration_mins * 60;
                                audio::play_notification();
                            }
                            TimerPhase::Rest => {
                                state.phase = TimerPhase::Done;
                                state.running = false;
                                audio::play_notification();
                            }
                            TimerPhase::Done => {
                                state.phase = TimerPhase::Idle;
                            }
                            TimerPhase::Idle => {}
                        }
                    }

                    //Calculate progress
                    let total = match state.phase {
                        TimerPhase::Work | TimerPhase::Idle | TimerPhase::Done => {
                            state.timer_duration_mins * 60
                        }
                        TimerPhase::Rest => state.rest_duration_mins * 60,
                    };
                    let progress = if total > 0 {
                        state.remaining as f32 / total as f32
                    } else {
                        1.0
                    };

                    (
                        format_time(state.remaining),
                        progress,
                        state.running,
                        state.phase,
                    )
                };
                let ui_weak = ui_weak.clone();
                let is_finished = phase == TimerPhase::Done;
                let is_rest = phase == TimerPhase::Rest;
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_time(time_str.into());
                        ui.set_progress(progress);
                        ui.set_running(is_running);
                        ui.set_finished(is_finished);
                        ui.set_is_rest(is_rest);
                    }
                })
                .ok();
            }
        });
    }

    ui.run().unwrap();
}

fn format_time(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn format_time_hms(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

/// Parse a hex color string (e.g., "#ff0000" or "ff0000") into a slint::Color
fn parse_color(hex: &str) -> slint::Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    slint::Color::from_rgb_u8(r, g, b)
}
