use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::io::Cursor;

const NOTIFICATION_SOUND: &[u8] = include_bytes!("../assets/sounds/bell.mp3");
const ALARM_SOUND: &[u8] = include_bytes!("../assets/sounds/alarm.mp3");

pub fn play_notification() {
    play_sound(NOTIFICATION_SOUND);
}

pub fn play_alarm() {
    play_sound(ALARM_SOUND);
}

fn play_sound(sound_data: &'static [u8]) {
    std::thread::spawn(move || {
        let Ok(stream) = OutputStreamBuilder::open_default_stream() else {
            return;
        };
        let Ok(source) = Decoder::new(Cursor::new(sound_data)) else {
            return;
        };
        let mut s = stream;
        s.log_on_drop(false);
        let sink = Sink::connect_new(s.mixer());
        sink.append(source);
        sink.sleep_until_end();
    });
}
