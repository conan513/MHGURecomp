#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rand::Rng;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::io::{Cursor, Read};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipArchive;

#[cfg(windows)]
const GAME_EXECUTABLE: &str = "mhgurecomp.exe";
#[cfg(not(windows))]
const GAME_EXECUTABLE: &str = "mhgurecomp";

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
    BoundedText(usize),
    Key,
    ChatKey,
    MatchmakingKey,
    Resolution,
    RenderResolution,
    HubServer,
    Text,
}

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
        key: "FXAAMode",
        id: "fxaaMode",
        default: "Original",
        kind: FieldKind::Select(&["Original", "3_11"]),
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
        section: "Graphics",
        key: "SmoothGradients",
        id: "smoothGradients",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "HDR",
        key: "HDR",
        id: "hdr",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "HDR",
        key: "PeakBrightness",
        id: "hdrPeak",
        default: "1000",
        kind: FieldKind::Decimal(80.0, 10000.0),
    },
    Field {
        section: "HDR",
        key: "GameBrightness",
        id: "hdrGame",
        default: "203",
        kind: FieldKind::Decimal(48.0, 500.0),
    },
    Field {
        section: "HDR",
        key: "UIBrightness",
        id: "hdrUi",
        default: "203",
        kind: FieldKind::Decimal(48.0, 500.0),
    },
    Field {
        section: "HDR",
        key: "ToneMapper",
        id: "hdrToneMapper",
        default: "RenoDRT",
        kind: FieldKind::Select(&["Vanilla", "None", "RenoDRT"]),
    },
    Field {
        section: "HDR",
        key: "GammaCorrection",
        id: "hdrGamma",
        default: "2.2",
        kind: FieldKind::Select(&["Off", "2.2", "BT.1886"]),
    },
    Field {
        section: "HDR",
        key: "Exposure",
        id: "hdrExposure",
        default: "0.65",
        kind: FieldKind::Decimal(0.0, 2.0),
    },
    Field {
        section: "HDR",
        key: "Highlights",
        id: "hdrHighlights",
        default: "60",
        kind: FieldKind::Decimal(0.0, 100.0),
    },
    Field {
        section: "HDR",
        key: "Shadows",
        id: "hdrShadows",
        default: "50",
        kind: FieldKind::Decimal(0.0, 100.0),
    },
    Field {
        section: "HDR",
        key: "Contrast",
        id: "hdrContrast",
        default: "50",
        kind: FieldKind::Decimal(0.0, 100.0),
    },
    Field {
        section: "HDR",
        key: "Saturation",
        id: "hdrSaturation",
        default: "50",
        kind: FieldKind::Decimal(0.0, 100.0),
    },
    Field {
        section: "HDR",
        key: "HighlightSaturation",
        id: "hdrHighlightSaturation",
        default: "82",
        kind: FieldKind::Decimal(0.0, 100.0),
    },
    Field {
        section: "HDR",
        key: "Flare",
        id: "hdrFlare",
        default: "0",
        kind: FieldKind::Decimal(0.0, 100.0),
    },
    Field {
        section: "HDR",
        key: "Bloom",
        id: "hdrBloom",
        default: "100",
        kind: FieldKind::Decimal(0.0, 100.0),
    },
    Field {
        section: "HDR",
        key: "UnclampedScene",
        id: "hdrScene",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "HDR",
        key: "Analysis",
        id: "hdrAnalysis",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Performance",
        key: "ShowFPS",
        id: "showFps",
        default: "Off",
        kind: FieldKind::Toggle,
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
        section: "Controls",
        key: "FaceButtons",
        id: "faceButtons",
        default: "Position",
        kind: FieldKind::Select(&["Position", "Label"]),
    },
    Field {
        section: "Controls",
        key: "MenuConfirm",
        id: "menuConfirm",
        default: "Original",
        kind: FieldKind::Select(&["Original", "South", "East"]),
    },
    Field {
        section: "Application",
        key: "ButtonIcons",
        id: "buttonIcons",
        default: "Xbox",
        kind: FieldKind::Select(&["Xbox", "Switch", "PlayStation"]),
    },
    Field {
        section: "Controls",
        key: "BackgroundInput",
        id: "backgroundInput",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Controls",
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
        section: "Controls",
        key: "ChatKey",
        id: "chatKey",
        default: "/",
        kind: FieldKind::ChatKey,
    },
    Field {
        section: "Audio",
        key: "Volume",
        id: "volume",
        default: "100",
        kind: FieldKind::Integer(0.0, 100.0),
    },
    Field {
        section: "Audio",
        key: "MuteInBackground",
        id: "muteInBackground",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Discord",
        key: "RichPresence",
        id: "discordRichPresence",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Discord",
        key: "ShowHunter",
        id: "discordShowHunter",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Keybinds",
        key: "MoveUp",
        id: "bindMoveUp",
        default: "W",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "MoveDown",
        id: "bindMoveDown",
        default: "S",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "MoveLeft",
        id: "bindMoveLeft",
        default: "A",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "MoveRight",
        id: "bindMoveRight",
        default: "D",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "Walk",
        id: "bindWalk",
        default: "CapsLock",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "A",
        id: "bindA",
        default: "F, Return, Mouse Right",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "B",
        id: "bindB",
        default: "Space, Backspace",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "X",
        id: "bindX",
        default: "G, H, Mouse Left",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "Y",
        id: "bindY",
        default: "E, T",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "L",
        id: "bindL",
        default: "Q, U",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "R",
        id: "bindR",
        default: "Left Shift",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "ZL",
        id: "bindZl",
        default: "Tab, Z",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "ZR",
        id: "bindZr",
        default: "R, B",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "Plus",
        id: "bindPlus",
        default: "Keypad Enter",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "Minus",
        id: "bindMinus",
        default: "C, V",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "DpadUp",
        id: "bindDpadUp",
        default: "1, Up",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "DpadDown",
        id: "bindDpadDown",
        default: "3, Down",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "DpadLeft",
        id: "bindDpadLeft",
        default: "4, Left",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "DpadRight",
        id: "bindDpadRight",
        default: "2, Right",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "LeftStickClick",
        id: "bindLeftStickClick",
        default: "Left Ctrl",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "RightStickClick",
        id: "bindRightStickClick",
        default: "Left Alt",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "ItemLeft",
        id: "bindItemLeft",
        default: "Wheel Up",
        kind: FieldKind::Key,
    },
    Field {
        section: "Keybinds",
        key: "ItemRight",
        id: "bindItemRight",
        default: "Wheel Down",
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
        section: "Multiplayer",
        key: "Server",
        id: "hubServer",
        default: "",
        kind: FieldKind::HubServer,
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
        key: "Key",
        id: "matchmakingKey",
        default: "",
        kind: FieldKind::MatchmakingKey,
    },
    Field {
        section: "Multiplayer",
        key: "SyncSmallMonsters",
        id: "syncSmallMonsters",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Multiplayer",
        key: "LagCompensation",
        id: "lagCompensation",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Multiplayer",
        key: "ShareUrgentCredit",
        id: "shareUrgentCredit",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Multiplayer",
        key: "RyuServers",
        id: "ryuServers",
        default: "",
        kind: FieldKind::Text,
    },
    Field {
        section: "Multiplayer",
        key: "RyuHost",
        id: "ryuHost",
        default: "",
        kind: FieldKind::BoundedText(127),
    },
    Field {
        section: "Multiplayer",
        key: "RyuPassphrase",
        id: "ryuPassphrase",
        default: "",
        kind: FieldKind::BoundedText(39),
    },
    Field {
        section: "Multiplayer",
        key: "RyuServe",
        id: "ryuServe",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Multiplayer",
        key: "RyuServePort",
        id: "ryuServePort",
        default: "30456",
        kind: FieldKind::Integer(0.0, 65535.0),
    },
    Field {
        section: "Multiplayer",
        key: "RoomServer",
        id: "roomServer",
        default: "",
        kind: FieldKind::BoundedText(127),
    },
    Field {
        section: "Multiplayer",
        key: "RoomPassword",
        id: "roomPassword",
        default: "",
        kind: FieldKind::BoundedText(63),
    },
    Field {
        section: "Multiplayer",
        key: "RoomName",
        id: "roomName",
        default: "",
        kind: FieldKind::BoundedText(31),
    },
    Field {
        section: "Multiplayer",
        key: "RoomHost",
        id: "roomHost",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Multiplayer",
        key: "RoomServe",
        id: "roomServe",
        default: "Off",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Multiplayer",
        key: "RoomServePort",
        id: "roomServePort",
        default: "24872",
        kind: FieldKind::Integer(0.0, 65535.0),
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
        section: "Application",
        key: "ConfirmExit",
        id: "confirmExit",
        default: "On",
        kind: FieldKind::Toggle,
    },
    Field {
        section: "Application",
        key: "Language",
        id: "language",
        default: "English",
        kind: FieldKind::Select(&["English", "Spanish"]),
    },
    Field {
        section: "Application",
        key: "Theme",
        id: "menuTheme",
        default: "Hunter",
        kind: FieldKind::Select(&["Gold", "Blue", "Hunter"]),
    },
    Field {
        section: "Application",
        key: "MenuScale",
        id: "menuScale",
        default: "100",
        kind: FieldKind::Integer(50.0, 200.0),
    },
    Field {
        section: "Application",
        key: "Updates",
        id: "updates",
        default: "Ask",
        kind: FieldKind::Select(&["Ask", "Auto", "Off"]),
    },
    Field {
        section: "Application",
        key: "SaveLocation",
        id: "saveLocation",
        default: "",
        kind: FieldKind::Text,
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
const MAX_BIND_KEYS: usize = 3;
const PREVIOUS_MAX_BIND_KEYS: usize = 4;

const CANONICAL_SECTIONS: &[&str] = &[
    "Display",
    "Anti-aliasing",
    "Upscaler",
    "Graphics",
    "HDR",
    "Performance",
    "Controls",
    "Mouse",
    "Keybinds",
    "Multiplayer",
    "Audio",
    "Monster HP",
    "Village Quests",
    "Discord",
    "Bonus Content",
    "Application",
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

fn legacy_keyboard_keys(key: &str) -> &'static [&'static str] {
    match normalize(key).as_str() {
        "moveup" => &["StickUp"],
        "movedown" => &["StickDown"],
        "moveleft" => &["StickLeft"],
        "moveright" => &["StickRight"],
        "a" => &["ButtonA", "ButtonAAlt"],
        "b" => &["ButtonB", "ButtonBAlt"],
        "x" => &["ButtonX", "ButtonXAlt"],
        "y" => &["ButtonY", "ButtonYAlt"],
        "l" => &["ButtonL", "ButtonLAlt"],
        "r" => &["ButtonR", "ButtonRAlt"],
        "zl" => &["ButtonZL", "ButtonZLAlt"],
        "zr" => &["ButtonZR", "ButtonZRAlt"],
        "plus" => &["ButtonPlus", "ButtonPlusAlt", "ButtonPlusAlt2"],
        "minus" => &["ButtonMinus", "ButtonMinusAlt"],
        "dpadup" => &["DpadUp", "DpadUpAlt"],
        "dpaddown" => &["DpadDown", "DpadDownAlt"],
        "dpadleft" => &["DpadLeft", "DpadLeftAlt"],
        "dpadright" => &["DpadRight", "DpadRightAlt"],
        "leftstickclick" => &["ButtonLeftStick", "ButtonLeftStickAlt"],
        "rightstickclick" => &["ButtonRightStick", "ButtonRightStickAlt"],
        _ => &[],
    }
}

fn legacy_mouse_key(key: &str) -> Option<&'static str> {
    let mouse_key = match normalize(key).as_str() {
        "leftbutton" => "Mouse Left",
        "rightbutton" => "Mouse Right",
        "middlebutton" => "Mouse Middle",
        "x1button" => "Mouse 4",
        "x2button" => "Mouse 5",
        _ => return None,
    };
    Some(mouse_key)
}

fn legacy_mouse_default_target(key: &str) -> Option<&'static str> {
    match normalize(key).as_str() {
        "leftbutton" => Some("X"),
        "rightbutton" => Some("A"),
        _ => None,
    }
}

fn migrate_legacy_binding(values: &HashMap<String, String>, key: &str) -> Option<String> {
    let mut keys = Vec::new();
    let mut found = false;
    for legacy_key in legacy_keyboard_keys(key) {
        if let Some(value) = values.get(&address("Keyboard", legacy_key)) {
            found = true;
            if !value.eq_ignore_ascii_case("None")
                && valid_key_binding(value)
                && !keys.iter().any(|key| key == value)
            {
                keys.push(value.clone());
            }
        }
    }
    for mouse_key in ["LeftButton", "RightButton", "MiddleButton", "X1Button", "X2Button"] {
        let Some(mouse_button) = legacy_mouse_key(mouse_key) else {
            continue;
        };
        let Some(value) = values.get(&address("Mouse", mouse_key)) else {
            continue;
        };
        let matches_binding = normalize(value) == normalize(key);
        let replaces_old_default = legacy_mouse_default_target(mouse_key)
            .is_some_and(|target| normalize(target) == normalize(key));
        if !matches_binding && !replaces_old_default {
            continue;
        }
        found = true;
        if matches_binding && !keys.iter().any(|binding| normalize(binding) == normalize(mouse_button)) {
            keys.push(mouse_button.to_string());
        }
    }
    if !found {
        return None;
    }
    keys.truncate(MAX_BIND_KEYS);
    Some(if keys.is_empty() {
        "None".to_string()
    } else {
        keys.join(", ")
    })
}

fn truncate_previous_key_binding(value: &str) -> Option<String> {
    let keys = value.split(',').map(str::trim).collect::<Vec<_>>();
    if keys.len() <= MAX_BIND_KEYS || !valid_key_binding_with_limit(value, PREVIOUS_MAX_BIND_KEYS) {
        return None;
    }
    let mut retained = keys.iter().take(2).copied().collect::<Vec<_>>();
    if let Some(mouse) = keys
        .iter()
        .skip(2)
        .find(|key| is_mouse_binding(key))
    {
        retained.push(*mouse);
    } else {
        retained.push(keys[2]);
    }
    Some(retained.join(", "))
}

fn is_mouse_binding(value: &str) -> bool {
    ["Mouse Left", "Mouse Right", "Mouse Middle", "Mouse 4", "Mouse 5"]
        .iter()
        .any(|key| value.eq_ignore_ascii_case(key))
}

fn is_previous_upstream_default(key: &str, value: &str) -> bool {
    let default = match normalize(key).as_str() {
        "moveup" => "W",
        "movedown" => "S",
        "moveleft" => "A",
        "moveright" => "D",
        "walk" => "Left Shift",
        "a" => "Mouse Right",
        "b" => "Space",
        "x" => "Mouse Left",
        "y" => "R, Mouse Middle",
        "l" => "Q",
        "r" => "E",
        "zl" => "Tab",
        "zr" => "T",
        "plus" => "Return, Keypad Enter",
        "minus" => "Backspace",
        "dpadup" => "Up",
        "dpaddown" => "Down",
        "dpadleft" => "Left",
        "dpadright" => "Right",
        "leftstickclick" => "Left Ctrl, Right Ctrl",
        "rightstickclick" => "Left Alt, Right Alt",
        "itemleft" => "Wheel Up",
        "itemright" => "Wheel Down",
        _ => return false,
    };
    normalize(default) == normalize(value)
}

fn is_previous_conflicting_default(key: &str, value: &str) -> bool {
    let previous_defaults = match normalize(key).as_str() {
        "a" => &["F, Return"][..],
        "x" => &["G, H"][..],
        "walk" => &["Left Shift"][..],
        "r" => &["Right Shift", "Left Shift, Right Shift"][..],
        "leftstickclick" => &["Left Ctrl, Left Alt"][..],
        "rightstickclick" => &["Left Alt, Right Ctrl", "Right Ctrl, Right Alt"][..],
        _ => return false,
    };
    previous_defaults
        .iter()
        .any(|default| normalize(default) == normalize(value))
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
        let current_value = values.get(&current_address);
        let legacy_address = (field.section == "Controls")
            .then(|| address("Controller", field.key));
        if field.id == "backgroundInput" || field.id == "rumble" {
            let section_value = values.get(&address("Controls", field.key));
            if let Some(value) = section_value {
                settings.insert(field.id.to_string(), json!(value));
                continue;
            }
        }
        if field.section == "Keybinds"
            && current_value.is_some_and(|value| is_previous_conflicting_default(field.key, value))
        {
            settings.insert(field.id.to_string(), json!(field.default));
            continue;
        }
        if field.section == "Keybinds"
            && current_value.map_or(true, |value| is_previous_upstream_default(field.key, value))
        {
            if let Some(value) = migrate_legacy_binding(&values, field.key) {
                settings.insert(field.id.to_string(), json!(value));
                continue;
            } else if current_value.is_some_and(|value| {
                is_previous_upstream_default(field.key, value) && value != field.default
            }) {
                settings.insert(field.id.to_string(), json!(field.default));
                continue;
            }
        }
        if let Some(value) = current_value.or_else(|| {
            legacy_address.as_ref().and_then(|key| values.get(key))
        }) {
            let value = if field.section == "Keybinds" {
                truncate_previous_key_binding(value).unwrap_or_else(|| value.clone())
            } else {
                value.clone()
            };
            settings.insert(field.id.to_string(), json!(value));
        }
    }

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
    if value.len() == 1 && value.as_bytes()[0].is_ascii_alphanumeric() {
        return true;
    }
    [
        "Up",
        "Down",
        "Left",
        "Right",
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
        "Wheel Up",
        "Wheel Down",
        "CapsLock",
        "F2",
        "F3",
        "F4",
        "F5",
        "F6",
        "Home",
        "End",
        "PageUp",
        "PageDown",
        "-",
        "=",
        "[",
        "]",
        "\\",
        "Semicolon",
        "Comma",
        "Keypad Comma",
        "Keypad Hash",
        "'",
        "`",
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
        || ["Mouse Left", "Mouse Right", "Mouse Middle", "Mouse 4", "Mouse 5"]
            .iter()
            .any(|key| value.eq_ignore_ascii_case(key))
}

fn valid_key_binding(value: &str) -> bool {
    valid_key_binding_with_limit(value, MAX_BIND_KEYS)
}

fn valid_key_binding_for_field(value: &str, field: &Field) -> bool {
    let max_keys = match field.key {
        "DpadUp" | "DpadDown" | "DpadLeft" | "DpadRight" => 2,
        _ => MAX_BIND_KEYS,
    };
    valid_key_binding_with_limit(value, max_keys)
}

fn valid_chat_key(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    let lowered = value.to_ascii_lowercase();
    value.len() < 32
        && !value.contains(['\r', '\n', ';', '#'])
        && !matches!(
            lowered.as_str(),
            "escape" | "esc" | "insert" | "f1" | "f7" | "f8" | "f9" | "f10" | "f11" | "f12"
        )
        && !matches!(
            lowered.as_str(),
            "mouse left" | "mouse right" | "mouse middle" | "mouse 4" | "mouse 5"
                | "wheel up" | "wheel down"
        )
        && valid_keyboard_key(value)
}

fn valid_key_binding_value(value: &str, max_keys: usize) -> bool {
    if value.contains(['\r', '\n', ';', '#']) {
        return false;
    }
    let keys = value.split(',').map(str::trim).collect::<Vec<_>>();
    if keys.is_empty() || keys.len() > max_keys || keys.iter().any(|key| key.is_empty()) {
        return false;
    }
    if keys.len() == 1 && keys[0].eq_ignore_ascii_case("none") {
        return true;
    }
    keys.iter().all(|key| {
        !key.eq_ignore_ascii_case("none")
            && !matches!(
                key.to_ascii_lowercase().as_str(),
                "escape" | "esc" | "insert" | "f1" | "f7" | "f8" | "f9" | "f10" | "f11" | "f12"
            )
            && (valid_keyboard_key(key)
                || key.eq_ignore_ascii_case("Wheel Up")
                || key.eq_ignore_ascii_case("Wheel Down"))
    })
}

fn valid_matchmaking_key(value: &str) -> bool {
    value.len() < 64 && !value.contains(['\r', '\n', ';', '#'])
}

fn valid_key_binding_with_limit(value: &str, max_keys: usize) -> bool {
    valid_key_binding_value(value, max_keys)
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
            FieldKind::Key => (!valid_key_binding_for_field(value, field)).then(|| {
                format!(
                    "{} must contain up to {} supported keys or mouse buttons.",
                    field.key,
                    if matches!(field.key, "DpadUp" | "DpadDown" | "DpadLeft" | "DpadRight") {
                        2
                    } else {
                        MAX_BIND_KEYS
                    }
                )
            }),
            FieldKind::ChatKey => (!valid_chat_key(value))
                .then(|| "ChatKey must be one supported keyboard key or None.".to_string()),
            FieldKind::MatchmakingKey => (!valid_matchmaking_key(value)).then(|| {
                "Key must be under 64 bytes and must not contain newlines, ';' or '#'.".to_string()
            }),
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
            FieldKind::HubServer => (value.len() >= 256 || value.contains(['\r', '\n', ';', '#']))
                .then(|| {
                    "Hub server must be under 256 bytes and must not contain newlines, ';' or '#'."
                        .to_string()
                }),
            FieldKind::Text => (value.len() >= 256 || value.contains(['\r', '\n', ';', '#']))
                .then(|| format!("{} must be under 256 bytes and fit on one line.", field.key)),
            FieldKind::BoundedText(max) => {
                (value.len() > max || value.contains(['\r', '\n', ';', '#'])).then(|| {
                    format!("{} must be at most {max} bytes and fit on one line.", field.key)
                })
            }
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

fn controls_order(key: &str) -> usize {
    match normalize(key).as_str() {
        "facebuttons" => 0,
        "backgroundinput" => 1,
        "rumble" => 2,
        "comboinputwindow" => 3,
        "chatkey" => 4,
        _ => 100,
    }
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
    let entries = entries;
    if normalize(section).as_str() == "controls" {
        let mut face_buttons = Vec::new();
        let mut rest = Vec::new();
        for (key, value) in entries {
            if normalize(&key).as_str() == "facebuttons" {
                face_buttons.push((key, value));
            } else {
                rest.push((key, value));
            }
        }
        rest.sort_by(|(left_key, _), (right_key, _)| {
            controls_order(left_key).cmp(&controls_order(right_key))
        });
        if !face_buttons.is_empty() {
            output.push("[Controls]".to_string());
            for (key, value) in face_buttons {
                let ini_key = FIELDS
                    .iter()
                    .find(|field| normalize(field.section) == section && normalize(field.key) == key)
                    .map(|field| field.key.to_string())
                    .unwrap_or_else(|| format!("Quest{}", key.trim_start_matches("quest")));
                output.push(format!("{ini_key} = {value}"));
            }
            if !rest.is_empty() {
                output.push(String::new());
                output.push("[Controls]".to_string());
                for (key, value) in rest {
                    let ini_key = FIELDS
                        .iter()
                        .find(|field| normalize(field.section) == section && normalize(field.key) == key)
                        .map(|field| field.key.to_string())
                        .unwrap_or_else(|| format!("Quest{}", key.trim_start_matches("quest")));
                    output.push(format!("{ini_key} = {value}"));
                }
            }
            return;
        }
        for (key, value) in rest {
            let ini_key = FIELDS
                .iter()
                .find(|field| normalize(field.section) == section && normalize(field.key) == key)
                .map(|field| field.key.to_string())
                .unwrap_or_else(|| format!("Quest{}", key.trim_start_matches("quest")));
            output.push(format!("{ini_key} = {value}"));
        }
        return;
    }
    for (key, value) in entries {
        let ini_key = FIELDS
            .iter()
            .find(|field| normalize(field.section) == section && normalize(field.key) == key)
            .map(|field| field.key.to_string())
            .unwrap_or_else(|| format!("Quest{}", key.trim_start_matches("quest")));
        output.push(format!("{ini_key} = {value}"));
    }
}

fn is_deprecated_ini_entry(section: &str, key: &str) -> bool {
    let section = normalize(section);
    let key = normalize(key);
    match section.as_str() {
        "mouse" => [
            "InvertHorizontal",
            "InvertVertical",
            "Smoothing",
            "LeftButton",
            "MiddleButton",
            "RightButton",
            "X1Button",
            "X2Button",
        ]
        .iter()
        .any(|candidate| key == normalize(candidate)),
        "keyboard" => [
            "DpadUp",
            "DpadUpAlt",
            "DpadRight",
            "DpadRightAlt",
            "DpadDown",
            "DpadDownAlt",
            "DpadLeft",
            "DpadLeftAlt",
            "ButtonA",
            "ButtonAAlt",
            "ButtonB",
            "ButtonBAlt",
            "ButtonX",
            "ButtonXAlt",
            "ButtonY",
            "ButtonYAlt",
            "ButtonL",
            "ButtonLAlt",
            "ButtonR",
            "ButtonRAlt",
            "ButtonZL",
            "ButtonZLAlt",
            "ButtonZR",
            "ButtonZRAlt",
            "ButtonPlus",
            "ButtonPlusAlt",
            "ButtonPlusAlt2",
            "ButtonMinus",
            "ButtonMinusAlt",
            "ButtonLeftStick",
            "ButtonLeftStickAlt",
            "ButtonRightStick",
            "ButtonRightStickAlt",
            "StickUp",
            "StickLeft",
            "StickDown",
            "StickRight",
        ]
        .iter()
        .any(|candidate| key == normalize(candidate)),
        "controller" => ["FaceButtons", "BackgroundInput", "Rumble"]
            .iter()
            .any(|candidate| key == normalize(candidate)),
        "upscaler" => key == normalize("Upscaler"),
        _ => false,
    }
}

fn should_discard_ini_section(lines: &[String], target_section: &str) -> bool {
    let target_section = normalize(target_section);
    let mut section = String::new();
    let mut found_deprecated_entry = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if let Some(close) = trimmed.find(']') {
                section = normalize(&trimmed[1..close]);
            }
            continue;
        }
        if section != target_section || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        let Some(separator) = trimmed.find('=') else {
            continue;
        };
        if !is_deprecated_ini_entry(&section, trimmed[..separator].trim()) {
            return false;
        }
        found_deprecated_entry = true;
    }
    found_deprecated_entry
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
    let discard_keyboard_section = should_discard_ini_section(&original_lines, "Keyboard");
    let discard_controller_section = should_discard_ini_section(&original_lines, "Controller");
    let mut section = String::new();
    let has_deprecated_entries = original_lines.iter().any(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if let Some(close) = trimmed.find(']') {
                section = trimmed[1..close].to_string();
            }
            return false;
        }
        let Some(separator) = trimmed.find('=') else {
            return false;
        };
        is_deprecated_ini_entry(&section, trimmed[..separator].trim())
    });
    let current_settings = read_settings(directory)?;
    let mut updates = Vec::new();
    for field in FIELDS {
        let value = string_setting(settings, field.id).unwrap_or_default();
        let current_address = address(field.section, field.key);
        let upgrades_previous_upstream_default = field.section == "Keybinds"
            && existing
                .get(&current_address)
                .is_some_and(|value| is_previous_upstream_default(field.key, value))
            && (migrate_legacy_binding(&existing, field.key).is_some()
                || existing.get(&current_address).is_some_and(|value| value != field.default));
        let truncates_previous_key_binding = field.section == "Keybinds"
            && existing
                .get(&current_address)
                .and_then(|value| truncate_previous_key_binding(value))
                .is_some();
        let replaces_previous_conflicting_default = field.section == "Keybinds"
            && existing
                .get(&current_address)
                .is_some_and(|value| is_previous_conflicting_default(field.key, value));
        let needs_legacy_migration = ((field.section == "Controls"
            && !existing.contains_key(&current_address)
            && existing.contains_key(&address("Controller", field.key)))
            || (field.section == "Keybinds"
                && !existing.contains_key(&current_address)
                && migrate_legacy_binding(&existing, field.key).is_some()))
            || upgrades_previous_upstream_default
            || truncates_previous_key_binding
            || replaces_previous_conflicting_default;
        if current_settings.get(field.id).and_then(Value::as_str) != Some(value)
            || needs_legacy_migration
        {
            updates.push((current_address, value.to_string()));
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
    if updates.is_empty() && !update_quests && !has_deprecated_entries {
        return Ok(());
    }

    let mut missing = BTreeMap::<String, Vec<(String, String)>>::new();
    let controls_missing = !original_lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("[") && normalize(&trimmed[1..trimmed.find(']').unwrap_or(trimmed.len())]) == "controls"
    });
    if controls_missing
        && string_setting(settings, "backgroundInput").is_some()
        && string_setting(settings, "rumble").is_some()
        && updates.iter().any(|(key, _)| key == &address("Controls", "BackgroundInput"))
        && !updates.iter().any(|(key, _)| key == &address("Controls", "Rumble"))
    {
        updates.push((address("Controls", "Rumble"), string_setting(settings, "rumble").unwrap().to_string()));
    }
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
    section.clear();
    let mut seen_control_order = false;
    for line in &original_lines {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if let Some(close) = trimmed.find(']') {
                append_missing_entries(&mut output, &mut missing, &section);
                section = normalize(&trimmed[1..close]);
                if section == "controls" { seen_control_order = true; }
            }
        }
        if (section == "keyboard" && discard_keyboard_section)
            || (section == "controller" && discard_controller_section)
        {
            continue;
        }
        if !trimmed.starts_with(';') && !trimmed.starts_with('#') {
            if let Some(separator) = trimmed.find('=') {
                let key = normalize(trimmed[..separator].trim());
                if is_deprecated_ini_entry(&section, &key) {
                    continue;
                }
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
    if !seen_control_order && !missing.is_empty() {
        for name in CANONICAL_SECTIONS {
            let normalized = normalize(name);
            if normalized == "controls" && missing.contains_key(&normalized) {
                if !output.is_empty() && output.last().is_some_and(|line| !line.is_empty()) {
                    output.push(String::new());
                }
                output.push("[Controls]".to_string());
                append_missing_entries(&mut output, &mut missing, &normalized);
                break;
            }
        }
    }
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
        game_exists: directory.join(GAME_EXECUTABLE).is_file(),
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
    let game = directory.join(GAME_EXECUTABLE);
    if !game.is_file() {
        return Err(format!("{GAME_EXECUTABLE} was not found next to the launcher."));
    }
    game_command(&game, &directory)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not launch {GAME_EXECUTABLE}: {error}"))
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

const GAMEBANANA_GAME_ID: u64 = 17088;
const MAX_MOD_ARCHIVE_BYTES: usize = 512 * 1024 * 1024;
const MAX_MOD_UNPACKED_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModFile {
    id: String,
    name: String,
    version: String,
    size: u64,
    added: u64,
    download_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModImage {
    url: String,
    alt: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModDetails {
    description_html: String,
    images: Vec<ModImage>,
    files: Vec<ModFile>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModEntry {
    id: String,
    name: String,
    version: String,
    updated: u64,
    url: String,
    installed: bool,
    update_available: bool,
}

fn gamebanana_get(url: &str) -> Result<Value, String> {
    reqwest::blocking::Client::builder()
        .user_agent("MHGU-Recomp-Launcher/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())?
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("GameBanana request failed: {error}"))?
        .json::<Value>()
        .map_err(|error| format!("Could not read GameBanana response: {error}"))
}

fn gb_string(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
}

fn gb_number(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or_default()
}

fn mod_registry_path(directory: &Path) -> PathBuf {
    directory.join("mods").join(".launcher-gamebanana.json")
}

fn read_mod_registry(directory: &Path) -> HashMap<String, Value> {
    fs::read(mod_registry_path(directory))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn write_mod_registry(directory: &Path, registry: &HashMap<String, Value>) -> Result<(), String> {
    let path = mod_registry_path(directory);
    fs::create_dir_all(path.parent().expect("registry parent"))
        .map_err(|error| format!("Could not create mods folder: {error}"))?;
    fs::write(path, serde_json::to_vec_pretty(registry).map_err(|error| error.to_string())?)
        .map_err(|error| format!("Could not save installed mod list: {error}"))
}

fn gb_mod_file_list(item: &Value) -> Result<Vec<ModFile>, String> {
    let files = item.get("_aFiles")
        .ok_or_else(|| "This GameBanana mod has no downloadable files.".to_string())?;
    let entries = if let Some(files) = files.as_array() {
        files.iter().collect::<Vec<_>>()
    } else if let Some(files) = files.as_object() {
        files.values().collect::<Vec<_>>()
    } else {
        return Err("This GameBanana mod has no downloadable files.".to_string());
    };
    let mut result = entries.into_iter().filter_map(|file| {
        let id = gb_number(file, "_idRow").to_string();
        let download_url = gb_string(file, "_sDownloadUrl");
        if id == "0" || !download_url.starts_with("https://gamebanana.com/dl/") { return None; }
        Some(ModFile {
            id,
            name: gb_string(file, "_sFile"),
            version: gb_string(file, "_sVersion"),
            size: gb_number(file, "_nFilesize"),
            added: gb_number(file, "_tsDateAdded"),
            download_url,
        })
    }).collect::<Vec<_>>();
    result.sort_by_key(|file| file.added);
    if result.is_empty() { return Err("This GameBanana mod has no downloadable files.".to_string()); }
    Ok(result)
}

fn gb_mod_files(mod_id: &str) -> Result<Vec<ModFile>, String> {
    if !mod_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Invalid GameBanana mod ID.".to_string());
    }
    let item = gamebanana_get(&format!("https://gamebanana.com/apiv11/Mod/{mod_id}/ProfilePage"))?;
    gb_mod_file_list(&item)
}

fn gb_mod_images(item: &Value, name: &str) -> Vec<ModImage> {
    item.pointer("/_aPreviewMedia/_aImages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|image| {
            let base = gb_string(image, "_sBaseUrl");
            let preferred = gb_string(image, "_sFile530");
            let filename = if preferred.is_empty() { gb_string(image, "_sFile") } else { preferred };
            if !base.starts_with("https://images.gamebanana.com/")
                || filename.is_empty()
                || filename.contains('/')
                || filename.contains('\\')
                || filename.contains("..")
            {
                return None;
            }
            Some(ModImage {
                url: format!("{}/{}", base.trim_end_matches('/'), filename),
                alt: format!("{name} preview"),
            })
        })
        .take(8)
        .collect()
}

#[tauri::command]
fn get_gamebanana_mods() -> Result<Vec<ModEntry>, String> {
    let directory = app_directory()?;
    let registry = read_mod_registry(&directory);
    let first_page = gamebanana_get("https://gamebanana.com/apiv13/Mod/Index?_nPerpage=15&_aFilters%5BGeneric_Game%5D=17088&_nPage=1")?;
    let record_count = first_page.pointer("/_aMetadata/_nRecordCount").and_then(Value::as_u64)
        .ok_or_else(|| "GameBanana returned invalid catalog metadata.".to_string())?;
    let page_count = record_count.div_ceil(15).min(100);
    let mut records = first_page.get("_aRecords").and_then(Value::as_array).cloned()
        .ok_or_else(|| "GameBanana returned an invalid mod list.".to_string())?;
    for page in 2..=page_count {
        let response = gamebanana_get(&format!(
            "https://gamebanana.com/apiv13/Mod/Index?_nPerpage=15&_aFilters%5BGeneric_Game%5D=17088&_nPage={page}"
        ))?;
        let page_records = response.get("_aRecords").and_then(Value::as_array)
            .ok_or_else(|| "GameBanana returned an invalid mod list page.".to_string())?;
        records.extend(page_records.iter().cloned());
    }
    let mut result = Vec::new();
    for item in &records {
        if item.pointer("/_aGame/_idRow").and_then(Value::as_u64) != Some(GAMEBANANA_GAME_ID)
            || item.get("_bHasFiles").and_then(Value::as_bool) != Some(true) { continue; }
        let id = gb_number(item, "_idRow").to_string();
        let installed = registry.get(&id);
        let updated = gb_number(item, "_tsDateUpdated");
        let installed_updated = installed.and_then(|entry| entry.get("updated")).and_then(Value::as_u64).unwrap_or_default();
        result.push(ModEntry {
            id: id.clone(),
            name: gb_string(item, "_sName"),
            version: gb_string(item, "_sVersion"),
            updated,
            url: format!("https://gamebanana.com/mods/{id}"),
            installed: installed.is_some(),
            update_available: installed.is_some() && updated > installed_updated,
        });
    }
    Ok(result)
}

#[tauri::command]
fn get_gamebanana_mod_details(mod_id: String) -> Result<ModDetails, String> {
    if !mod_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Invalid GameBanana mod ID.".to_string());
    }
    let item = gamebanana_get(&format!("https://gamebanana.com/apiv11/Mod/{mod_id}/ProfilePage"))?;
    if item.pointer("/_aGame/_idRow").and_then(Value::as_u64) != Some(GAMEBANANA_GAME_ID) {
        return Err("This mod is not listed for Monster Hunter Generations Ultimate.".to_string());
    }
    let name = gb_string(&item, "_sName");
    Ok(ModDetails {
        description_html: gb_string(&item, "_sText"),
        images: gb_mod_images(&item, &name),
        files: gb_mod_file_list(&item)?,
    })
}

fn safe_mod_name(name: &str, id: &str) -> String {
    let cleaned = name.chars().map(|character| {
        if character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_' | '.') { character } else { '_' }
    }).collect::<String>().trim_matches([' ', '.']).to_string();
    format!("{}-{}", cleaned.chars().take(72).collect::<String>(), id)
}

fn extract_mod_archive(bytes: &[u8], destination: &Path) -> Result<(), String> {
    if bytes.len() > MAX_MOD_ARCHIVE_BYTES { return Err("Mod archive exceeds the 512 MiB download limit.".to_string()); }
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|error| format!("Invalid ZIP archive: {error}"))?;
    let mut total_size = 0u64;
    let mut count = 0usize;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|error| error.to_string())?;
        count += 1;
        total_size = total_size.saturating_add(file.size());
        if count > 20_000 || total_size > MAX_MOD_UNPACKED_BYTES {
            return Err("Mod archive exceeds extraction limits.".to_string());
        }
        let Some(relative) = file.enclosed_name() else {
            return Err("Mod archive contains an unsafe file path.".to_string());
        };
        let output = destination.join(relative);
        if file.is_dir() {
            fs::create_dir_all(&output).map_err(|error| error.to_string())?;
        } else {
            if let Some(parent) = output.parent() { fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
            let mut output_file = fs::File::create(output).map_err(|error| error.to_string())?;
            std::io::copy(&mut file, &mut output_file).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn has_mod_directory(root: &Path, name: &str, depth: usize) -> bool {
    if root.join(name).is_dir() { return true; }
    if depth == 0 { return false; }
    fs::read_dir(root).ok().into_iter().flatten().filter_map(Result::ok).any(|entry| {
        entry.file_type().is_ok_and(|kind| kind.is_dir())
            && !entry.file_name().to_string_lossy().starts_with('.')
            && has_mod_directory(&entry.path(), name, depth - 1)
    })
}

#[tauri::command]
fn install_gamebanana_mod(mod_id: String, file_ids: Vec<String>) -> Result<(), String> {
    if !mod_id.bytes().all(|byte| byte.is_ascii_digit()) || file_ids.is_empty() || file_ids.len() > 20 {
        return Err("Select one or more valid mod files.".to_string());
    }
    let files = gb_mod_files(&mod_id)?;
    let selected = files.into_iter().filter(|file| file_ids.contains(&file.id)).collect::<Vec<_>>();
    if selected.len() != file_ids.iter().collect::<HashSet<_>>().len() {
        return Err("One or more selected files do not belong to this mod.".to_string());
    }
    let details = gamebanana_get(&format!("https://gamebanana.com/apiv11/Mod/{mod_id}/ProfilePage"))?;
    let name = gb_string(&details, "_sName");
    let game_id = details.pointer("/_aGame/_idRow").and_then(Value::as_u64);
    if game_id != Some(GAMEBANANA_GAME_ID) { return Err("This mod is not listed for Monster Hunter Generations Ultimate.".to_string()); }
    let directory = app_directory()?;
    let mods = directory.join("mods");
    fs::create_dir_all(&mods).map_err(|error| format!("Could not create mods folder: {error}"))?;
    let folder_name = safe_mod_name(&name, &mod_id);
    let staging = mods.join(format!(".install-{mod_id}"));
    if staging.exists() { fs::remove_dir_all(&staging).map_err(|error| error.to_string())?; }
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    let result = (|| {
        for file in &selected {
            let response = reqwest::blocking::Client::builder()
                .user_agent("MHGU-Recomp-Launcher/1.0")
                .timeout(std::time::Duration::from_secs(120))
                .build().map_err(|error| error.to_string())?
                .get(&file.download_url).send()
                .and_then(reqwest::blocking::Response::error_for_status)
                .map_err(|error| format!("Could not download {}: {error}", file.name))?;
            if response.content_length().is_some_and(|size| size > MAX_MOD_ARCHIVE_BYTES as u64) {
                return Err(format!("{} exceeds the 512 MiB download limit.", file.name));
            }
            let mut bytes = Vec::new();
            response.take(MAX_MOD_ARCHIVE_BYTES as u64 + 1).read_to_end(&mut bytes)
                .map_err(|error| format!("Could not read {}: {error}", file.name))?;
            extract_mod_archive(&bytes, &staging)?;
        }
        let mut entries = fs::read_dir(&staging).map_err(|error| error.to_string())?;
        let root = entries.next().and_then(Result::ok).map(|entry| entry.path());
        let source = if let Some(root) = root.filter(|path| path.is_dir()) {
            if fs::read_dir(&staging).map_err(|error| error.to_string())?.count() == 1 { root } else { staging.clone() }
        } else { staging.clone() };
        let valid_layout = ["romfs", "exefs", "nativeNX", "subsdk0"]
            .iter().any(|folder| has_mod_directory(&source, folder, 4));
        if !valid_layout { return Err("The selected archive does not contain a recognized MHGU mod folder (romfs, exefs, nativeNX, or subsdk).".to_string()); }
        let final_path = mods.join(&folder_name);
        let backup = mods.join(format!(".backup-{mod_id}"));
        if backup.exists() { fs::remove_dir_all(&backup).map_err(|error| error.to_string())?; }
        if final_path.exists() { fs::rename(&final_path, &backup).map_err(|error| format!("Could not replace the previous mod: {error}"))?; }
        let move_result = fs::rename(&source, &final_path).map_err(|error| format!("Could not install mod: {error}"));
        if move_result.is_err() && backup.exists() { let _ = fs::rename(&backup, &final_path); }
        move_result?;
        if backup.exists() { fs::remove_dir_all(backup).map_err(|error| error.to_string())?; }
        let mut registry = read_mod_registry(&directory);
        registry.insert(mod_id.clone(), json!({
            "name": name,
            "folder": folder_name,
            "fileIds": selected.iter().map(|file| file.id.clone()).collect::<Vec<_>>(),
            "updated": gb_number(&details, "_tsDateUpdated")
        }));
        write_mod_registry(&directory, &registry)
    })();
    if staging.exists() { let _ = fs::remove_dir_all(staging); }
    result
}

fn is_safe_managed_mod_folder(folder: &str, mod_id: &str) -> bool {
    !folder.is_empty()
        && !folder.starts_with('.')
        && !folder.contains('/')
        && !folder.contains('\\')
        && folder.ends_with(&format!("-{mod_id}"))
}

#[tauri::command]
fn remove_gamebanana_mod(mod_id: String) -> Result<(), String> {
    if !mod_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Invalid GameBanana mod ID.".to_string());
    }
    let directory = app_directory()?;
    let mods = directory.join("mods");
    let mut registry = read_mod_registry(&directory);
    let folder = registry.get(&mod_id)
        .and_then(|entry| entry.get("folder"))
        .and_then(Value::as_str)
        .ok_or_else(|| "This mod was not installed by the launcher.".to_string())?
        .to_string();
    if !is_safe_managed_mod_folder(&folder, &mod_id) {
        return Err("The recorded mod folder is not safe to remove.".to_string());
    }
    let path = mods.join(&folder);
    if path.exists() {
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err("The recorded mod path is not a regular directory.".to_string());
        }
        fs::remove_dir_all(&path).map_err(|error| format!("Could not remove mod folder: {error}"))?;
    }
    registry.remove(&mod_id);
    write_mod_registry(&directory, &registry)
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
            open_ini,
            get_gamebanana_mods,
            get_gamebanana_mod_details,
            install_gamebanana_mod,
            remove_gamebanana_mod
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
    use std::io::Write;
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
    fn mod_archive_extracts_switch_layout_and_rejects_parent_paths() {
        let directory = temp_directory();
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        archive.start_file(
            "atmosphere/contents/0100770008DD8000/romfs/test.bin",
            zip::write::SimpleFileOptions::default(),
        ).expect("add valid mod file");
        archive.write_all(b"mod data").expect("write valid mod file");
        let bytes = archive.finish().expect("finish valid archive").into_inner();
        extract_mod_archive(&bytes, &directory).expect("extract valid mod archive");
        assert!(has_mod_directory(&directory, "romfs", 4));
        assert_eq!(fs::read(directory.join("atmosphere/contents/0100770008DD8000/romfs/test.bin")).unwrap(), b"mod data");

        let mut unsafe_archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        unsafe_archive.start_file("../outside.bin", zip::write::SimpleFileOptions::default())
            .expect("add unsafe archive path");
        unsafe_archive.write_all(b"outside").expect("write unsafe archive entry");
        let unsafe_bytes = unsafe_archive.finish().expect("finish unsafe archive").into_inner();
        assert!(extract_mod_archive(&unsafe_bytes, &directory).is_err());
        assert!(!directory.parent().unwrap().join("outside.bin").exists());
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn gamebanana_file_list_accepts_array_and_object_shapes() {
        let file = json!({
            "_idRow": 1886003,
            "_sFile": "weapon-transmog.zip",
            "_sVersion": "1.2.1",
            "_nFilesize": 2997,
            "_tsDateAdded": 1794454850,
            "_sDownloadUrl": "https://gamebanana.com/dl/1886003"
        });
        let array = json!({ "_aFiles": [file.clone()] });
        let object = json!({ "_aFiles": { "1886003": file } });

        assert_eq!(gb_mod_file_list(&array).expect("array file list")[0].id, "1886003");
        assert_eq!(gb_mod_file_list(&object).expect("object file list")[0].id, "1886003");
    }

    #[test]
    fn managed_mod_folder_validation_rejects_unmanaged_and_unsafe_paths() {
        assert!(is_safe_managed_mod_folder("Weapon Transmog-726009", "726009"));
        assert!(!is_safe_managed_mod_folder("Custom Mod", "726009"));
        assert!(!is_safe_managed_mod_folder("../outside-726009", "726009"));
        assert!(!is_safe_managed_mod_folder(".backup-726009", "726009"));
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
    fn keybinds_from_four_key_launcher_version_keep_an_extra_binding() {
        let directory = temp_directory();
        fs::write(
            ini_path(&directory),
            "[Keybinds]\nA = J, K, L, Mouse Middle\n",
        )
        .expect("write previous keybind format");

        let settings = read_settings(&directory).expect("load previous keybind format");
        assert_eq!(settings["bindA"], "J, K, Mouse Middle");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save three-binding format");
        let output = fs::read_to_string(ini_path(&directory)).expect("read three-binding format");
        assert!(output.contains("A = J, K, Mouse Middle"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn launcher_exposes_runtime_settings_and_validates_chat_key() {
        let settings = default_settings();
        for id in [
            "ssao",
            "fxaaMode",
            "smoothGradients",
            "hdr",
            "hdrPeak",
            "hdrGame",
            "hdrUi",
            "menuConfirm",
            "buttonIcons",
            "chatKey",
            "volume",
            "matchmakingKey",
            "discordRichPresence",
            "discordShowHunter",
            "showFps",
            "shareUrgentCredit",
            "lagCompensation",
            "ryuServers",
            "roomServer",
            "saveLocation",
            "confirmExit",
            "language",
            "menuTheme",
            "menuScale",
            "updates",
        ] {
            assert!(settings.get(id).is_some(), "missing launcher setting: {id}");
        }
        assert_eq!(settings["confirmExit"], "On");
        assert_eq!(settings["showFps"], "Off");
        assert_eq!(settings["shareUrgentCredit"], "On");
        assert!(settings.get("fsrUpscaler").is_none());
        assert!(validate_settings(&settings).is_empty());

        let mut settings = settings;
        settings["chatKey"] = json!("Comma");
        assert!(validate_settings(&settings).is_empty());
        settings["chatKey"] = json!("Mouse Left");
        assert!(validate_settings(&settings).contains_key("chatKey"));
        settings["chatKey"] = json!("F12");
        assert!(validate_settings(&settings).contains_key("chatKey"));
        settings["chatKey"] = json!("");
        assert!(validate_settings(&settings).is_empty());
        settings["matchmakingKey"] = json!("x".repeat(64));
        assert!(validate_settings(&settings).contains_key("matchmakingKey"));
        settings["matchmakingKey"] = json!("friends");
        settings["join"] = json!("host#comment");
        assert!(validate_settings(&settings).contains_key("join"));
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
        let game = directory.join(GAME_EXECUTABLE);
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
        assert!(!output.contains("Upscaler ="));
        assert!(output.contains("FSRQuality = Balanced"));
        assert!(output.contains("Sharpness = 0.5"));
        assert!(output.contains("ObjectMotion = Off"));
        assert!(output.contains("Reactive = 1.0"));
        assert!(output.contains("AnimSurf = Off"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["fsr"], "On");
        assert!(loaded.get("fsrUpscaler").is_none());
        assert_eq!(loaded["fsrQuality"], "Balanced");
        assert_eq!(loaded["fsrSharpness"], "0.5");
        assert_eq!(loaded["fsrObjectMotion"], "Off");
        assert_eq!(loaded["fsrReactive"], "1.0");
        assert_eq!(loaded["fsrAnimSurf"], "Off");

        settings["fsrSharpness"] = json!("1.1");
        assert!(validate_settings(&settings).contains_key("fsrSharpness"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn retired_fsr4_selector_is_removed_when_saving() {
        let directory = temp_directory();
        fs::write(
            ini_path(&directory),
            "[Upscaler]\nFSR = On\nUpscaler = FSR4\nFSRQuality = Balanced\n",
        )
        .expect("write previous FSR4 settings");

        let settings = read_settings(&directory).expect("load previous settings");
        assert_eq!(settings["fsr"], "On");
        assert_eq!(settings["fsrQuality"], "Balanced");
        assert!(settings.get("fsrUpscaler").is_none());
        save_settings(&directory, &settings).expect("remove retired upscaler setting");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("FSR = On"));
        assert!(output.contains("FSRQuality = Balanced"));
        assert!(!output.contains("Upscaler = FSR4"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn new_runtime_toggles_are_saved() {
        let directory = temp_directory();
        let mut settings = default_settings();
        settings["showFps"] = json!("On");
        settings["shareUrgentCredit"] = json!("Off");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save new runtime settings");
        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("ShowFPS = On"));
        assert!(output.contains("ShareUrgentCredit = Off"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["showFps"], "On");
        assert_eq!(loaded["shareUrgentCredit"], "Off");
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn hub_server_address_is_validated_and_saved() {
        let directory = temp_directory();
        let mut settings = read_settings(&directory).expect("load defaults");
        assert_eq!(settings["hubServer"], "");
        assert!(validate_settings(&settings).is_empty());

        settings["hubServer"] = json!("hub.example.org:39510");
        assert!(validate_settings(&settings).is_empty());
        save_settings(&directory, &settings).expect("save hub server address");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("[Multiplayer]\nServer = hub.example.org:39510"));
        let loaded = read_settings(&directory).expect("reload hub server address");
        assert_eq!(loaded["hubServer"], "hub.example.org:39510");

        settings["hubServer"] = json!("Off");
        save_settings(&directory, &settings).expect("disable internet hubs");
        assert_eq!(
            read_settings(&directory).expect("reload disabled hub server")["hubServer"],
            "Off"
        );

        for invalid in ["host;comment", "host#comment", "host\nOther = value"] {
            settings["hubServer"] = json!(invalid);
            assert!(validate_settings(&settings).contains_key("hubServer"));
        }
        settings["hubServer"] = json!("x".repeat(256));
        assert!(validate_settings(&settings).contains_key("hubServer"));
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
        settings["rumble"] = json!("On");
        assert!(validate_settings(&settings).is_empty());
        save_settings(&directory, &settings).expect("save display settings");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("AspectRatio = 21:9"));
        assert!(output.contains("FOV = 1.2"));
        assert!(output.contains("UIAnchor = Edges"));
        assert!(output.contains("[Controls]\nBackgroundInput = Off\nRumble = On"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["aspectRatio"], "21:9");
        assert_eq!(loaded["fov"], "1.2");
        assert_eq!(loaded["uiAnchor"], "Edges");
        assert_eq!(loaded["backgroundInput"], "Off");
        assert_eq!(loaded["rumble"], "On");

        settings["aspectRatio"] = json!("21:0");
        assert!(validate_settings(&settings).contains_key("aspectRatio"));
        settings["aspectRatio"] = json!("0");
        assert!(validate_settings(&settings).contains_key("aspectRatio"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn mouse_and_item_bar_settings_round_trip() {
        let directory = temp_directory();
        fs::write(ini_path(&directory), "[Mouse]\nMouseSensitivity = 1.0\n")
            .expect("write mouse settings");
        let mut settings = default_settings();
        assert_eq!(settings["bindItemLeft"], "Wheel Up");
        assert_eq!(settings["bindItemRight"], "Wheel Down");
        settings["mouseSensitivity"] = json!("1.5");
        settings["bindItemLeft"] = json!("J");
        settings["bindItemRight"] = json!("Wheel Up");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save mouse and item bar settings");
        let output =
            fs::read_to_string(ini_path(&directory)).expect("read mouse and item bar settings");
        assert!(output.contains("MouseSensitivity = 1.5"));
        assert!(output.contains("[Keybinds]\nItemLeft = J\nItemRight = Wheel Up"));

        let loaded = read_settings(&directory).expect("reload mouse and item bar settings");
        assert_eq!(loaded["mouseSensitivity"], "1.5");
        assert_eq!(loaded["bindItemLeft"], "J");
        assert_eq!(loaded["bindItemRight"], "Wheel Up");
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn controller_settings_migrate_legacy_controller_section() {
        let directory = temp_directory();
        let original = "[Controller]\nFaceButtons = Label\nBackgroundInput = Off\nRumble = Off\n";
        fs::write(ini_path(&directory), original).expect("write legacy controller settings");

        let mut settings = read_settings(&directory).expect("load legacy settings");
        assert_eq!(settings["faceButtons"], "Label");
        assert_eq!(settings["backgroundInput"], "Off");
        assert_eq!(settings["rumble"], "Off");

        settings["backgroundInput"] = json!("On");
        save_settings(&directory, &settings).expect("save updated controller settings");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(!output.contains("[Controller]"));
        assert!(output.contains("[Controls]\nFaceButtons = Label"));
        assert!(output.contains("[Controls]\nBackgroundInput = On"));
        let loaded = read_settings(&directory).expect("reload controller settings");
        assert_eq!(loaded["faceButtons"], "Label");
        assert_eq!(loaded["backgroundInput"], "On");
        assert_eq!(loaded["rumble"], "Off");
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn saving_removes_deprecated_bindings_and_preserves_unknown_settings() {
        let directory = temp_directory();
        fs::write(
            ini_path(&directory),
            "[Mouse]\nMouseSensitivity = 1.0\nFutureMouseOption = preserve-mouse\n\
             InvertHorizontal = On\nSmoothing = 0.7\nLeftButton = X\n\
             [Keyboard]\nButtonA = F\nStickUp = W\n[Controller]\nFaceButtons = Label\nRumble = Off\n\
             FutureControllerOption = preserve-controller\n\
             [Future section]\nFutureOption = preserve-me\n",
        )
        .expect("write deprecated and unknown settings");

        save_settings(&directory, &default_settings()).expect("clean deprecated settings");
        let output = fs::read_to_string(ini_path(&directory)).expect("read cleaned ini");
        assert!(output.contains("MouseSensitivity = 1.0"));
        assert!(output.contains("FutureMouseOption = preserve-mouse"));
        assert!(output.contains("FutureControllerOption = preserve-controller"));
        assert!(output.contains("FutureOption = preserve-me"));
        assert!(!output.contains("[Keyboard]"));
        assert!(!output.contains("InvertHorizontal"));
        assert!(!output.contains("Smoothing"));
        assert!(!output.contains("LeftButton ="));
        assert!(!output.contains("ButtonA ="));
        assert!(!output.contains("StickUp ="));
        assert!(!output.contains("FaceButtons = Label"));
        assert!(!output.contains("Rumble = Off"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn legacy_keyboard_and_mouse_bindings_migrate_into_keybinds() {
        let directory = temp_directory();
        fs::write(
            ini_path(&directory),
            "[Keyboard]\nButtonA = F\nButtonAAlt = Return\nButtonX = G\nButtonXAlt = H\n\
             StickUp = W\nDpadUp = 1\nDpadUpAlt = Up\n[Mouse]\nRightButton = A\nLeftButton = X\n\
             [Keybinds]\nA = Mouse Right\nX = Mouse Left\n",
        )
        .expect("write legacy keyboard and mouse settings");

        let settings = read_settings(&directory).expect("migrate legacy bindings");
        assert_eq!(settings["bindA"], "F, Return, Mouse Right");
        assert_eq!(settings["bindX"], "G, H, Mouse Left");
        assert_eq!(settings["bindMoveUp"], "W");
        assert_eq!(settings["bindDpadUp"], "1, Up");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save migrated bindings");
        let output = fs::read_to_string(ini_path(&directory)).expect("read migrated bindings");
        assert!(!output.contains("[Keyboard]"));
        assert!(!output.contains("RightButton ="));
        assert!(output.contains("A = F, Return, Mouse Right"));
        assert!(output.contains("X = G, H, Mouse Left"));
        assert!(output.contains("MoveUp = W"));
        assert!(output.contains("DpadUp = 1, Up"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn previous_upstream_keybind_defaults_upgrade_to_the_mhw_layout() {
        let directory = temp_directory();
        fs::write(
            ini_path(&directory),
            "[Keybinds]\nA = Mouse Right\nB = Space\nX = Mouse Left\nY = R, Mouse Middle\n\
             R = E\nDpadUp = Up\n",
        )
        .expect("write previous keybind defaults");

        let settings = read_settings(&directory).expect("upgrade old keybind defaults");
        assert_eq!(settings["bindA"], "F, Return, Mouse Right");
        assert_eq!(settings["bindB"], "Space, Backspace");
        assert_eq!(settings["bindX"], "G, H, Mouse Left");
        assert_eq!(settings["bindY"], "E, T");
        assert_eq!(settings["bindR"], "Left Shift");
        assert_eq!(settings["bindDpadUp"], "1, Up");

        save_settings(&directory, &settings).expect("save upgraded keybind defaults");
        let output = fs::read_to_string(ini_path(&directory)).expect("read upgraded keybinds");
        assert!(output.contains("A = F, Return, Mouse Right"));
        assert!(output.contains("B = Space, Backspace"));
        assert!(output.contains("X = G, H, Mouse Left"));
        assert!(output.contains("Y = E, T"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn default_keybinds_do_not_share_inputs() {
        let settings = default_settings();
        assert_eq!(settings["bindA"], "F, Return, Mouse Right");
        assert_eq!(settings["bindX"], "G, H, Mouse Left");
        let mut assigned = HashMap::new();
        for field in FIELDS.iter().filter(|field| field.id.starts_with("bind")) {
            let value = settings[field.id].as_str().expect("keybind default");
            for key in value.split(',').map(str::trim) {
                let normalized = key.to_ascii_lowercase();
                assert!(
                    !matches!(normalized.as_str(), "right ctrl" | "right alt" | "right shift"),
                    "right-side modifier must not be a default binding: {key}"
                );
                assert!(
                    assigned.insert(normalized, field.id).is_none(),
                    "default keybind collision in {}: {key}",
                    field.id
                );
            }
        }
    }

    #[test]
    fn previous_overlapping_keybind_defaults_are_migrated() {
        let directory = temp_directory();
        fs::write(
            ini_path(&directory),
            "[Keybinds]\nA = F, Return\nX = G, H\nWalk = Left Shift\nR = Left Shift, Right Shift\n\
             LeftStickClick = Left Ctrl, Left Alt\nRightStickClick = Left Alt, Right Ctrl\n",
        )
        .expect("write previous keybind defaults");

        let settings = read_settings(&directory).expect("migrate overlapping keybind defaults");
        assert_eq!(settings["bindA"], "F, Return, Mouse Right");
        assert_eq!(settings["bindX"], "G, H, Mouse Left");
        assert_eq!(settings["bindWalk"], "CapsLock");
        assert_eq!(settings["bindLeftStickClick"], "Left Ctrl");
        assert_eq!(settings["bindR"], "Left Shift");
        assert_eq!(settings["bindRightStickClick"], "Left Alt");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save migrated keybind defaults");
        let output = fs::read_to_string(ini_path(&directory)).expect("read migrated keybinds");
        assert!(output.contains("A = F, Return, Mouse Right"));
        assert!(output.contains("X = G, H, Mouse Left"));
        assert!(output.contains("Walk = CapsLock"));
        assert!(output.contains("LeftStickClick = Left Ctrl"));
        assert!(output.contains("R = Left Shift"));
        assert!(output.contains("RightStickClick = Left Alt"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn previous_right_shift_default_moves_to_left_shift() {
        let directory = temp_directory();
        fs::write(ini_path(&directory), "[Keybinds]\nR = Right Shift\n")
            .expect("write previous R default");

        let settings = read_settings(&directory).expect("migrate previous R default");
        assert_eq!(settings["bindR"], "Left Shift");
        assert!(validate_settings(&settings).is_empty());

        save_settings(&directory, &settings).expect("save updated R default");
        let output = fs::read_to_string(ini_path(&directory)).expect("read updated keybind");
        assert!(output.contains("R = Left Shift"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }

    #[test]
    fn keybinds_accept_two_keys_mouse_buttons_and_wheel() {
        let directory = temp_directory();
        let mut settings = default_settings();
        settings["bindA"] = json!("J, Mouse 4");
        settings["bindY"] = json!("K, Mouse Middle");
        settings["bindDpadUp"] = json!("I");
        settings["bindItemLeft"] = json!("Wheel Down");
        assert!(validate_settings(&settings).is_empty());
        save_settings(&directory, &settings).expect("save keybinds");

        let output = fs::read_to_string(ini_path(&directory)).expect("read updated ini");
        assert!(output.contains("[Keybinds]\nA = J, Mouse 4"));
        assert!(output.contains("Y = K, Mouse Middle"));
        assert!(output.contains("DpadUp = I"));
        assert!(output.contains("ItemLeft = Wheel Down"));
        let loaded = read_settings(&directory).expect("reload settings");
        assert_eq!(loaded["bindA"], "J, Mouse 4");
        assert_eq!(loaded["bindY"], "K, Mouse Middle");
        assert_eq!(loaded["bindDpadUp"], "I");
        assert_eq!(loaded["bindItemLeft"], "Wheel Down");

        settings["bindDpadUp"] = json!("not-a-key");
        assert!(validate_settings(&settings).contains_key("bindDpadUp"));
        settings["bindDpadUp"] = json!("J, K, L");
        assert!(validate_settings(&settings).contains_key("bindDpadUp"));
        settings["bindDpadUp"] = json!("None");
        assert!(validate_settings(&settings).is_empty());
        settings["bindDpadUp"] = json!("Escape");
        assert!(validate_settings(&settings).contains_key("bindDpadUp"));
        settings["bindDpadUp"] = json!("W, A, S, D, Space");
        assert!(validate_settings(&settings).contains_key("bindDpadUp"));
        settings["bindDpadUp"] = json!("W,\nA");
        assert!(validate_settings(&settings).contains_key("bindDpadUp"));
        fs::remove_dir_all(directory).expect("clean temp directory");
    }
}
