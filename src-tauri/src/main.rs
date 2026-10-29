#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rand::Rng;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy)]
struct Field {
    section: &'static str,
    key: &'static str,
    id: &'static str,
    default: &'static str,
    kind: FieldKind,
}

#[derive(Clone, Copy)]
enum FieldKind {
    Select(&'static [&'static str]),
    Toggle,
    Integer(f64, f64),
    Decimal(f64, f64),
    AspectRatio,
    Multiplier(f64, f64),
    Key,
    Resolution,
    RenderResolution,
    Text,
}

const MOUSE_BUTTON_OPTIONS: &[&str] = &[
    "None",
    "A",
    "B",
    "X",
    "Y",
    "L Stick",
    "R Stick",
    "L",
    "R",
    "ZL",
    "ZR",
    "Plus",
    "Minus",
    "D-pad Left",
    "D-pad Up",
    "D-pad Right",
    "D-pad Down",
];

const FIELDS: &[Field] = &[
    Field {
        section: "Display",
        key: "Mode",
        id: "mode",
        default: "Borderless",
        kind: FieldKind::Select(&["Borderless", "Fullscreen", "Windowed"]),
    },
    Field {
        section: "Display",
        key: "Resolution",
        id: "resolution",
        default: "1920x1080",
        kind: FieldKind::Resolution,
    },
    Field {
        section: "Display",
        key: "RenderResolution",
        id: "renderResolution",
        default: "Auto",
        kind: FieldKind::RenderResolution,
    },
    Field {
        section: "Display",
        key: "AspectRatio",
        id: "aspectRatio",
        default: "Auto",
        kind: FieldKind::AspectRatio,
    },
    Field {
        section: "Display",
        key: "FOV",
        id: "fov",
        default: "1.0",
        kind: FieldKind::Decimal(0.5, 2.0),
    },
    Field {
        section: "Display",
        key: "UIAnchor",
        id: "uiAnchor",
        default: "Center",
        kind: FieldKind::Select(&["Center", "21:9", "32:9", "Edges"]),
    },
    Field {
        section: "Anti-aliasing",
        key: "MSAA",
        id: "msaa",
        default: "Off",
        kind: FieldKind::Select(&["Off", "2x", "4x", "8x"]),
    },
    Field {
        section: "Anti-aliasing",
        key: "FXAA",
        id: "fxaa",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Anti-aliasing",
        key: "SSAO",
        id: "ssao",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Upscaler",
        key: "FSR",
        id: "fsr",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Upscaler",
        key: "Upscaler",
        id: "fsrUpscaler",
        default: "FSR3",
        kind: FieldKind::Select(&["FSR3", "FSR4"]),
    },
    Field {
        section: "Upscaler",
        key: "FSRQuality",
        id: "fsrQuality",
        default: "NativeAA",
        kind: FieldKind::Select(&[
            "NativeAA",
            "Quality",
            "Balanced",
            "Performance",
            "UltraPerformance",
        ]),
    },
    Field {
        section: "Upscaler",
        key: "Sharpness",
        id: "fsrSharpness",
        default: "0.8",
        kind: FieldKind::Decimal(0.0, 1.0),
    },
    Field {
        section: "Upscaler",
        key: "ObjectMotion",
        id: "fsrObjectMotion",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Upscaler",
        key: "Reactive",
        id: "fsrReactive",
        default: "0.9",
        kind: FieldKind::Decimal(0.0, 1.0),
    },
    Field {
        section: "Upscaler",
        key: "AnimSurf",
        id: "fsrAnimSurf",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Graphics",
        key: "DrawDistance",
        id: "drawDistance",
        default: "1.0",
        kind: FieldKind::Decimal(0.0, 10.0),
    },
    Field {
        section: "Performance",
        key: "FPS",
        id: "fps",
        default: "60",
        kind: FieldKind::Integer(30.0, 240.0),
    },
    Field {
        section: "Performance",
        key: "VSync",
        id: "vsync",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Performance",
        key: "ShaderPrewarm",
        id: "shaderPrewarm",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Controller",
        key: "FaceButtons",
        id: "faceButtons",
        default: "Position",
        kind: FieldKind::Select(&["Position", "Label"]),
    },
    Field {
        section: "Controller",
        key: "BackgroundInput",
        id: "backgroundInput",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Controller",
        key: "Rumble",
        id: "rumble",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Controls",
        key: "ComboInputWindow",
        id: "comboInputWindow",
        default: "Lenient",
        kind: FieldKind::Select(&["Lenient", "Strict"]),
    },
    Field {
        section: "Audio",
        key: "MuteInBackground",
        id: "muteInBackground",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Keyboard",
        key: "DpadUp",
        id: "keyDpadUp",
        default: "1",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "DpadUpAlt",
        id: "keyDpadUpAlt",
        default: "Up",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "DpadRight",
        id: "keyDpadRight",
        default: "2",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "DpadRightAlt",
        id: "keyDpadRightAlt",
        default: "Right",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "DpadDown",
        id: "keyDpadDown",
        default: "3",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "DpadDownAlt",
        id: "keyDpadDownAlt",
        default: "Down",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "DpadLeft",
        id: "keyDpadLeft",
        default: "4",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "DpadLeftAlt",
        id: "keyDpadLeftAlt",
        default: "Left",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonB",
        id: "keyButtonB",
        default: "Space",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonBAlt",
        id: "keyButtonBAlt",
        default: "Backspace",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonX",
        id: "keyButtonX",
        default: "G",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonXAlt",
        id: "keyButtonXAlt",
        default: "H",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonY",
        id: "keyButtonY",
        default: "E",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonYAlt",
        id: "keyButtonYAlt",
        default: "T",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonL",
        id: "keyButtonL",
        default: "Q",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonLAlt",
        id: "keyButtonLAlt",
        default: "U",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonR",
        id: "keyButtonR",
        default: "Left Shift",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonRAlt",
        id: "keyButtonRAlt",
        default: "Right Shift",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonZL",
        id: "keyButtonZl",
        default: "Tab",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonZLAlt",
        id: "keyButtonZlAlt",
        default: "Z",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonZR",
        id: "keyButtonZr",
        default: "R",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonZRAlt",
        id: "keyButtonZrAlt",
        default: "B",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonPlus",
        id: "keyButtonPlus",
        default: "Keypad Enter",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonPlusAlt",
        id: "keyButtonPlusAlt",
        default: "Escape",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonPlusAlt2",
        id: "keyButtonPlusAlt2",
        default: "None",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonMinus",
        id: "keyButtonMinus",
        default: "C",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonMinusAlt",
        id: "keyButtonMinusAlt",
        default: "V",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonLeftStick",
        id: "keyButtonLeftStick",
        default: "Left Ctrl",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonLeftStickAlt",
        id: "keyButtonLeftStickAlt",
        default: "Left Alt",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonA",
        id: "keyButtonA",
        default: "F",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonAAlt",
        id: "keyButtonAAlt",
        default: "Return",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonRightStick",
        id: "keyButtonRightStick",
        default: "Right Ctrl",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "ButtonRightStickAlt",
        id: "keyButtonRightStickAlt",
        default: "Right Alt",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "StickUp",
        id: "keyStickUp",
        default: "W",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "StickLeft",
        id: "keyStickLeft",
        default: "A",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "StickDown",
        id: "keyStickDown",
        default: "S",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keyboard",
        key: "StickRight",
        id: "keyStickRight",
        default: "D",
        kind: FieldKind::Key,
    },
    Field {
        section: "Mouse",
        key: "MouseSensitivity",
        id: "mouseSensitivity",
        default: "1.0",
        kind: FieldKind::Decimal(0.1, 10.0),
    },
    Field {
        section: "Mouse",
        key: "InvertHorizontal",
        id: "cameraInvertHorizontal",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Mouse",
        key: "InvertVertical",
        id: "cameraInvertVertical",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Mouse",
        key: "Smoothing",
        id: "cameraSmoothing",
        default: "0.5",
        kind: FieldKind::Decimal(0.05, 1.0),
    },
    Field {
        section: "Mouse",
        key: "LeftButton",
        id: "mouseLeftButton",
        default: "X",
        kind: FieldKind::Select(MOUSE_BUTTON_OPTIONS),
    },
    Field {
        section: "Mouse",
        key: "MiddleButton",
        id: "mouseMiddleButton",
        default: "None",
        kind: FieldKind::Select(MOUSE_BUTTON_OPTIONS),
    },
    Field {
        section: "Mouse",
        key: "RightButton",
        id: "mouseRightButton",
        default: "A",
        kind: FieldKind::Select(MOUSE_BUTTON_OPTIONS),
    },
    Field {
        section: "Mouse",
        key: "X1Button",
        id: "mouseX1Button",
        default: "None",
        kind: FieldKind::Select(MOUSE_BUTTON_OPTIONS),
    },
    Field {
        section: "Mouse",
        key: "X2Button",
        id: "mouseX2Button",
        default: "None",
        kind: FieldKind::Select(MOUSE_BUTTON_OPTIONS),
    },
    Field {
        section: "Multiplayer",
        key: "Port",
        id: "port",
        default: "39500",
        kind: FieldKind::Integer(1.0, 65535.0),
    },
    Field {
        section: "Multiplayer",
        key: "Join",
        id: "join",
        default: "",
        kind: FieldKind::Text,
    },
    Field {
        section: "Multiplayer",
        key: "SyncSmallMonsters",
        id: "syncSmallMonsters",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Village Quests",
        key: "VillageQuestsInHub",
        id: "villageQuestsInHub",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Village Quests",
        key: "HubHP",
        id: "hubHp",
        default: "2.0",
        kind: FieldKind::Multiplier(0.05, 20.0),
    },
    Field {
        section: "Bonus Content",
        key: "BonusContent",
        id: "bonusContent",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Monster HP",
        key: "1Player",
        id: "hp1",
        default: "1.0",
        kind: FieldKind::Multiplier(0.05, 20.0),
    },
    Field {
        section: "Monster HP",
        key: "2Players",
        id: "hp2",
        default: "1.0",
        kind: FieldKind::Multiplier(0.05, 20.0),
    },
    Field {
        section: "Monster HP",
        key: "3Players",
        id: "hp3",
        default: "1.0",
        kind: FieldKind::Multiplier(0.05, 20.0),
    },
    Field {
        section: "Monster HP",
        key: "4Players",
        id: "hp4",
        default: "1.0",
        kind: FieldKind::Multiplier(0.05, 20.0),
    },
];

const CANONICAL_SECTIONS: &[&str] = &[
    "Display",
    "Anti-aliasing",
    "Upscaler",
    "Graphics",
    "Performance",
    "Controller",
    "Controls",
    "Keyboard",
    "Mouse",
    "Multiplayer",
    "Audio",
    "Monster HP",
    "Village Quests",
    "Bonus Content",
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppState {
    game_exists: bool,
    background: &'static str,
}

#[derive(Serialize)]
struct SaveResult {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation: Option<HashMap<String, String>>,
}

fn app_directory() -> Result<PathBuf, String> {
    std::env::current_exe()
        .map_err(|error| format!("Could not locate the launcher: {error}"))?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Could not locate the launcher's folder.".to_string())
}

fn ini_path(directory: &Path) -> PathBuf {
    directory.join("mhgurecomp.ini")
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && !matches!(character, '_' | '-'))
        .flat_map(char::to_lowercase)
        .collect()
}

fn address(section: &str, key: &str) -> String {
    format!("{}|{}", normalize(section), normalize(key))
}

fn default_settings() -> Value {
    let mut settings = Map::new();
    for field in FIELDS {
        settings.insert(field.id.to_string(), json!(field.default));
    }
    settings.insert("quests".to_string(), json!([]));
    Value::Object(settings)
}

fn split_ini(content: &str) -> Vec<String> {
    content
        .strip_prefix('\u{feff}')
        .unwrap_or(content)
        .lines()
        .map(str::to_string)
        .collect()
}

fn parse_ini(lines: &[String]) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let mut section = String::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            if let Some(close) = trimmed.find(']') {
                section = normalize(&trimmed[1..close]);
            }
            continue;
        }
        let Some(separator) = trimmed.find('=') else {
            continue;
        };
        let key = normalize(trimmed[..separator].trim());
        let mut value = &trimmed[separator + 1..];
        if let Some(comment) = value.find([';', '#']) {
            value = &value[..comment];
        }
        values.insert(format!("{section}|{key}"), value.trim().to_string());
    }
    values
}

fn read_settings(directory: &Path) -> Result<Value, String> {
    let ini = ini_path(directory);
    let content = match fs::read_to_string(&ini) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(default_settings()),
        Err(error) => return Err(format!("Could not read {}: {error}", ini.display())),
    };

    let values = parse_ini(&split_ini(&content));
    let mut settings = default_settings();
    let Some(settings) = settings.as_object_mut() else {
        unreachable!()
    };

    for field in FIELDS {
        let current_address = address(field.section, field.key);
        let legacy_address =
            (field.section == "Controller").then(|| address("Controls", field.key));
        if let Some(value) = values.get(&current_address).or_else(|| {
            legacy_address
                .as_ref()
                .and_then(|legacy_address| values.get(legacy_address))
        }) {
            settings.insert(field.id.to_string(), json!(value));
        }
    }
    migrate_default_keyboard_bindings(settings);

    let mut quests = values
        .iter()
        .filter_map(|(key, value)| {
            let suffix = key.strip_prefix("monsterhp|quest")?;
            let id = suffix.parse::<u32>().ok()?;
            Some((id, value.clone()))
        })
        .collect::<Vec<_>>();
    quests.sort_by_key(|(id, _)| *id);
    settings.insert(
        "quests".to_string(),
        Value::Array(
            quests
                .into_iter()
                .map(|(id, value)| json!({ "id": id, "value": value }))
                .collect(),
        ),
    );
    Ok(Value::Object(settings.clone()))
}

fn migrate_default_keyboard_bindings(settings: &mut Map<String, Value>) {
    let is_default =
        |id: &str, value: &str| settings.get(id).and_then(Value::as_str) == Some(value);
    if is_default("keyButtonB", "Space")
        && is_default("keyButtonBAlt", "Return")
        && is_default("keyButtonA", "F")
        && is_default("keyButtonAAlt", "Backspace")
        && is_default("keyButtonPlus", "Return")
        && is_default("keyButtonPlusAlt", "Keypad Enter")
        && is_default("keyButtonPlusAlt2", "Escape")
    {
        settings.insert("keyButtonBAlt".to_string(), json!("Backspace"));
        settings.insert("keyButtonAAlt".to_string(), json!("Return"));
        settings.insert("keyButtonPlus".to_string(), json!("Keypad Enter"));
        settings.insert("keyButtonPlusAlt".to_string(), json!("Escape"));
        settings.insert("keyButtonPlusAlt2".to_string(), json!("None"));
    }
}

fn string_setting<'a>(settings: &'a Value, id: &str) -> Option<&'a str> {
    settings.get(id).and_then(Value::as_str)
}

fn valid_number(value: &str, min: f64, max: f64) -> bool {
    let value = value.trim();
    let value = value
        .strip_suffix('x')
        .or_else(|| value.strip_suffix('X'))
        .unwrap_or(value);
    value
        .parse::<f64>()
        .map(|number| number.is_finite() && number >= min && number <= max)
        .unwrap_or(false)
}

fn valid_aspect_ratio(value: &str) -> bool {
    if value.eq_ignore_ascii_case("auto") {
        return true;
    }
    let ratio = if let Some((width, height)) = value
        .split_once(':')
        .or_else(|| value.split_once('/'))
    {
        let (Ok(width), Ok(height)) = (
            width.trim().parse::<f64>(),
            height.trim().parse::<f64>(),
        ) else {
            return false;
        };
        if width <= 0.0 || height <= 0.0 {
            return false;
        }
        width / height
    } else {
        let Ok(ratio) = value.parse::<f64>() else {
            return false;
        };
        ratio
    };
    ratio.is_finite() && (0.5..=10.0).contains(&ratio)
}

fn valid_keyboard_key(value: &str) -> bool {
    if value.eq_ignore_ascii_case("none") {
        return true;
    }
    if value.len() == 1 && value.as_bytes()[0].is_ascii_alphanumeric() {
        return true;
    }
    [
        "Up",
        "Down",
        "Left",
        "Right",
        "Escape",
        "Return",
        "Keypad Enter",
        "Space",
        "Tab",
        "Backspace",
        "Left Shift",
        "Right Shift",
        "Left Ctrl",
        "Right Ctrl",
        "Left Alt",
        "Right Alt",
        "CapsLock",
        "F1",
        "F2",
        "F3",
        "F4",
        "F5",
        "F6",
        "F7",
        "F8",
        "F9",
        "F10",
        "F11",
        "F12",
        "Insert",
        "Delete",
        "Home",
        "End",
        "PageUp",
        "PageDown",
        "-",
        "=",
        "[",
        "]",
        "\\",
        ";",
        "'",
        "`",
        ",",
        ".",
        "/",
        "Keypad 0",
        "Keypad 1",
        "Keypad 2",
        "Keypad 3",
        "Keypad 4",
        "Keypad 5",
        "Keypad 6",
        "Keypad 7",
        "Keypad 8",
        "Keypad 9",
        "Keypad +",
        "Keypad -",
        "Keypad *",
        "Keypad /",
        "Keypad .",
    ]
    .iter()
    .any(|key| value.eq_ignore_ascii_case(key))
}

fn validate_settings(settings: &Value) -> HashMap<String, String> {
    let mut errors = HashMap::new();

    for field in FIELDS {
        let Some(value) = string_setting(settings, field.id) else {
            errors.insert(field.id.to_string(), format!("{} must be text.", field.key));
            continue;
        };
        let error = match field.kind {
            FieldKind::Select(options) => {
                (!options.contains(&value)).then(|| format!("Choose a valid {} value.", field.key))
            }
            FieldKind::Toggle => (!(value == "On" || value == "Off"))
                .then(|| format!("{} must be On or Off.", field.key)),
            FieldKind::Integer(min, max) => {
                let is_integer =
                    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit());
                (!(is_integer && valid_number(value, min, max))).then(|| {
                    format!(
                        "{} must be between {} and {}.",
                        field.key, min as u32, max as u32
                    )
                })
            }
            FieldKind::Decimal(min, max) | FieldKind::Multiplier(min, max) => {
                (!valid_number(value, min, max))
                    .then(|| format!("{} must be between {} and {}.", field.key, min, max))
            }
            FieldKind::AspectRatio => (!valid_aspect_ratio(value))
                .then(|| "AspectRatio must be Auto or a ratio from 0.5 to 10.".to_string()),
            FieldKind::Key => (!valid_keyboard_key(value))
                .then(|| format!("{} must be a supported keyboard key.", field.key)),
            FieldKind::Resolution | FieldKind::RenderResolution => {
                let special = match field.kind {
                    FieldKind::Resolution => value == "Auto" || value == "Desktop",
                    FieldKind::RenderResolution => value == "Auto",
                    _ => false,
                };
                let dimensions = value
                    .split_once('x')
                    .or_else(|| value.split_once('X'))
                    .and_then(|(width, height)| {
                        Some((width.parse::<u32>().ok()?, height.parse::<u32>().ok()?))
                    });
                let dimensions_valid = dimensions
                    .map(|(width, height)| {
                        (320..=16384).contains(&width) && (180..=16384).contains(&height)
                    })
                    .unwrap_or(false);
                (!(special || dimensions_valid)).then(|| {
                    format!(
                        "{} must be Auto, Desktop, or a valid WxH resolution.",
                        field.key
                    )
                })
            }
            FieldKind::Text => (value.len() >= 256 || value.contains(['\r', '\n']))
                .then(|| format!("{} must be under 256 bytes and fit on one line.", field.key)),
        };
        if let Some(error) = error {
            errors.insert(field.id.to_string(), error);
        }
    }

    let Some(quests) = settings.get("quests").and_then(Value::as_array) else {
        errors.insert(
            "quests".to_string(),
            "Enter no more than 256 quest entries.".to_string(),
        );
        return errors;
    };
    if quests.len() > 256 {
        errors.insert(
            "quests".to_string(),
            "Enter no more than 256 quest entries.".to_string(),
        );
        return errors;
    }

    let mut ids = HashSet::new();
    for quest in quests {
        let id = quest.get("id").and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_str()?.parse::<u64>().ok())
        });
        let Some(id) = id.filter(|id| (1..=u32::MAX as u64).contains(id)) else {
            errors.insert(
                "quests".to_string(),
                "Quest IDs must be unique integers from 1 to 4294967295.".to_string(),
            );
            break;
        };
        if !ids.insert(id) {
            errors.insert(
                "quests".to_string(),
                "Quest IDs must be unique integers from 1 to 4294967295.".to_string(),
            );
            break;
        }
        let value = quest.get("value").map(value_as_string).unwrap_or_default();
        let factors = value.split(',').collect::<Vec<_>>();
        if (factors.len() != 1 && factors.len() != 4)
            || factors
                .iter()
                .any(|factor| !valid_number(factor, 0.05, 20.0))
        {
            errors.insert(
                "quests".to_string(),
                "Quest multipliers must contain one or four values from 0.05 to 20.".to_string(),
            );
            break;
        }
    }
    errors
}

fn value_as_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        _ => String::new(),
    }
}

fn replace_value(line: &str, value: &str) -> String {
    let Some(separator) = line.find('=') else {
        return line.to_string();
    };
    let bytes = line.as_bytes();
    let mut value_start = separator + 1;
    while value_start < bytes.len() && matches!(bytes[value_start], b' ' | b'\t') {
        value_start += 1;
    }
    let mut value_end = line[value_start..]
        .find([';', '#'])
        .map(|index| value_start + index)
        .unwrap_or(line.len());
    while value_end > value_start && matches!(bytes[value_end - 1], b' ' | b'\t') {
        value_end -= 1;
    }
    format!("{}{}{}", &line[..value_start], value, &line[value_end..])
}

fn append_missing_entries(
    output: &mut Vec<String>,
    missing: &mut BTreeMap<String, Vec<(String, String)>>,
    section: &str,
) {
    let Some(entries) = missing.remove(section) else {
        return;
    };
    for (key, value) in entries {
        let ini_key = FIELDS
            .iter()
            .find(|field| normalize(field.section) == section && normalize(field.key) == key)
            .map(|field| field.key.to_string())
            .unwrap_or_else(|| format!("Quest{}", key.trim_start_matches("quest")));
        output.push(format!("{ini_key} = {value}"));
    }
}

fn save_settings(directory: &Path, settings: &Value) -> Result<(), String> {
    let errors = validate_settings(settings);
    if !errors.is_empty() {
        return Err("Some settings are invalid.".to_string());
    }

    let ini = ini_path(directory);
    let original_content = match fs::read_to_string(&ini) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("Could not read {}: {error}", ini.display())),
    };
    let original_lines = split_ini(&original_content);
    let existing = parse_ini(&original_lines);
    let current_settings = read_settings(directory)?;
    let mut updates = Vec::new();
    for field in FIELDS {
        let value = string_setting(settings, field.id).unwrap_or_default();
        if current_settings.get(field.id).and_then(Value::as_str) != Some(value) {
            let current_address = address(field.section, field.key);
            let legacy_address =
                (field.section == "Controller").then(|| address("Controls", field.key));
            let update_address = match legacy_address {
                Some(legacy_address)
                    if !existing.contains_key(&current_address)
                        && existing.contains_key(&legacy_address) =>
                {
                    legacy_address
                }
                _ => current_address,
            };
            updates.push((update_address, value.to_string()));
        }
    }
    let quest_values = |settings: &Value| {
        settings
            .get("quests")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|quest| {
                let id = quest.get("id")?.as_u64()?;
                let value = quest.get("value").map(value_as_string)?;
                Some((id, value))
            })
            .collect::<BTreeMap<_, _>>()
    };
    let update_quests = quest_values(settings) != quest_values(&current_settings);
    if update_quests {
        if let Some(quests) = settings.get("quests").and_then(Value::as_array) {
            for quest in quests {
                let id = quest.get("id").map(value_as_string).unwrap_or_default();
                let value = quest.get("value").map(value_as_string).unwrap_or_default();
                updates.push((address("Monster HP", &format!("Quest{id}")), value));
            }
        }
    }
    if updates.is_empty() && !update_quests {
        return Ok(());
    }

    let mut missing = BTreeMap::<String, Vec<(String, String)>>::new();
    for (current_address, value) in &updates {
        if existing.contains_key(current_address) {
            continue;
        }
        if let Some((section, key)) = current_address.split_once('|') {
            missing
                .entry(section.to_string())
                .or_default()
                .push((key.to_string(), value.clone()));
        }
    }

    let mut output = Vec::new();
    let mut section = String::new();
    for line in &original_lines {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if let Some(close) = trimmed.find(']') {
                append_missing_entries(&mut output, &mut missing, &section);
                section = normalize(&trimmed[1..close]);
            }
        }
        if !trimmed.starts_with(';') && !trimmed.starts_with('#') {
            if let Some(separator) = trimmed.find('=') {
                let key = normalize(trimmed[..separator].trim());
                let current_address = format!("{section}|{key}");
                let is_quest = section == "monsterhp" && key.starts_with("quest");
                if update_quests
                    && is_quest
                    && !updates.iter().any(|(key, _)| key == &current_address)
                {
                    continue;
                }
                if let Some((_, value)) = updates.iter().find(|(key, _)| key == &current_address) {
                    output.push(replace_value(line, value));
                    continue;
                }
            }
        }
        output.push(line.clone());
    }

    append_missing_entries(&mut output, &mut missing, &section);
    for name in CANONICAL_SECTIONS {
        let normalized = normalize(name);
        if !missing.contains_key(&normalized) {
            continue;
        }
        if !output.is_empty() && output.last().is_some_and(|line| !line.is_empty()) {
            output.push(String::new());
        }
        output.push(format!("[{name}]"));
        append_missing_entries(&mut output, &mut missing, &normalized);
    }
    if !missing.is_empty() {
        return Err(format!(
            "Cannot write unknown INI sections: {}",
            missing.keys().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    let line_ending = if original_content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let content = output.join(line_ending);
    let content = if original_content.is_empty()
        || original_content.ends_with('\n')
        || original_content.ends_with('\r')
    {
        format!("{content}{line_ending}")
    } else {
        content
    };
    fs::write(&ini, content).map_err(|error| format!("Could not write {}: {error}", ini.display()))
}

#[tauri::command]
fn get_app_state() -> Result<AppState, String> {
    let directory = app_directory()?;
    let mut rng = rand::thread_rng();
    let background = ["1.webp", "2.webp", "3.webp", "4.webp"][rng.gen_range(0..4)];
    Ok(AppState {
        game_exists: directory.join("mhgurecomp.exe").is_file(),
        background,
    })
}

#[tauri::command]
fn get_settings() -> Result<Value, String> {
    read_settings(&app_directory()?)
}

#[tauri::command]
fn get_defaults() -> Value {
    default_settings()
}

#[tauri::command]
fn save_launcher_settings(settings: Value) -> Result<SaveResult, String> {
    match save_settings(&app_directory()?, &settings) {
        Ok(()) => Ok(SaveResult {
            ok: true,
            validation: None,
        }),
        Err(error) if error == "Some settings are invalid." => Ok(SaveResult {
            ok: false,
            validation: Some(validate_settings(&settings)),
        }),
        Err(error) => Err(error),
    }
}

fn game_command(game: &Path, directory: &Path) -> Command {
    let mut command = Command::new(game);
    command
        .current_dir(directory)
        .env_remove("MHGU_HOME")
        .env_remove("MHGU_CONFIG");
    command
}

#[tauri::command]
fn launch_game() -> Result<(), String> {
    let directory = app_directory()?;
    let game = directory.join("mhgurecomp.exe");
    if !game.is_file() {
        return Err("mhgurecomp.exe was not found next to the launcher.".to_string());
    }
    game_command(&game, &directory)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not launch mhgurecomp.exe: {error}"))
}

#[tauri::command]
fn open_ini() -> Result<(), String> {
    let directory = app_directory()?;
    let ini = ini_path(&directory);
    if !ini.exists() {
        save_settings(&directory, &default_settings())?;
    }
    open::that(&ini).map_err(|error| format!("Could not open {}: {error}", ini.display()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            get_settings,
            get_defaults,
            save_launcher_settings,
            launch_game,
            open_ini
        ])
        .run(tauri::generate_context!())
        .expect("error while running MHGU Launcher");
}

fn main() {
    run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_directory() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("MHGU Launcher spécial 路径 {id}"));
        fs::create_dir_all(&path).expect("create temp directory");
        path
    }

    #[test]
    fn saving_only_changes_preserves_ini_structure_and_unmodified_settings() {
        let directory = temp_directory();
        let original = "; keep this note\n[Display]\nMode = Windowed ; keep this inline note\nCustom = untouched\n[Other]\nValue = yes\n[Monster HP]\nQuest8 = 1.2\nQuest9 = 2.0\n";
        fs::write(ini_path(&directory), &original).expect("write source ini");
        let mut settings = read_settings(&directory).expect("load ini");
        assert_eq!(settings["mode"], "Windowed");
        assert_eq!(settings["quests"][0]["id"], 8);
        settings["mode"] = json!("Fullscreen");
        settings["resolution"] = json!("2560x1440");
        save_settings(&directory, &settings).expect("update ini");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("; keep this note"));
        assert!(output.contains("Mode = Fullscreen ; keep this inline note"));
        assert!(output.contains("Custom = untouched"));
        assert!(output.contains("Custom = untouched\nResolution = 2560x1440\n[Other]"));
        assert!(output.contains("[Other]\nValue = yes\n[Monster HP]\nQuest8 = 1.2\nQuest9 = 2.0\n"));
        assert!(!output.contains("RenderResolution ="));
        assert!(!output.contains("FPS ="));
        assert!(!output.contains("VillageQuestsInHub ="));
        assert!(output.contains("Quest8 = 1.2"));
        assert!(output.contains("Quest9 = 2.0"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn unchanged_defaults_do_not_create_or_rewrite_ini() {
        let directory = temp_directory();
        let settings = read_settings(&directory).expect("load missing ini");
        save_settings(&directory, &settings).expect("save unchanged defaults");
        assert!(!ini_path(&directory).exists());

        let original = "[Display]\nMode = Windowed\n";
        fs::write(ini_path(&directory), original).expect("write source ini");
        let settings = read_settings(&directory).expect("load ini");
        save_settings(&directory, &settings).expect("save unchanged settings");
        assert_eq!(
            fs::read_to_string(ini_path(&directory)).expect("read ini"),
            original
        );
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn changed_quest_settings_update_and_remove_only_requested_quests() {
        let directory = temp_directory();
        fs::write(
            ini_path(&directory),
            "[Monster HP]\nQuest8 = 1.2\nQuest9 = 2.0\n",
        )
        .expect("write source ini");
        let mut settings = read_settings(&directory).expect("load ini");
        settings["quests"] = json!([{ "id": 8, "value": "1.5" }]);
        save_settings(&directory, &settings).expect("update quests");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("Quest8 = 1.5"));
        assert!(!output.contains("Quest9"));
        assert!(!output.contains("Mode ="));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn launcher_started_game_uses_ini_beside_its_executable() {
        let directory = temp_directory();
        let game = directory.join("mhgurecomp.exe");
        let command = game_command(&game, &directory);

        assert_eq!(command.get_program(), game.as_os_str());
        assert_eq!(command.get_current_dir(), Some(directory.as_path()));
        for name in ["MHGU_HOME", "MHGU_CONFIG"] {
            assert_eq!(
                command.get_envs().find(|(key, _)| *key == name),
                Some((std::ffi::OsStr::new(name), None))
            );
        }

        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn validates_standard_values_and_per_quest_multipliers() {
        let mut settings = default_settings();
        settings["fps"] = json!("241");
        settings["quests"] = json!([{ "id": 0, "value": "99" }]);
        let errors = validate_settings(&settings);
        assert!(errors.contains_key("fps"));
        assert!(errors.contains_key("quests"));

        settings["fps"] = json!("60");
        settings["mouseSensitivity"] = json!("1.0xx");
        assert!(validate_settings(&settings).contains_key("mouseSensitivity"));
        settings["mouseSensitivity"] = json!("1.0");
        settings["quests"] = json!([
            { "id": 123, "value": "1.0" },
            { "id": 456, "value": "1, 1.2, 1.5, 2.0" }
        ]);
        assert!(validate_settings(&settings).is_empty());
    }

    #[test]
    fn fsr_options_are_validated_and_saved() {
        let directory = temp_directory();
        let mut settings = default_settings();
        settings["fsr"] = json!("On");
        settings["fsrUpscaler"] = json!("FSR4");
        settings["fsrQuality"] = json!("Balanced");
        settings["fsrSharpness"] = json!("0.5");
        settings["fsrObjectMotion"] = json!("Off");
        settings["fsrReactive"] = json!("1.0");
        settings["fsrAnimSurf"] = json!("Off");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save FSR settings");
        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("[Upscaler]"));
        assert!(output.contains("FSR = On"));
        assert!(output.contains("Upscaler = FSR4"));
        assert!(output.contains("FSRQuality = Balanced"));
        assert!(output.contains("Sharpness = 0.5"));
        assert!(output.contains("ObjectMotion = Off"));
        assert!(output.contains("Reactive = 1.0"));
        assert!(output.contains("AnimSurf = Off"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["fsr"], "On");
        assert_eq!(loaded["fsrUpscaler"], "FSR4");
        assert_eq!(loaded["fsrQuality"], "Balanced");
        assert_eq!(loaded["fsrSharpness"], "0.5");
        assert_eq!(loaded["fsrObjectMotion"], "Off");
        assert_eq!(loaded["fsrReactive"], "1.0");
        assert_eq!(loaded["fsrAnimSurf"], "Off");

        settings["fsrSharpness"] = json!("1.1");
        assert!(validate_settings(&settings).contains_key("fsrSharpness"));
        settings["fsrSharpness"] = json!("0.5");
        settings["fsrUpscaler"] = json!("FSR5");
        assert!(validate_settings(&settings).contains_key("fsrUpscaler"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn shader_prewarm_and_small_monster_defaults_match_runtime() {
        let directory = temp_directory();
        let mut settings = default_settings();
        assert_eq!(settings["shaderPrewarm"], "On");
        assert_eq!(settings["syncSmallMonsters"], "Off");
        settings["shaderPrewarm"] = json!("Off");
        settings["syncSmallMonsters"] = json!("On");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save performance and multiplayer settings");
        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("ShaderPrewarm = Off"));
        assert!(output.contains("SyncSmallMonsters = On"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["shaderPrewarm"], "Off");
        assert_eq!(loaded["syncSmallMonsters"], "On");
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn upstream_graphics_controls_and_audio_settings_round_trip() {
        let directory = temp_directory();
        let mut settings = default_settings();
        assert_eq!(settings["drawDistance"], "1.0");
        assert_eq!(settings["comboInputWindow"], "Lenient");
        assert_eq!(settings["muteInBackground"], "Off");

        settings["drawDistance"] = json!("2.5");
        settings["comboInputWindow"] = json!("Strict");
        settings["muteInBackground"] = json!("On");
        assert!(validate_settings(&settings).is_empty());
        save_settings(&directory, &settings).expect("save upstream settings");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("[Graphics]\nDrawDistance = 2.5"));
        assert!(output.contains("[Controls]\nComboInputWindow = Strict"));
        assert!(output.contains("[Audio]\nMuteInBackground = On"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["drawDistance"], "2.5");
        assert_eq!(loaded["comboInputWindow"], "Strict");
        assert_eq!(loaded["muteInBackground"], "On");

        settings["drawDistance"] = json!("10.1");
        assert!(validate_settings(&settings).contains_key("drawDistance"));
        settings["drawDistance"] = json!("2.5");
        settings["comboInputWindow"] = json!("Loose");
        assert!(validate_settings(&settings).contains_key("comboInputWindow"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn display_and_background_options_round_trip() {
        let directory = temp_directory();
        let mut settings = default_settings();
        settings["aspectRatio"] = json!("21:9");
        settings["fov"] = json!("1.2");
        settings["uiAnchor"] = json!("Edges");
        settings["backgroundInput"] = json!("Off");
        settings["rumble"] = json!("Off");
        assert!(validate_settings(&settings).is_empty());
        save_settings(&directory, &settings).expect("save display settings");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("AspectRatio = 21:9"));
        assert!(output.contains("FOV = 1.2"));
        assert!(output.contains("UIAnchor = Edges"));
        assert!(output.contains("[Controller]\nBackgroundInput = Off\nRumble = Off"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["aspectRatio"], "21:9");
        assert_eq!(loaded["fov"], "1.2");
        assert_eq!(loaded["uiAnchor"], "Edges");
        assert_eq!(loaded["backgroundInput"], "Off");
        assert_eq!(loaded["rumble"], "Off");

        settings["aspectRatio"] = json!("21:0");
        assert!(validate_settings(&settings).contains_key("aspectRatio"));
        settings["aspectRatio"] = json!("0");
        assert!(validate_settings(&settings).contains_key("aspectRatio"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn camera_settings_round_trip() {
        let directory = temp_directory();
        let mut settings = default_settings();
        settings["mouseSensitivity"] = json!("1.5");
        settings["cameraSmoothing"] = json!("0.7");
        settings["cameraInvertHorizontal"] = json!("On");
        settings["cameraInvertVertical"] = json!("On");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save camera settings");
        let output = fs::read_to_string(ini_path(&directory)).expect("read camera settings");
        assert!(output.contains("MouseSensitivity = 1.5"));
        assert!(output.contains("[Mouse]\nMouseSensitivity = 1.5\nInvertHorizontal = On"));
        assert!(output.contains("InvertVertical = On"));
        assert!(output.contains("Smoothing = 0.7"));

        let loaded = read_settings(&directory).expect("reload camera settings");
        assert_eq!(loaded["mouseSensitivity"], "1.5");
        assert_eq!(loaded["cameraSmoothing"], "0.7");
        assert_eq!(loaded["cameraInvertHorizontal"], "On");
        assert_eq!(loaded["cameraInvertVertical"], "On");
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn controller_settings_read_legacy_controls_section() {
        let directory = temp_directory();
        let original = "[Controls]\nFaceButtons = Label\nBackgroundInput = Off\n";
        fs::write(ini_path(&directory), original).expect("write legacy controller settings");

        let mut settings = read_settings(&directory).expect("load legacy settings");
        assert_eq!(settings["faceButtons"], "Label");
        assert_eq!(settings["backgroundInput"], "Off");
        assert_eq!(settings["rumble"], "On");

        settings["backgroundInput"] = json!("On");
        settings["rumble"] = json!("Off");
        save_settings(&directory, &settings).expect("save updated controller settings");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("[Controls]\nFaceButtons = Label\nBackgroundInput = On"));
        assert!(output.contains("[Controller]\nRumble = Off"));
        let loaded = read_settings(&directory).expect("reload controller settings");
        assert_eq!(loaded["faceButtons"], "Label");
        assert_eq!(loaded["backgroundInput"], "On");
        assert_eq!(loaded["rumble"], "Off");
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn keyboard_bindings_are_validated_and_saved_in_keyboard_section() {
        let directory = temp_directory();
        let mut settings = default_settings();
        settings["keyButtonX"] = json!("H");
        settings["keyButtonXAlt"] = json!("J");
        settings["keyButtonYAlt"] = json!("K");
        settings["keyButtonLAlt"] = json!("M");
        settings["keyButtonRAlt"] = json!("N");
        settings["keyButtonZlAlt"] = json!("O");
        settings["keyButtonZrAlt"] = json!("P");
        settings["keyButtonLeftStickAlt"] = json!("Home");
        settings["keyButtonRightStickAlt"] = json!("End");
        settings["keyDpadUp"] = json!("I");
        settings["keyDpadUpAlt"] = json!("Up");
        assert!(validate_settings(&settings).is_empty());
        save_settings(&directory, &settings).expect("save keyboard bindings");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("ButtonX = H"));
        assert!(output.contains("ButtonXAlt = J"));
        assert!(output.contains("ButtonYAlt = K"));
        assert!(output.contains("ButtonLAlt = M"));
        assert!(output.contains("ButtonRAlt = N"));
        assert!(output.contains("ButtonZLAlt = O"));
        assert!(output.contains("ButtonZRAlt = P"));
        assert!(output.contains("ButtonLeftStickAlt = Home"));
        assert!(output.contains("ButtonRightStickAlt = End"));
        assert!(output.contains("[Keyboard]\nDpadUp = I\n"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["keyButtonX"], "H");
        assert_eq!(loaded["keyButtonXAlt"], "J");
        assert_eq!(loaded["keyButtonYAlt"], "K");
        assert_eq!(loaded["keyButtonLAlt"], "M");
        assert_eq!(loaded["keyButtonRAlt"], "N");
        assert_eq!(loaded["keyButtonZlAlt"], "O");
        assert_eq!(loaded["keyButtonZrAlt"], "P");
        assert_eq!(loaded["keyButtonLeftStickAlt"], "Home");
        assert_eq!(loaded["keyButtonRightStickAlt"], "End");
        assert_eq!(
            loaded["keyDpadUp"],
            "I"
        );
        assert_eq!(
            loaded["keyDpadUpAlt"],
            "Up"
        );

        settings["keyDpadUp"] = json!("not-a-key");
        assert!(validate_settings(&settings).contains_key("keyDpadUp"));
        settings["keyDpadUp"] = json!("None");
        assert!(validate_settings(&settings).is_empty());
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn mouse_button_assignments_are_validated_and_saved_in_mouse_section() {
        let directory = temp_directory();
        let mut settings = default_settings();
        assert_eq!(settings["mouseLeftButton"], "X");
        assert_eq!(settings["mouseRightButton"], "A");
        settings["mouseLeftButton"] = json!("B");
        settings["mouseMiddleButton"] = json!("D-pad Up");
        settings["mouseRightButton"] = json!("None");
        assert!(validate_settings(&settings).is_empty());
        save_settings(&directory, &settings).expect("save mouse button assignments");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("[Mouse]\nLeftButton = B\n"));
        assert!(output.contains("MiddleButton = D-pad Up\n"));
        assert!(output.contains("RightButton = None\n"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["mouseLeftButton"], "B");
        assert_eq!(loaded["mouseMiddleButton"], "D-pad Up");
        assert_eq!(loaded["mouseRightButton"], "None");

        settings["mouseLeftButton"] = json!("invalid");
        assert!(validate_settings(&settings).contains_key("mouseLeftButton"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn old_default_keyboard_mapping_migrates_without_rewriting_custom_mappings() {
        let directory = temp_directory();
        fs::write(
            ini_path(&directory),
            "[Keyboard]\nButtonB = Space\nButtonBAlt = Return\nButtonA = F\nButtonAAlt = Backspace\nButtonPlus = Return\nButtonPlusAlt = Keypad Enter\nButtonPlusAlt2 = Escape\n",
        )
        .expect("write old default bindings");
        let settings = read_settings(&directory).expect("load old defaults");
        assert_eq!(settings["keyButtonBAlt"], "Backspace");
        assert_eq!(settings["keyButtonAAlt"], "Return");
        assert_eq!(settings["keyButtonPlus"], "Keypad Enter");
        assert_eq!(settings["keyButtonPlusAlt"], "Escape");
        assert_eq!(settings["keyButtonPlusAlt2"], "None");

        fs::write(
            ini_path(&directory),
            "[Keyboard]\nButtonB = Space\nButtonBAlt = Return\nButtonA = F\nButtonAAlt = Backspace\nButtonPlus = Return\nButtonPlusAlt = Keypad Enter\nButtonPlusAlt2 = Escape\nButtonY = U\n",
        )
        .expect("write customized bindings");
        let settings = read_settings(&directory).expect("load customized bindings");
        assert_eq!(settings["keyButtonPlus"], "Keypad Enter");
        assert_eq!(settings["keyButtonBAlt"], "Backspace");
        assert_eq!(settings["keyButtonAAlt"], "Return");
        assert_eq!(settings["keyButtonY"], "U");
        fs::remove_dir_all(directory).expect("clean temp directory");
    }
}
