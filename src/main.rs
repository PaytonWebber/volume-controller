use evdev::{Device, InputEventKind, Key};
use std::{io::Result, process::Command};

fn find_bluetooth_keyboard() -> Option<String> {
    for path in evdev::enumerate() {
        if let Ok(device) = Device::open(path.0.as_path()) {
            if let Some(name) = device.name() {
                if name.contains("Corne Keyboard") {
                    return Some(path.0.as_path().display().to_string());
                }
            }
        }
    }
    None
}

fn get_volume() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(r"pactl get-sink-volume @DEFAULT_SINK@ | grep -oP '\d+%' | head -1 | tr -d '%'")
        .output();
    match output {
        Ok(raw_output) => String::from_utf8(raw_output.stdout)
            .unwrap()
            .trim_end()
            .to_string(),
        Err(e) => {
            eprintln!("Error getting volume {}", e);
            String::from("INVALID")
        }
    }
}

fn change_volume(action: &str) {
    let volume = get_volume();
    let volume_command = match action {
        "up" => &format!("pactl set-sink-volume @DEFAULT_SINK@ +5% && notify-send -u low -t 1500 'Volume: {}% 🔊'", volume),
        "down" => &format!("pactl set-sink-volume @DEFAULT_SINK@ -5% && notify-send -u low -t 1500 'Volume: {}% 🔊'", volume),
        "mute" => "pactl set-sink-mute @DEFAULT_SINK@ toggle && notify-send -u low -t 1500 'Volume: Muted 🔇'",
        _ => return,
    };

    if let Err(err) = Command::new("sh").arg("-c").arg(volume_command).output() {
        eprintln!("Error executing command: {}", err);
    }
}

fn main() -> Result<()> {
    let path = find_bluetooth_keyboard().unwrap();
    let mut keyboard = Device::open(path)?;
    loop {
        for ev in keyboard.fetch_events()? {
            if ev.value() == 0 {
                continue;
            }
            match ev.kind() {
                InputEventKind::Key(Key::KEY_VOLUMEUP) => {
                    change_volume("up");
                }
                InputEventKind::Key(Key::KEY_VOLUMEDOWN) => {
                    change_volume("down");
                }
                InputEventKind::Key(Key::KEY_MUTE) => {
                    change_volume("mute");
                }
                _ => {}
            }
        }
    }
}
