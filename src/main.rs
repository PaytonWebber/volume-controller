use evdev::{Device, InputEventKind, Key};
use std::process::Command;
use std::{thread, time};

fn find_bluetooth_keyboard() -> Option<String> {
    for path in evdev::enumerate() {
        if let Ok(device) = Device::open(path.0.as_path()) {
            if let Some(name) = device.name() {
                if name.contains("Keyboard") {
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
        "up" => &format!("pactl set-sink-volume @DEFAULT_SINK@ +5% && notify-send -r 1234 -u low 'Volume: {}% 🔊'", volume),
        "down" => &format!("pactl set-sink-volume @DEFAULT_SINK@ -5% && notify-send -r 1234 -u low 'Volume: {}% 🔊'", volume),
        "mute" => "pactl set-sink-mute @DEFAULT_SINK@ toggle && notify-send -r 1234 -u low 'Volume: Muted 🔇'",
        _ => return,
    };

    if let Err(err) = Command::new("sh").arg("-c").arg(volume_command).output() {
        eprintln!("Error executing command: {}", err);
    }
}

fn main() {
    let keyboard_path = loop {
        if let Some(path) = find_bluetooth_keyboard() {
            break path;
        } else {
            eprint!("Keyboard not found. Retrying in 30 seconds...");
            thread::sleep(time::Duration::from_secs(30));
        }
    };
    println!("Path found: {}", keyboard_path);
    let mut keyboard = match Device::open(keyboard_path) {
        Ok(device) => device,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    println!("Device created");
    loop {
        let events = match keyboard.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };
        for ev in events {
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
