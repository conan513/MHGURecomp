#define UNICODE
#define _UNICODE
#include <windows.h>
#include <commctrl.h>
#include <shellapi.h>
#include <stdint.h>
#include <stdio.h>
#include <algorithm>
#include <cwctype>
#include <utility>
#include <map>
#include <sstream>
#include <string>
#include <vector>

namespace {

enum ControlId {
    TAB_CONTROL = 1,
    MODE = 100,
    RESOLUTION,
    RENDER_RESOLUTION,
    MSAA,
    FXAA,
    FPS,
    VSYNC,
    FACE_BUTTONS,
    MOUSE_SENSITIVITY,
    PORT,
    JOIN,
    SYNC_SMALL_MONSTERS,
    VILLAGE_IN_HUB,
    HUB_HP,
    BONUS_CONTENT,
    HP_1,
    HP_2,
    HP_3,
    HP_4,
    QUESTS,
    SAVE,
    SAVE_AND_LAUNCH,
    RESET,
    OPEN_INI
};

struct Settings {
    std::wstring mode = L"Borderless";
    std::wstring resolution = L"1920x1080";
    std::wstring render_resolution = L"Auto";
    std::wstring msaa = L"Off";
    std::wstring fxaa = L"On";
    std::wstring fps = L"60";
    std::wstring vsync = L"On";
    std::wstring face_buttons = L"Position";
    std::wstring mouse_sensitivity = L"1.0";
    std::wstring port = L"39500";
    std::wstring join;
    std::wstring sync_small_monsters = L"On";
    std::wstring village_in_hub = L"On";
    std::wstring hub_hp = L"2.0";
    std::wstring bonus_content = L"On";
    std::wstring hp[4] = {L"1.0", L"1.0", L"1.0", L"1.0"};
    std::wstring quests;
};

struct Field {
    int id;
    const wchar_t *section;
    const wchar_t *key;
};

const Field fields[] = {
    {MODE, L"Display", L"Mode"},
    {RESOLUTION, L"Display", L"Resolution"},
    {RENDER_RESOLUTION, L"Display", L"RenderResolution"},
    {MSAA, L"Anti-aliasing", L"MSAA"},
    {FXAA, L"Anti-aliasing", L"FXAA"},
    {FPS, L"Performance", L"FPS"},
    {VSYNC, L"Performance", L"VSync"},
    {FACE_BUTTONS, L"Controls", L"FaceButtons"},
    {MOUSE_SENSITIVITY, L"Mouse", L"MouseSensitivity"},
    {PORT, L"Multiplayer", L"Port"},
    {JOIN, L"Multiplayer", L"Join"},
    {SYNC_SMALL_MONSTERS, L"Multiplayer", L"SyncSmallMonsters"},
    {VILLAGE_IN_HUB, L"Village Quests", L"VillageQuestsInHub"},
    {HUB_HP, L"Village Quests", L"HubHP"},
    {BONUS_CONTENT, L"Bonus Content", L"BonusContent"},
    {HP_1, L"Monster HP", L"1Player"},
    {HP_2, L"Monster HP", L"2Players"},
    {HP_3, L"Monster HP", L"3Players"},
    {HP_4, L"Monster HP", L"4Players"},
};

HWND g_window;
HWND g_tabs;
HWND g_controls[QUESTS + 1]{};
HWND g_pages[5]{};
std::wstring g_ini_path;
std::wstring g_game_path;
Settings g_settings;
std::vector<std::string> g_ini_lines;
bool g_ini_read_error;

std::wstring utf8_to_wide(const std::string &s) {
    if (s.empty()) return {};
    int n = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, s.c_str(), -1, nullptr, 0);
    UINT code_page = CP_UTF8;
    if (!n) {
        code_page = CP_ACP;
        n = MultiByteToWideChar(code_page, 0, s.c_str(), -1, nullptr, 0);
    }
    if (!n) return {};
    std::wstring out(static_cast<size_t>(n), L'\0');
    MultiByteToWideChar(code_page, 0, s.c_str(), -1, &out[0], n);
    out.resize(static_cast<size_t>(n - 1));
    return out;
}

std::string wide_to_utf8(const std::wstring &s) {
    if (s.empty()) return {};
    int n = WideCharToMultiByte(CP_UTF8, 0, s.c_str(), -1, nullptr, 0, nullptr, nullptr);
    std::string out(static_cast<size_t>(n), '\0');
    WideCharToMultiByte(CP_UTF8, 0, s.c_str(), -1, &out[0], n, nullptr, nullptr);
    out.resize(static_cast<size_t>(n - 1));
    return out;
}

std::wstring module_dir() {
    wchar_t path[32768];
    DWORD n = GetModuleFileNameW(nullptr, path, static_cast<DWORD>(sizeof(path) / sizeof(path[0])));
    if (!n || n >= sizeof(path) / sizeof(path[0])) return L".";
    std::wstring result(path, n);
    size_t slash = result.find_last_of(L"\\/");
    if (slash != std::wstring::npos) result.resize(slash);
    return result;
}

std::string trim_ascii(std::string s) {
    const char *ws = " \t\r\n";
    size_t first = s.find_first_not_of(ws);
    if (first == std::string::npos) return {};
    size_t last = s.find_last_not_of(ws);
    return s.substr(first, last - first + 1);
}

std::wstring squash(std::wstring s) {
    std::wstring out;
    for (wchar_t c : s)
        if (c != L' ' && c != L'\t' && c != L'_' && c != L'-' && c != L'\r' && c != L'\n')
            out.push_back(static_cast<wchar_t>(towlower(c)));
    return out;
}

bool read_file(const std::wstring &path, std::string &content) {
    HANDLE file = CreateFileW(path.c_str(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING,
                              FILE_ATTRIBUTE_NORMAL, nullptr);
    if (file == INVALID_HANDLE_VALUE) {
        DWORD error = GetLastError();
        g_ini_read_error = error != ERROR_FILE_NOT_FOUND && error != ERROR_PATH_NOT_FOUND;
        return false;
    }
    LARGE_INTEGER size;
    bool ok = GetFileSizeEx(file, &size) && size.QuadPart >= 0 && size.QuadPart <= 4 * 1024 * 1024;
    if (ok) {
        content.resize(static_cast<size_t>(size.QuadPart));
        DWORD read = 0;
        ok = content.empty() || (ReadFile(file, &content[0], static_cast<DWORD>(content.size()), &read, nullptr) &&
                                 read == content.size());
    }
    CloseHandle(file);
    if (!ok) g_ini_read_error = true;
    if (ok && content.size() >= 3 && static_cast<unsigned char>(content[0]) == 0xef &&
        static_cast<unsigned char>(content[1]) == 0xbb && static_cast<unsigned char>(content[2]) == 0xbf)
        content.erase(0, 3);
    return ok;
}

bool write_file(const std::wstring &path, const std::string &content) {
    HANDLE file = CreateFileW(path.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (file == INVALID_HANDLE_VALUE) return false;
    DWORD written = 0;
    bool ok = content.size() <= UINT32_MAX &&
              (content.empty() || (WriteFile(file, content.data(), static_cast<DWORD>(content.size()), &written, nullptr) &&
                                   written == content.size()));
    if (!CloseHandle(file)) ok = false;
    return ok;
}

std::vector<std::string> split_lines(const std::string &text) {
    std::vector<std::string> lines;
    size_t start = 0;
    while (start < text.size()) {
        size_t end = text.find('\n', start);
        if (end == std::string::npos) {
            lines.push_back(text.substr(start));
            break;
        }
        size_t length = end - start;
        if (length && text[start + length - 1] == '\r') --length;
        lines.push_back(text.substr(start, length));
        start = end + 1;
    }
    return lines;
}

std::string value_part(std::string line) {
    size_t comment = line.find_first_of(";#");
    if (comment != std::string::npos) line.resize(comment);
    size_t eq = line.find('=');
    return eq == std::string::npos ? std::string{} : trim_ascii(line.substr(eq + 1));
}

std::map<std::wstring, std::wstring> parse_ini(const std::vector<std::string> &lines) {
    std::map<std::wstring, std::wstring> values;
    std::wstring section;
    for (const std::string &line : lines) {
        std::string t = trim_ascii(line);
        if (t.empty() || t[0] == ';' || t[0] == '#') continue;
        if (t[0] == '[') {
            size_t close = t.find(']');
            if (close != std::string::npos) section = squash(utf8_to_wide(t.substr(1, close - 1)));
            continue;
        }
        size_t eq = t.find('=');
        if (eq == std::string::npos) continue;
        std::wstring key = squash(utf8_to_wide(trim_ascii(t.substr(0, eq))));
        values[section + L"|" + key] = utf8_to_wide(value_part(t));
    }
    return values;
}

std::wstring lookup(const std::map<std::wstring, std::wstring> &values, const wchar_t *section, const wchar_t *key,
                    const std::wstring &fallback) {
    auto it = values.find(squash(section) + L"|" + squash(key));
    return it == values.end() ? fallback : it->second;
}

Settings load_settings() {
    Settings s;
    std::string raw;
    if (!read_file(g_ini_path, raw)) return s;
    g_ini_lines = split_lines(raw);
    auto values = parse_ini(g_ini_lines);
    for (const Field &f : fields) {
        std::wstring fallback;
        switch (f.id) {
        case MODE: fallback = s.mode; break;
        case RESOLUTION: fallback = s.resolution; break;
        case RENDER_RESOLUTION: fallback = s.render_resolution; break;
        case MSAA: fallback = s.msaa; break;
        case FXAA: fallback = s.fxaa; break;
        case FPS: fallback = s.fps; break;
        case VSYNC: fallback = s.vsync; break;
        case FACE_BUTTONS: fallback = s.face_buttons; break;
        case MOUSE_SENSITIVITY: fallback = s.mouse_sensitivity; break;
        case PORT: fallback = s.port; break;
        case JOIN: fallback = s.join; break;
        case SYNC_SMALL_MONSTERS: fallback = s.sync_small_monsters; break;
        case VILLAGE_IN_HUB: fallback = s.village_in_hub; break;
        case HUB_HP: fallback = s.hub_hp; break;
        case BONUS_CONTENT: fallback = s.bonus_content; break;
        case HP_1: fallback = s.hp[0]; break;
        case HP_2: fallback = s.hp[1]; break;
        case HP_3: fallback = s.hp[2]; break;
        case HP_4: fallback = s.hp[3]; break;
        }
        std::wstring v = lookup(values, f.section, f.key, fallback);
        switch (f.id) {
        case MODE: s.mode = v; break;
        case RESOLUTION: s.resolution = v; break;
        case RENDER_RESOLUTION: s.render_resolution = v; break;
        case MSAA: s.msaa = v; break;
        case FXAA: s.fxaa = v; break;
        case FPS: s.fps = v; break;
        case VSYNC: s.vsync = v; break;
        case FACE_BUTTONS: s.face_buttons = v; break;
        case MOUSE_SENSITIVITY: s.mouse_sensitivity = v; break;
        case PORT: s.port = v; break;
        case JOIN: s.join = v; break;
        case SYNC_SMALL_MONSTERS: s.sync_small_monsters = v; break;
        case VILLAGE_IN_HUB: s.village_in_hub = v; break;
        case HUB_HP: s.hub_hp = v; break;
        case BONUS_CONTENT: s.bonus_content = v; break;
        case HP_1: s.hp[0] = v; break;
        case HP_2: s.hp[1] = v; break;
        case HP_3: s.hp[2] = v; break;
        case HP_4: s.hp[3] = v; break;
        }
    }
    std::wstringstream quests;
    for (const auto &entry : values) {
        const std::wstring prefix = L"monsterhp|quest";
        if (entry.first.compare(0, prefix.size(), prefix) != 0) continue;
        std::wstring key = entry.first.substr(prefix.size());
        if (key.empty() || key.find_first_not_of(L"0123456789") != std::wstring::npos) continue;
        quests << L"Quest" << key << L" = " << entry.second << L"\r\n";
    }
    s.quests = quests.str();
    return s;
}

std::string normalized_value_line(const std::string &line, const std::string &new_value) {
    size_t eq = line.find('=');
    size_t value_start = eq + 1;
    while (value_start < line.size() && (line[value_start] == ' ' || line[value_start] == '\t')) ++value_start;
    size_t value_end = line.find_first_of(";#", value_start);
    if (value_end == std::string::npos) {
        value_end = line.size();
        while (value_end > value_start && (line[value_end - 1] == ' ' || line[value_end - 1] == '\t')) --value_end;
    } else {
        while (value_end > value_start && (line[value_end - 1] == ' ' || line[value_end - 1] == '\t')) --value_end;
    }
    return line.substr(0, value_start) + new_value + line.substr(value_end);
}

bool parse_number(const std::wstring &text, double &out) {
    std::string s = wide_to_utf8(text);
    if (s.empty()) return false;
    if (s.back() == 'x' || s.back() == 'X') s.pop_back();
    char *end = nullptr;
    out = strtod(s.c_str(), &end);
    return end != s.c_str() && *end == '\0' && out == out;
}

bool validate_quests(const std::wstring &text, std::map<std::wstring, std::wstring> &quest_values) {
    std::wstring input = text;
    size_t pos = 0;
    while (pos <= input.size()) {
        size_t end = input.find_first_of(L"\r\n", pos);
        std::wstring line = input.substr(pos, end == std::wstring::npos ? end : end - pos);
        pos = end == std::wstring::npos ? input.size() + 1 : end + 1;
        size_t next = input.find_first_not_of(L"\r\n", pos);
        if (next == std::wstring::npos) pos = input.size() + 1;
        else pos = next;
        size_t first = line.find_first_not_of(L" \t");
        if (first == std::wstring::npos) continue;
        line = line.substr(first);
        size_t eq = line.find('=');
        if (eq == std::wstring::npos) return false;
        std::wstring key = squash(line.substr(0, eq));
        if (key.compare(0, 5, L"quest") != 0 || key.size() <= 5 ||
            key.substr(5).find_first_not_of(L"0123456789") != std::wstring::npos)
            return false;
        unsigned long long id = 0;
        for (size_t i = 5; i < key.size(); ++i) {
            id = id * 10 + static_cast<unsigned>(key[i] - L'0');
            if (id > UINT32_MAX) return false;
        }
        if (!id) return false;
        key = L"quest" + std::to_wstring(id);
        if (quest_values.count(key)) return false;
        std::wstring v = line.substr(eq + 1);
        std::vector<double> factors;
        size_t start = 0;
        while (start < v.size()) {
            size_t comma = v.find(',', start);
            std::wstring part = v.substr(start, comma == std::wstring::npos ? comma : comma - start);
            size_t a = part.find_first_not_of(L" \t");
            size_t b = part.find_last_not_of(L" \t");
            if (a == std::wstring::npos) return false;
            double factor;
            if (!parse_number(part.substr(a, b - a + 1), factor) || factor < 0.05 || factor > 20.0) return false;
            factors.push_back(factor);
            if (comma == std::wstring::npos) break;
            start = comma + 1;
        }
        if (factors.size() != 1 && factors.size() != 4) return false;
        quest_values[key] = wide_to_utf8(v);
    }
    return quest_values.size() <= 256;
}

std::wstring get_control_text(int id) {
    HWND control = g_controls[id];
    int n = GetWindowTextLengthW(control);
    std::wstring text(static_cast<size_t>(n + 1), L'\0');
    GetWindowTextW(control, &text[0], n + 1);
    text.resize(static_cast<size_t>(n));
    return text;
}

std::wstring get_field_value(int id, const Settings &s) {
    switch (id) {
    case MODE: return s.mode;
    case RESOLUTION: return s.resolution;
    case RENDER_RESOLUTION: return s.render_resolution;
    case MSAA: return s.msaa;
    case FXAA: return s.fxaa;
    case FPS: return s.fps;
    case VSYNC: return s.vsync;
    case FACE_BUTTONS: return s.face_buttons;
    case MOUSE_SENSITIVITY: return s.mouse_sensitivity;
    case PORT: return s.port;
    case JOIN: return s.join;
    case SYNC_SMALL_MONSTERS: return s.sync_small_monsters;
    case VILLAGE_IN_HUB: return s.village_in_hub;
    case HUB_HP: return s.hub_hp;
    case BONUS_CONTENT: return s.bonus_content;
    case HP_1: return s.hp[0];
    case HP_2: return s.hp[1];
    case HP_3: return s.hp[2];
    case HP_4: return s.hp[3];
    default: return {};
    }
}

void set_control_text(int id, const std::wstring &text) {
    SetWindowTextW(g_controls[id], text.c_str());
}

void fill_controls(const Settings &s) {
    for (const Field &f : fields) set_control_text(f.id, get_field_value(f.id, s));
    set_control_text(QUESTS, s.quests);
}

bool validate_and_collect(Settings &s, std::map<std::wstring, std::wstring> &quest_values) {
    for (const Field &f : fields) {
        std::wstring value = get_control_text(f.id);
        switch (f.id) {
        case MODE:
            if (value != L"Borderless" && value != L"Fullscreen" && value != L"Windowed") return false;
            break;
        case RESOLUTION:
        case RENDER_RESOLUTION: {
            if (f.id == RENDER_RESOLUTION && value == L"Auto") break;
            if (f.id == RESOLUTION && (value == L"Desktop" || value == L"Auto")) break;
            int w = 0, h = 0;
            if (swscanf(value.c_str(), L"%dx%d", &w, &h) != 2 || w < 320 || h < 180 || w > 16384 || h > 16384)
                return false;
            break;
        }
        case MSAA:
            if (value != L"Off" && value != L"2x" && value != L"4x" && value != L"8x") return false;
            break;
        case FXAA:
        case VSYNC:
        case SYNC_SMALL_MONSTERS:
        case VILLAGE_IN_HUB:
        case BONUS_CONTENT:
            if (value != L"On" && value != L"Off") return false;
            break;
        case FACE_BUTTONS:
            if (value != L"Position" && value != L"Label") return false;
            break;
        case FPS:
        case PORT: {
            wchar_t *end = nullptr;
            long number = wcstol(value.c_str(), &end, 10);
            if (!end || *end || (f.id == FPS ? number < 30 || number > 240 : number < 1 || number > 65535))
                return false;
            break;
        }
        case MOUSE_SENSITIVITY:
        case HUB_HP:
        case HP_1:
        case HP_2:
        case HP_3:
        case HP_4: {
            double number;
            if (!parse_number(value, number) ||
                (f.id == MOUSE_SENSITIVITY ? number < 0.1 || number > 10.0 : number < 0.05 || number > 20.0))
                return false;
            break;
        }
        case JOIN:
            if (wide_to_utf8(value).size() >= 256 || value.find_first_of(L"\r\n") != std::wstring::npos) return false;
            break;
        }
        switch (f.id) {
        case MODE: s.mode = value; break;
        case RESOLUTION: s.resolution = value; break;
        case RENDER_RESOLUTION: s.render_resolution = value; break;
        case MSAA: s.msaa = value; break;
        case FXAA: s.fxaa = value; break;
        case FPS: s.fps = value; break;
        case VSYNC: s.vsync = value; break;
        case FACE_BUTTONS: s.face_buttons = value; break;
        case MOUSE_SENSITIVITY: s.mouse_sensitivity = value; break;
        case PORT: s.port = value; break;
        case JOIN: s.join = value; break;
        case SYNC_SMALL_MONSTERS: s.sync_small_monsters = value; break;
        case VILLAGE_IN_HUB: s.village_in_hub = value; break;
        case HUB_HP: s.hub_hp = value; break;
        case BONUS_CONTENT: s.bonus_content = value; break;
        case HP_1: s.hp[0] = value; break;
        case HP_2: s.hp[1] = value; break;
        case HP_3: s.hp[2] = value; break;
        case HP_4: s.hp[3] = value; break;
        }
    }
    return validate_quests(get_control_text(QUESTS), quest_values);
}

bool save_settings(const Settings &s, const std::map<std::wstring, std::wstring> &quest_values) {
    std::map<std::wstring, std::string> updates;
    for (const Field &f : fields)
        updates[squash(f.section) + L"|" + squash(f.key)] = wide_to_utf8(get_field_value(f.id, s));
    for (const auto &q : quest_values) updates[squash(L"Monster HP") + L"|" + q.first] = q.second;

    std::vector<std::string> output;
    std::map<std::wstring, bool> handled;
    std::wstring section;
    for (const std::string &line : g_ini_lines) {
        std::string t = trim_ascii(line);
        if (!t.empty() && t[0] == '[') {
            size_t close = t.find(']');
            if (close != std::string::npos) section = squash(utf8_to_wide(t.substr(1, close - 1)));
        }
        size_t eq = t.find('=');
        if (eq != std::string::npos && (t.empty() || (t[0] != ';' && t[0] != '#'))) {
            std::wstring key = section + L"|" + squash(utf8_to_wide(trim_ascii(t.substr(0, eq))));
            const std::wstring quest_prefix = squash(L"Monster HP") + L"|quest";
            if (key.compare(0, quest_prefix.size(), quest_prefix) == 0 &&
                key.substr(quest_prefix.size()).find_first_not_of(L"0123456789") == std::wstring::npos &&
                quest_values.find(key.substr(key.find(L'|') + 1)) == quest_values.end())
                continue;
            auto update = updates.find(key);
            if (update != updates.end()) {
                output.push_back(normalized_value_line(line, update->second));
                handled[key] = true;
                continue;
            }
        }
        output.push_back(line);
    }

    std::map<std::wstring, std::vector<std::pair<std::wstring, std::string>>> missing;
    for (const auto &update : updates)
        if (!handled[update.first]) {
            size_t sep = update.first.find(L'|');
            missing[update.first.substr(0, sep)].push_back({update.first.substr(sep + 1), update.second});
        }
    const wchar_t *sections[] = {L"display", L"antialiasing", L"performance", L"controls", L"mouse", L"multiplayer",
                                 L"monsterhp", L"villagequests", L"bonuscontent"};
    std::map<std::wstring, std::wstring> display_names = {
        {L"display", L"Display"}, {L"antialiasing", L"Anti-aliasing"}, {L"performance", L"Performance"},
        {L"controls", L"Controls"}, {L"mouse", L"Mouse"}, {L"multiplayer", L"Multiplayer"},
        {L"monsterhp", L"Monster HP"}, {L"villagequests", L"Village Quests"}, {L"bonuscontent", L"Bonus Content"}};
    for (const wchar_t *name : sections) {
        auto it = missing.find(name);
        if (it == missing.end()) continue;
        if (!output.empty() && !output.back().empty()) output.emplace_back();
        output.push_back("[" + wide_to_utf8(display_names[name]) + "]");
        for (const auto &entry : it->second)
            output.push_back(wide_to_utf8(entry.first) + " = " + entry.second);
    }
    std::string content;
    for (const std::string &line : output) content += line + "\r\n";
    if (!write_file(g_ini_path, content)) return false;
    g_ini_lines = std::move(output);
    return true;
}

HWND make_control(HWND parent, const wchar_t *klass, const wchar_t *text, DWORD style, int id, int x, int y,
                  int width, int height) {
    return CreateWindowExW(0, klass, text, WS_CHILD | WS_VISIBLE | style, x, y, width, height, parent,
                           reinterpret_cast<HMENU>(static_cast<INT_PTR>(id)), GetModuleHandleW(nullptr), nullptr);
}

void label(HWND page, const wchar_t *text, int x = 40, int y = 76, int width = 250) {
    make_control(page, L"STATIC", text, SS_LEFT, 0, x, y + 5, width, 24);
}

HWND combo(HWND page, int id, const wchar_t *const *items, int count, int y, bool editable = true) {
    DWORD style = editable ? CBS_DROPDOWN | WS_VSCROLL : CBS_DROPDOWNLIST | WS_VSCROLL;
    HWND control = make_control(page, L"COMBOBOX", L"", style | WS_TABSTOP, id, 320, y, 400, 240);
    for (int i = 0; i < count; ++i) SendMessageW(control, CB_ADDSTRING, 0, reinterpret_cast<LPARAM>(items[i]));
    g_controls[id] = control;
    return control;
}

HWND edit(HWND page, int id, int y, int width = 400, bool multiline = false, int x = 320) {
    DWORD style = ES_AUTOHSCROLL | WS_TABSTOP;
    if (multiline) style = ES_MULTILINE | ES_AUTOVSCROLL | WS_VSCROLL | WS_TABSTOP;
    HWND control = make_control(page, L"EDIT", L"", WS_BORDER | ES_LEFT | style, id, x, y, width,
                                multiline ? 210 : 28);
    g_controls[id] = control;
    return control;
}

void add_tab_page(HWND window, int index, int x, int y, int width, int height) {
    g_pages[index] = CreateWindowExW(0, L"STATIC", L"", WS_CHILD | WS_VISIBLE, x, y, width, height, window, nullptr,
                                     GetModuleHandleW(nullptr), nullptr);
}

void create_pages(HWND window) {
    RECT r{};
    GetClientRect(g_tabs, &r);
    TabCtrl_AdjustRect(g_tabs, FALSE, &r);
    POINT p{r.left, r.top};
    MapWindowPoints(g_tabs, window, &p, 1);
    for (int i = 0; i < 5; ++i) add_tab_page(window, i, p.x, p.y, r.right - r.left, r.bottom - r.top);

    const wchar_t *modes[] = {L"Borderless", L"Fullscreen", L"Windowed"};
    label(g_pages[0], L"Ablak mód"); combo(g_pages[0], MODE, modes, 3, 76);
    label(g_pages[0], L"Felbontás (pl. 1920x1080 vagy Desktop)", 40, 120);
    const wchar_t *resolutions[] = {L"Desktop", L"1280x720", L"1600x900", L"1920x1080", L"2560x1440", L"3840x2160"};
    combo(g_pages[0], RESOLUTION, resolutions, 6, 120, true);
    label(g_pages[0], L"3D renderfelbontás (Auto vagy WxH)", 40, 164);
    const wchar_t *renders[] = {L"Auto", L"1280x720", L"1920x1080", L"2560x1440", L"3840x2160"};
    combo(g_pages[0], RENDER_RESOLUTION, renders, 5, 164, true);
    label(g_pages[0], L"Az Auto a képernyőhöz illeszkedő 16:9 felbontást választja.", 40, 214, 680);

    const wchar_t *msaa[] = {L"Off", L"2x", L"4x", L"8x"};
    const wchar_t *on_off[] = {L"On", L"Off"};
    label(g_pages[1], L"MSAA"); combo(g_pages[1], MSAA, msaa, 4, 76);
    label(g_pages[1], L"FXAA", 40, 120); combo(g_pages[1], FXAA, on_off, 2, 120);
    label(g_pages[1], L"FPS-korlát (30–240)", 40, 164); edit(g_pages[1], FPS, 164, 120);
    label(g_pages[1], L"VSync", 40, 208); combo(g_pages[1], VSYNC, on_off, 2, 208);

    const wchar_t *buttons[] = {L"Position", L"Label"};
    label(g_pages[2], L"Kontroller gombkiosztás"); combo(g_pages[2], FACE_BUTTONS, buttons, 2, 76);
    label(g_pages[2], L"Egér érzékenysége (0.1–10)", 40, 120); edit(g_pages[2], MOUSE_SENSITIVITY, 120, 160);
    label(g_pages[2], L"Position: Switch-elrendezés; Label: a gombokra nyomtatott betűk.", 40, 172, 680);

    label(g_pages[3], L"Multiplayer port (1–65535)"); edit(g_pages[3], PORT, 76, 160);
    label(g_pages[3], L"Távoli hostok (vesszővel elválasztva)", 40, 120); edit(g_pages[3], JOIN, 120);
    label(g_pages[3], L"Kis szörnyek szinkronizálása", 40, 164); combo(g_pages[3], SYNC_SMALL_MONSTERS, on_off, 2, 164);

    label(g_pages[4], L"HP-szorzók a Hub küldetésekben (1–4 vadász)");
    label(g_pages[4], L"1 vadász", 40, 120, 120); edit(g_pages[4], HP_1, 145, 110, false, 40);
    label(g_pages[4], L"2 vadász", 185, 120, 120); edit(g_pages[4], HP_2, 145, 110, false, 185);
    label(g_pages[4], L"3 vadász", 330, 120, 120); edit(g_pages[4], HP_3, 145, 110, false, 330);
    label(g_pages[4], L"4 vadász", 475, 120, 120); edit(g_pages[4], HP_4, 145, 110, false, 475);
    label(g_pages[4], L"Falusi küldetések a Hubban", 40, 164); combo(g_pages[4], VILLAGE_IN_HUB, on_off, 2, 164);
    label(g_pages[4], L"Falusi küldetések HP-szorzója a Hubban", 40, 208); edit(g_pages[4], HUB_HP, 208, 160);
    label(g_pages[4], L"Bónusztartalom átvétele mentésből", 40, 252); combo(g_pages[4], BONUS_CONTENT, on_off, 2, 252);
    label(g_pages[4], L"Egyedi küldetés HP (Quest123 = 1.0 vagy Quest123 = 1, 1, 1, 1)", 40, 296, 680);
    edit(g_pages[4], QUESTS, 336, 680, true);
}

void show_page(int selected) {
    for (int i = 0; i < 5; ++i) ShowWindow(g_pages[i], i == selected ? SW_SHOW : SW_HIDE);
}

void show_error(const wchar_t *message) {
    MessageBoxW(g_window, message, L"MHGU Recomp Launcher", MB_OK | MB_ICONERROR);
}

bool apply_form(Settings &s, std::map<std::wstring, std::wstring> &quests) {
    if (!validate_and_collect(s, quests)) {
        show_error(L"Ellenőrizd a beállításokat. A mezők értékei és az egyedi Quest sorok formátuma érvénytelen.");
        return false;
    }
    if (!save_settings(s, quests)) {
        show_error(L"Nem sikerült elmenteni a mhgurecomp.ini fájlt. Ellenőrizd a mappa írási jogosultságát.");
        return false;
    }
    return true;
}

void launch_game() {
    SHELLEXECUTEINFOW info{};
    info.cbSize = sizeof(info);
    info.fMask = SEE_MASK_NOCLOSEPROCESS;
    info.lpFile = g_game_path.c_str();
    std::wstring working_dir = module_dir();
    info.lpDirectory = working_dir.c_str();
    info.nShow = SW_SHOWNORMAL;
    if (!ShellExecuteExW(&info)) {
        show_error(L"Nem sikerült elindítani a mhgurecomp.exe fájlt.");
        return;
    }
    if (info.hProcess) CloseHandle(info.hProcess);
    DestroyWindow(g_window);
}

LRESULT CALLBACK window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    switch (message) {
    case WM_CREATE: {
        INITCOMMONCONTROLSEX common{sizeof(common), ICC_TAB_CLASSES};
        InitCommonControlsEx(&common);
        g_tabs = CreateWindowExW(0, WC_TABCONTROLW, L"", WS_CHILD | WS_VISIBLE | WS_TABSTOP | TCS_FOCUSNEVER, 12, 12,
                                 852, 610, window, reinterpret_cast<HMENU>(TAB_CONTROL), GetModuleHandleW(nullptr), nullptr);
        const wchar_t *names[] = {L"Kép", L"Grafika", L"Irányítás", L"Multiplayer", L"Játékmenet"};
        for (int i = 0; i < 5; ++i) {
            TCITEMW item{};
            item.mask = TCIF_TEXT;
            item.pszText = const_cast<wchar_t *>(names[i]);
            TabCtrl_InsertItem(g_tabs, i, &item);
        }
        create_pages(window);
        show_page(0);
        fill_controls(g_settings);
        make_control(window, L"BUTTON", L"Mentés", BS_PUSHBUTTON | WS_TABSTOP, SAVE, 18, 638, 140, 34);
        make_control(window, L"BUTTON", L"Mentés és indítás", BS_DEFPUSHBUTTON | WS_TABSTOP, SAVE_AND_LAUNCH, 168, 638,
                     170, 34);
        make_control(window, L"BUTTON", L"Alapértékek", BS_PUSHBUTTON | WS_TABSTOP, RESET, 348, 638, 140, 34);
        make_control(window, L"BUTTON", L"INI megnyitása", BS_PUSHBUTTON | WS_TABSTOP, OPEN_INI, 708, 638, 145, 34);
        return 0;
    }
    case WM_NOTIFY:
        if (reinterpret_cast<NMHDR *>(lparam)->idFrom == TAB_CONTROL &&
            reinterpret_cast<NMHDR *>(lparam)->code == TCN_SELCHANGE)
            show_page(TabCtrl_GetCurSel(g_tabs));
        return 0;
    case WM_COMMAND:
        switch (LOWORD(wparam)) {
        case SAVE:
        case SAVE_AND_LAUNCH: {
            Settings changed = g_settings;
            std::map<std::wstring, std::wstring> quests;
            if (!apply_form(changed, quests)) return 0;
            g_settings = changed;
            if (LOWORD(wparam) == SAVE_AND_LAUNCH) launch_game();
            else MessageBoxW(window, L"A beállítások el lettek mentve.", L"MHGU Recomp Launcher",
                             MB_OK | MB_ICONINFORMATION);
            return 0;
        }
        case RESET:
            if (MessageBoxW(window, L"Visszaállítod az összes beállítást az alapértékekre?", L"MHGU Recomp Launcher",
                            MB_YESNO | MB_ICONQUESTION) == IDYES) {
                g_settings = Settings{};
                fill_controls(g_settings);
            }
            return 0;
        case OPEN_INI:
        {
            std::wstring working_dir = module_dir();
            HINSTANCE result = ShellExecuteW(window, L"open", g_ini_path.c_str(), nullptr, working_dir.c_str(),
                                             SW_SHOWNORMAL);
            if (reinterpret_cast<INT_PTR>(result) <= 32) show_error(L"Nem sikerült megnyitni a mhgurecomp.ini fájlt.");
            return 0;
        }
        }
        return 0;
    case WM_DESTROY:
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProcW(window, message, wparam, lparam);
}

} // namespace

int WINAPI WinMain(HINSTANCE instance, HINSTANCE, LPSTR, int show) {
    std::wstring directory = module_dir();
    g_ini_path = directory + L"\\mhgurecomp.ini";
    g_game_path = directory + L"\\mhgurecomp.exe";
    g_settings = load_settings();
    INITCOMMONCONTROLSEX common{sizeof(common), ICC_STANDARD_CLASSES | ICC_TAB_CLASSES};
    InitCommonControlsEx(&common);
    WNDCLASSW wc{};
    wc.lpfnWndProc = window_proc;
    wc.hInstance = instance;
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.hIcon = LoadIconW(nullptr, IDI_APPLICATION);
    wc.hbrBackground = reinterpret_cast<HBRUSH>(COLOR_BTNFACE + 1);
    wc.lpszClassName = L"MHGURecompLauncherWindow";
    if (!RegisterClassW(&wc)) {
        MessageBoxW(nullptr, L"Nem sikerült létrehozni az indítóablakot.", L"MHGU Recomp Launcher",
                    MB_OK | MB_ICONERROR);
        return 1;
    }
    g_window = CreateWindowExW(0, wc.lpszClassName, L"MHGU Recomp Launcher", WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU |
                               WS_MINIMIZEBOX, CW_USEDEFAULT, CW_USEDEFAULT, 892, 722, nullptr, nullptr, instance, nullptr);
    if (!g_window) return 1;
    if (g_ini_read_error) {
        show_error(L"A mhgurecomp.ini fájlt nem sikerült beolvasni. A launcher nem menti rá az alapértékeket.");
        DestroyWindow(g_window);
        return 1;
    }
    ShowWindow(g_window, show);
    UpdateWindow(g_window);
    MSG message;
    while (GetMessageW(&message, nullptr, 0, 0) > 0) {
        TranslateMessage(&message);
        DispatchMessageW(&message);
    }
    return static_cast<int>(message.wParam);
}
