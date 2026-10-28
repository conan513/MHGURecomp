// Player settings (config.h): mhgurecomp.ini in the game folder (next to the executable, or MHGU_HOME; with a game
// folder argument, the current folder). A missing file is written with the defaults, so
// deleting it resets everything; keys that are missing or not understood keep their defaults (the log says which).
//   MHGU_CONFIG=<path>   another settings file
//   MHGU_WINDOW=WxH      windowed at that size, MHGU_RENDER=WxH  3D resolution, MHGU_MSAA=1|2|4|8, MHGU_FXAA=0|1,
//                        MHGU_SSAO=0|1,
//                        MHGU_FSR=0|1, MHGU_FSR_QUALITY=<name>, MHGU_FSR_SHARPNESS=<0..1>,
//                        MHGU_FSR_REACTIVE=<0..1>, MHGU_FSR_OBJMOTION=0|1, MHGU_FSR_ANIMSURF=0|1,
//                        MHGU_PAD_POSITIONAL=0|1 (test runs; override the file)
#include <ctype.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>

#include <SDL3/SDL.h>

#include "config.h"
#include "hle/input.h"
#include "rt.h"

Config g_config = {
    .display_mode = DISPLAY_BORDERLESS,
    .width = 1920,
    .height = 1080,
    .fps = 60,
    .vsync = 1,
    .msaa = 1,
    .fxaa = 1,
    .fsr_quality = FSR_NATIVE,
    .fsr_sharpness = 0.8f,
    .fsr_objmotion = 1,
    .fsr_reactive = 0.9f,
    .fsr_animsurf = 1,
    .face_buttons = FACE_POSITION,
    .controller_rumble = 1,
    .ldn_port = 39500,
    .sync_small_monsters = 1,
    .bonus_content = 1,
    .village_in_hub = 1,
    .hp = {{1, 1, 1, 1}, 2},
    .mouse_sensitivity = 1.0f,
    .camera_invert_horizontal = 0,
    .camera_invert_vertical = 0,
    .camera_smoothing = 0.5f,
    .mouse_buttons = {BTN_X, -1, BTN_A, -1, -1},
    .keyboard = {
        SDL_SCANCODE_1, SDL_SCANCODE_UP, SDL_SCANCODE_2, SDL_SCANCODE_RIGHT,
        SDL_SCANCODE_3, SDL_SCANCODE_DOWN, SDL_SCANCODE_4, SDL_SCANCODE_LEFT,
        SDL_SCANCODE_SPACE, SDL_SCANCODE_BACKSPACE, SDL_SCANCODE_E, SDL_SCANCODE_Q,
        SDL_SCANCODE_LSHIFT, SDL_SCANCODE_TAB, SDL_SCANCODE_R, SDL_SCANCODE_KP_ENTER,
        SDL_SCANCODE_ESCAPE, SDL_SCANCODE_UNKNOWN, SDL_SCANCODE_C, SDL_SCANCODE_V,
        SDL_SCANCODE_LCTRL, SDL_SCANCODE_F, SDL_SCANCODE_RETURN, SDL_SCANCODE_LALT,
        SDL_SCANCODE_W, SDL_SCANCODE_A,
        SDL_SCANCODE_S, SDL_SCANCODE_D,
        SDL_SCANCODE_G, SDL_SCANCODE_H, SDL_SCANCODE_T, SDL_SCANCODE_U,
        SDL_SCANCODE_RSHIFT, SDL_SCANCODE_Z, SDL_SCANCODE_B, SDL_SCANCODE_LALT, SDL_SCANCODE_RCTRL,
    },
    .aspect_ratio = 0.0f,
    .fov = 1.0f,
    .ui_anchor = UI_CENTER,
    .background_input = 1,
};
uint32_t g_render_size[2] = {1920, 1080};
float g_render_aspect = 16.0f / 9.0f;
float g_fov_scale = 1.0f;
uint32_t g_ui_size[2] = {1280, 720};
float g_ui_anchor_aspect = 16.0f / 9.0f;
uint32_t g_fsr_output[2];
float g_fsr_lod_bias;

static const char DEFAULT_INI[] =
    "; MHGU Recomp settings. Delete this file to get the defaults back: it is written again at the next start.\n"
    "\n"
    "[Display]\n"
    "; Fullscreen, Windowed or Borderless (a borderless window covering the whole screen).\n"
    "; Alt+Enter or F11 switches between Windowed and the full-screen mode chosen here.\n"
    "Mode = Borderless\n"
    "\n"
    "; Screen size: WxH (1280x720, 1600x900, 1920x1080, 2560x1440...) or Desktop (the monitor's own resolution).\n"
    "; Fullscreen switches the monitor to this mode, Windowed opens a window this size (it can be resized), and\n"
    "; Borderless always covers the screen, scaling the game's image to it.\n"
    "Resolution = 1920x1080\n"
    "\n"
    "; The game's 3D resolution: Auto (as large as fits Resolution, in the picture's shape below) or WxH. Lower is\n"
    "; faster, higher is sharper; the Switch renders 1920x1080 docked. Above Resolution it supersamples (smoothest at\n"
    "; exactly double, e.g. 3840x2160 for 1920x1080).\n"
    "RenderResolution = Auto\n"
    "\n"
    "; The picture's shape: Auto (the window's) or a ratio such as 21:9. Wider than 16:9 (ultrawide screens) shows\n"
    "; more to the sides, 16:10 a little more above and below. Narrower than 16:10 (4:3) gets 16:10 with bars.\n"
    "AspectRatio = Auto\n"
    "\n"
    "; Field of view: 1.0 is the game's, above 1.0 (1.1, 1.2) shows more, below 1.0 less. 0.5 to 2.\n"
    "FOV = 1.0\n"
    "\n"
    "; Where the HUD goes on a picture wider than 16:9: Center (in the middle 16:9, as the game lays it out) or Edges\n"
    "; (out to the sides of the screen). 21:9 or 32:9 spread it that wide, at most to the sides.\n"
    "UIAnchor = Center\n"
    "\n"
    "[Anti-aliasing]\n"
    "; MSAA: Off, 2x, 4x or 8x. Smooths the edges of 3D geometry and of some cut-out leaves and fences; each step\n"
    "; costs GPU time and video memory, a lot on older cards (an R9 270X at 1080p: 56 fps off, 35 at 2x, 28 at 4x).\n"
    "; F7 cycles through them while playing, to compare. (The Switch version has none.)\n"
    "MSAA = Off\n"
    "\n"
    "; FXAA: On or Off. The game's own edge smoothing: a filter over the finished image, slightly blurry.\n"
    "FXAA = On\n"
    "\n"
    "; SSAO: On or Off. Adds depth-aware screen-space contact shadows, including around skybox edges; may reduce\n"
    "; performance. It needs scene depth, so flat 2D loading screens are unaffected.\n"
    "SSAO = Off\n"
    "\n"
    "[Upscaler]\n"
    "; FSR: On or Off. AMD FidelityFX Super Resolution 3.1 temporal upscaling. It renders the 3D picture at a lower\n"
    "; resolution and reconstructs it at RenderResolution (or the display resolution with Auto). The HUD stays full\n"
    "; resolution. MSAA and FXAA are disabled while FSR is active.\n"
    "FSR = Off\n"
    "\n"
    "; FSRQuality: NativeAA, Quality, Balanced, Performance or UltraPerformance.\n"
    "FSRQuality = NativeAA\n"
    "\n"
    "; Sharpness: FSR sharpening, 0 (off) to 1 (strongest).\n"
    "Sharpness = 0.8\n"
    "\n"
    "; ObjectMotion: On tracks characters and monsters; Off uses camera motion only.\n"
    "ObjectMotion = On\n"
    "\n"
    "; Reactive: strength of the moving-object mask, from 0 (off) to 1.\n"
    "Reactive = 0.9\n"
    "\n"
    "; AnimSurf: On detects surfaces animated through their textures to reduce ghosting.\n"
    "AnimSurf = On\n"
    "\n"
    "[Performance]\n"
    "; Frame rate cap: 30 (the Switch version's rate) to 240. Game speed stays the same at any rate.\n"
    "FPS = 60\n"
    "\n"
    "; VSync: On or Off. On: no tearing, but no more frames than the monitor shows (a 60 Hz monitor stays at 60 fps).\n"
    "; Off: frames are shown as soon as they are done, up to FPS (tearing possible).\n"
    "VSync = On\n"
    "\n"
    "[Controller]\n"
    "; Controller face buttons: Position uses the Switch layout; Label follows the letters printed on the buttons.\n"
    "; Position: the Switch layout the game is made for - the right button is A (confirm), the bottom one B, the left\n"
    ";   one Y, the top one X, whatever is printed on them (on an Xbox controller the in-game icons then don't match).\n"
    "; Label: by the letter printed on the button (an Xbox controller's A is A; the icons match, the layout is awkward).\n"
    "FaceButtons = Position\n"
    "\n"
    "; Rumble: On or Off. Sends the game's vibration to the connected controller.\n"
    "Rumble = On\n"
    "\n"
    "; The controller while the game's window is in the background (a wiki in front of it, or on another screen):\n"
    "; On (it keeps working) or Off. The keyboard and mouse only work while the window has the focus.\n"
    "BackgroundInput = On\n"
    "\n"
    "[Keyboard]\n"
    "; Keyboard keys for Switch buttons and stick directions. The launcher can capture a key when you click a binding.\n"
    "; D-pad defaults keep both the number keys and arrow keys; button Alt keys provide a second binding. Delete\n"
    "; clears a launcher binding; None disables it.\n"
    "; Enter confirms (A), Backspace goes back (B). Key names use SDL names (e.g. Left Shift, Keypad Enter).\n"
    "DpadUp = 1\n"
    "DpadUpAlt = Up\n"
    "DpadRight = 2\n"
    "DpadRightAlt = Right\n"
    "DpadDown = 3\n"
    "DpadDownAlt = Down\n"
    "DpadLeft = 4\n"
    "DpadLeftAlt = Left\n"
    "ButtonB = Space\n"
    "ButtonBAlt = Backspace\n"
    "ButtonX = G\n"
    "ButtonXAlt = H\n"
    "ButtonY = E\n"
    "ButtonYAlt = T\n"
    "ButtonL = Q\n"
    "ButtonLAlt = U\n"
    "ButtonR = Left Shift\n"
    "ButtonRAlt = Right Shift\n"
    "ButtonZL = Tab\n"
    "ButtonZLAlt = Z\n"
    "ButtonZR = R\n"
    "ButtonZRAlt = B\n"
    "ButtonPlus = Keypad Enter\n"
    "ButtonPlusAlt = Escape\n"
    "ButtonPlusAlt2 = None\n"
    "ButtonMinus = C\n"
    "ButtonMinusAlt = V\n"
    "ButtonLeftStick = Left Ctrl\n"
    "ButtonLeftStickAlt = Left Alt\n"
    "ButtonA = F\n"
    "ButtonAAlt = Return\n"
    "ButtonRightStick = Left Alt\n"
    "ButtonRightStickAlt = Right Ctrl\n"
    "StickUp = W\n"
    "StickLeft = A\n"
    "StickDown = S\n"
    "StickRight = D\n"
    "\n"
    "[Mouse]\n"
    "; Mouse look: click in the game window to capture the mouse (it then turns the camera, as the right stick), Esc\n"
    "; lets it go. Sensitivity: 1 = 0.06 degrees per mouse count; 0.5 slower, 2 faster (0.1 to 10).\n"
    "; The mouse turns the camera directly; the game's stick dead zone and camera speed do not apply.\n"
    "; InvertHorizontal / InvertVertical: reverse the corresponding mouse-look axis. On or Off.\n"
    "InvertHorizontal = Off\n"
    "InvertVertical = Off\n"
    "; Smoothing: 0.05 (smooth, slower response) to 1 (immediate response).\n"
    "Smoothing = 0.5\n"
    "; Mouse buttons can be mapped to any Switch button\n"
    "; or None. The first click used to capture the mouse is not sent to the game.\n"
    "MouseSensitivity = 1.0\n"
    "LeftButton = X\n"
    "MiddleButton = None\n"
    "RightButton = A\n"
    "X1Button = None\n"
    "X2Button = None\n"
    "\n"
    "[Multiplayer]\n"
    "; Local Play (the Switch's wireless play with people nearby) over the network. Create a hub under Local Play to\n"
    "; host: the others reach your computer on this port. Over the internet the host forwards it in their router,\n"
    "; TCP and UDP (TCP alone works too, but copes worse with a bad connection); nobody else needs to.\n"
    "Port = 39500\n"
    "\n"
    "; Search Hubs finds hubs on your LAN by itself. To reach a host elsewhere, put its address here: an IP or name,\n"
    "; with :port if it isn't the Port above. Several: separate them with commas. Example: Join = 203.0.113.7:39500\n"
    "Join =\n"
    "\n"
    "; Small monsters (Jaggi, Aptonoth...) synced between the players in a zone, like large monsters are: On or Off.\n"
    "; Off is the original game: each player sees them in different places. Everyone in the hunt should use the same.\n"
    "SyncSmallMonsters = On\n"
    "\n"
    "[Monster HP]\n"
    "; Large monster HP in Hub quests by the number of hunters on the quest. Each quest sets its monsters' HP as a\n"
    "; multiplier (most Hub quests are made for about two hunters); these factors multiply it: 0.6 = 60% of the\n"
    "; quest's HP, 1 = the original game. Only HP changes: attack, defense, size and the rest stay as they are.\n"
    "; Applies to every quest but the Village ones (played alone anyway): Hub and G Rank, Arena, Special Permit,\n"
    "; Event and Hub Prowler quests. In a hub, everyone uses the host's settings (each game computes the HP itself).\n"
    "1Player = 1.0\n"
    "2Players = 1.0\n"
    "3Players = 1.0\n"
    "4Players = 1.0\n"
    "\n"
    "; One quest set apart: Quest<number> = factors for 1, 2, 3 and 4 hunters (or one factor for all). The log\n"
    "; (log.txt) names the quest number whenever a monster's HP is set, e.g. \"[hpscale] quest 10107, ...\".\n"
    "; Example: Quest10107 = 1.0, 1.0, 1.2, 1.4\n"
    "\n"
    "[Village Quests]\n"
    "; Village quests at the Hub's and the Hunters' Pub's quest counter, to play them together: On or Off. In the\n"
    "; counter's rank menu or star menu, L or R switches between the Hub's quests and the Village quests (the same\n"
    "; list as at a village: what your village progress has unlocked).\n"
    "VillageQuestsInHub = On\n"
    "\n"
    "; Village quests are made for one hunter: posted in the Hub or Pub, their large monsters' HP is multiplied by this\n"
    "; first, then by the [Monster HP] factor for the number of hunters. 1 = as in the village.\n"
    "HubHP = 2.0\n"
    "\n"
    "[Bonus Content]\n"
    "; The free downloads (event and challenge quests, item packs, Palicos) can't be fetched from Capcom's server any\n"
    "; more. On: they are copied from a save that has them (a \"donor\" save file named system, in a folder called bonus\n"
    "; next to mhgurecomp.exe) into a save that has never had any downloads, such as a new one. Only the download data\n"
    "; is written, once; saves that already have downloads are left alone. Without a donor save nothing happens.\n"
    "; Off: saves are never changed.\n"
    "BonusContent = On\n";

static void ini_path(char *buf, size_t size) {
    const char *e = getenv("MHGU_CONFIG");
    if (e && *e) { snprintf(buf, size, "%s", e); return; }
    snprintf(buf, size, "mhgurecomp.ini");  // (started on its own the game runs in its folder, or MHGU_HOME: main.c)
}

// lower case, spaces and '_' / '-' removed: "Borderless Window" == "borderlesswindow"
static void squash(char *s) {
    char *d = s;
    for (; *s; s++) if (!isspace((unsigned char)*s) && *s != '_' && *s != '-') *d++ = (char)tolower((unsigned char)*s);
    *d = 0;
}

static bool parse_size(const char *v, int *w, int *h) {
    int a, b;
    char x;
    if (sscanf(v, "%d %c %d", &a, &x, &b) != 3 || (x != 'x' && x != 'X' && x != '*')) return false;
    if (a < 320 || b < 180 || a > 16384 || b > 16384) return false;
    *w = a, *h = b;
    return true;
}

// [Monster HP] factors: "0.6", "0.6x", or a comma separated list of up to four (squashed: no spaces). The count read,
// 0 when one is not a number in HP_SCALE_MIN..HP_SCALE_MAX.
static int parse_factors(const char *v, float f[4]) {
    int n = 0;
    while (*v && n < 4) {
        char *end;
        float x = strtof(v, &end);
        if (end == v || !(x >= HP_SCALE_MIN && x <= HP_SCALE_MAX)) return 0;
        f[n++] = x;
        v = end + (*end == 'x');
        if (*v == ',') v++;
        else if (*v) return 0;
    }
    return *v ? 0 : n;
}

// The vertical field of view (radians) a camera of this aspect gets (tools/gen.py PATCHES: the three callers of the
// perspective matrix that load a camera's). One made for the screen (its 16:9 aspect replaced with g_render_aspect)
// gets the FOV factor and, on a picture narrower than 16:9 (16:10), the height that keeps 16:9's width (Vert+): more
// above and below instead of less to the sides. Other cameras keep their own.
float camera_fovy(float fovy, float aspect) {
    if (aspect != g_render_aspect) return fovy;
    fovy *= g_fov_scale;
    if (aspect < 16.0f / 9.0f) fovy = 2.0f * atanf(tanf(fovy * 0.5f) * (16.0f / 9.0f) / aspect);
    return fovy;
}

// A picture shape: "Auto" (0), "21:9" / "21/9", or a number (0.5 .. 10).
static bool parse_aspect(const char *v, float *out) {
    float n, d, a;
    char sep, *end;
    if (!strcasecmp(v, "auto")) { *out = 0; return true; }
    if (sscanf(v, "%f %c %f", &n, &sep, &d) == 3 && (sep == ':' || sep == '/') && n > 0 && d > 0) {
        a = n / d;
    } else {
        a = strtof(v, &end);
        if (end == v || *end) return false;
    }
    if (a < 0.5f || a > 10.0f) return false;
    *out = a;
    return true;
}

// UIAnchor: Center (also 1, 16:9), 21:9 (2), 32:9 (3) or Edges, as Config.ui_anchor's UI_*
static bool parse_anchor(const char *v, int *out) {
    if (!strcasecmp(v, "center") || !strcmp(v, "1") || !strcmp(v, "16:9")) *out = UI_CENTER;
    else if (!strcmp(v, "2") || !strcmp(v, "21:9")) *out = UI_21_9;
    else if (!strcmp(v, "3") || !strcmp(v, "32:9")) *out = UI_32_9;
    else if (!strcasecmp(v, "edges")) *out = UI_EDGES;
    else return false;
    return true;
}

static bool parse_fsr_quality(const char *v, int *out) {
    if (!strcmp(v, "nativeaa") || !strcmp(v, "native") || !strcmp(v, "aa")) *out = FSR_NATIVE;
    else if (!strcmp(v, "quality")) *out = FSR_QUALITY;
    else if (!strcmp(v, "balanced")) *out = FSR_BALANCED;
    else if (!strcmp(v, "performance")) *out = FSR_PERFORMANCE;
    else if (!strcmp(v, "ultraperformance") || !strcmp(v, "ultraperf")) *out = FSR_ULTRA_PERFORMANCE;
    else return false;
    return true;
}

static int desktop_w, desktop_h;
static bool render_auto = true;

static const struct {
    const char *key;
    int binding;
} keyboard_settings[] = {
    {"dpadup", KEY_DPAD_UP}, {"dpadupalt", KEY_DPAD_UP_ALT},
    {"dpadright", KEY_DPAD_RIGHT}, {"dpadrightalt", KEY_DPAD_RIGHT_ALT},
    {"dpaddown", KEY_DPAD_DOWN}, {"dpaddownalt", KEY_DPAD_DOWN_ALT},
    {"dpadleft", KEY_DPAD_LEFT}, {"dpadleftalt", KEY_DPAD_LEFT_ALT},
    {"buttonb", KEY_BUTTON_B}, {"buttonbalt", KEY_BUTTON_B_ALT}, {"buttonx", KEY_BUTTON_X},
    {"buttonxalt", KEY_BUTTON_X_ALT}, {"buttony", KEY_BUTTON_Y}, {"buttonyalt", KEY_BUTTON_Y_ALT},
    {"buttonl", KEY_BUTTON_L}, {"buttonlalt", KEY_BUTTON_L_ALT},
    {"buttonr", KEY_BUTTON_R}, {"buttonralt", KEY_BUTTON_R_ALT},
    {"buttonzl", KEY_BUTTON_ZL}, {"buttonzlalt", KEY_BUTTON_ZL_ALT},
    {"buttonzr", KEY_BUTTON_ZR}, {"buttonzralt", KEY_BUTTON_ZR_ALT},
    {"buttonplus", KEY_BUTTON_PLUS}, {"buttonplusalt", KEY_BUTTON_PLUS_ALT},
    {"buttonplusalt2", KEY_BUTTON_PLUS_ALT2}, {"buttonminus", KEY_BUTTON_MINUS},
    {"buttonminusalt", KEY_BUTTON_MINUS_ALT}, {"buttonleftstick", KEY_BUTTON_LSTICK},
    {"buttonleftstickalt", KEY_BUTTON_LSTICK_ALT},
    {"buttona", KEY_BUTTON_A}, {"buttonaalt", KEY_BUTTON_A_ALT}, {"buttonrightstick", KEY_BUTTON_RSTICK},
    {"buttonrightstickalt", KEY_BUTTON_RSTICK_ALT},
    {"stickup", KEY_STICK_UP}, {"stickleft", KEY_STICK_LEFT},
    {"stickdown", KEY_STICK_DOWN}, {"stickright", KEY_STICK_RIGHT},
};

static const struct {
    const char *key;
    int binding;
} mouse_settings[] = {
    {"leftbutton", MOUSE_LEFT}, {"middlebutton", MOUSE_MIDDLE}, {"rightbutton", MOUSE_RIGHT},
    {"x1button", MOUSE_X1}, {"x2button", MOUSE_X2},
};

static const struct {
    const char *name;
    int button;
} mouse_button_names[] = {
    {"a", BTN_A}, {"b", BTN_B}, {"x", BTN_X}, {"y", BTN_Y},
    {"lstick", BTN_LSTICK}, {"rstick", BTN_RSTICK}, {"l", BTN_L}, {"r", BTN_R},
    {"zl", BTN_ZL}, {"zr", BTN_ZR}, {"plus", BTN_PLUS}, {"minus", BTN_MINUS},
    {"dpadleft", BTN_LEFT}, {"dpadup", BTN_UP}, {"dpadright", BTN_RIGHT}, {"dpaddown", BTN_DOWN},
};

static int parse_mouse_button(const char *name) {
    if (!strcmp(name, "none")) return -1;
    for (unsigned i = 0; i < sizeof mouse_button_names / sizeof *mouse_button_names; i++)
        if (!strcmp(name, mouse_button_names[i].name)) return mouse_button_names[i].button;
    return -2;
}

static void set_key(const char *key, char *val, int line) {
    char k[64], v[256];
    snprintf(k, sizeof k, "%s", key);
    snprintf(v, sizeof v, "%s", val);
    squash(k);
    squash(v);
    for (unsigned i = 0; i < sizeof keyboard_settings / sizeof *keyboard_settings; i++) {
        if (strcmp(k, keyboard_settings[i].key)) continue;
        SDL_Scancode scancode = !strcmp(v, "none") ? SDL_SCANCODE_UNKNOWN : SDL_GetScancodeFromName(val);
        if ((!strcmp(v, "none") || scancode != SDL_SCANCODE_UNKNOWN) && scancode < SDL_SCANCODE_COUNT)
            g_config.keyboard[keyboard_settings[i].binding] = scancode;
        else
            LOG("[config] line %d: \"%s\" is not a recognized keyboard key; using the default\n", line, val);
        return;
    }
    for (unsigned i = 0; i < sizeof mouse_settings / sizeof *mouse_settings; i++) {
        if (strcmp(k, mouse_settings[i].key)) continue;
        int button = parse_mouse_button(v);
        if (button != -2) g_config.mouse_buttons[mouse_settings[i].binding] = button;
        else LOG("[config] line %d: \"%s\" is not a recognized mouse button mapping; using the default\n", line, val);
        return;
    }
    bool ok = true;
    if (!strcmp(k, "mode")) {
        if (!strcmp(v, "fullscreen") || !strcmp(v, "exclusive") || !strcmp(v, "exclusivefullscreen"))
            g_config.display_mode = DISPLAY_FULLSCREEN;
        else if (!strcmp(v, "windowed") || !strcmp(v, "window")) g_config.display_mode = DISPLAY_WINDOWED;
        else if (!strncmp(v, "borderless", 10)) g_config.display_mode = DISPLAY_BORDERLESS;
        else ok = false;
    } else if (!strcmp(k, "resolution")) {
        if (!strcmp(v, "desktop") || !strcmp(v, "auto")) g_config.width = g_config.height = 0;
        else ok = parse_size(v, &g_config.width, &g_config.height);
    } else if (!strcmp(k, "renderresolution")) {
        if (!strcmp(v, "auto")) render_auto = true;
        else if ((ok = parse_size(v, &g_config.render_w, &g_config.render_h))) render_auto = false;
    } else if (!strcmp(k, "aspectratio")) {
        ok = parse_aspect(v, &g_config.aspect_ratio);
    } else if (!strcmp(k, "fov")) {
        float f = strtof(v, NULL);
        if ((ok = f >= 0.5f && f <= 2.0f)) g_config.fov = f;
    } else if (!strcmp(k, "uianchor")) {
        ok = parse_anchor(v, &g_config.ui_anchor);
    } else if (!strcmp(k, "msaa")) {
        if (!strcmp(v, "off") || !strcmp(v, "none") || !strcmp(v, "0") || !strcmp(v, "1") || !strcmp(v, "1x"))
            g_config.msaa = 1;
        else if (!strcmp(v, "2x") || !strcmp(v, "2")) g_config.msaa = 2;
        else if (!strcmp(v, "4x") || !strcmp(v, "4")) g_config.msaa = 4;
        else if (!strcmp(v, "8x") || !strcmp(v, "8")) g_config.msaa = 8;
        else ok = false;
    } else if (!strcmp(k, "backgroundinput")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.background_input = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.background_input = 0;
        else ok = false;
    } else if (!strcmp(k, "fxaa")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.fxaa = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.fxaa = 0;
        else ok = false;
    } else if (!strcmp(k, "ssao")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.ssao = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.ssao = 0;
        else ok = false;
    } else if (!strcmp(k, "fsr")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.fsr = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.fsr = 0;
        else ok = false;
    } else if (!strcmp(k, "fsrquality")) {
        ok = parse_fsr_quality(v, &g_config.fsr_quality);
    } else if (!strcmp(k, "sharpness")) {
        char *end;
        float f = strtof(v, &end);
        if ((ok = end != v && !*end && f >= 0.0f && f <= 1.0f)) g_config.fsr_sharpness = f;
    } else if (!strcmp(k, "objectmotion") || !strcmp(k, "animsurf")) {
        int *setting = !strcmp(k, "objectmotion") ? &g_config.fsr_objmotion : &g_config.fsr_animsurf;
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) *setting = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) *setting = 0;
        else ok = false;
    } else if (!strcmp(k, "reactive")) {
        char *end;
        float f = strtof(v, &end);
        if ((ok = end != v && !*end && f >= 0.0f && f <= 1.0f)) g_config.fsr_reactive = f;
    } else if (!strcmp(k, "vsync")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.vsync = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.vsync = 0;
        else ok = false;
    } else if (!strcmp(k, "facebuttons")) {
        if (!strcmp(v, "position") || !strcmp(v, "positional") || !strcmp(v, "nintendo"))
            g_config.face_buttons = FACE_POSITION;
        else if (!strcmp(v, "label") || !strcmp(v, "labels") || !strcmp(v, "xbox"))
            g_config.face_buttons = FACE_LABEL;
        else ok = false;
    } else if (!strcmp(k, "rumble")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.controller_rumble = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.controller_rumble = 0;
        else ok = false;
    } else if (!strcmp(k, "mousesensitivity")) {
        float f = strtof(v, NULL);
        if ((ok = f >= 0.1f && f <= 10.0f)) g_config.mouse_sensitivity = f;
    } else if (!strcmp(k, "inverthorizontal")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.camera_invert_horizontal = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.camera_invert_horizontal = 0;
        else ok = false;
    } else if (!strcmp(k, "invertvertical")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.camera_invert_vertical = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.camera_invert_vertical = 0;
        else ok = false;
    } else if (!strcmp(k, "smoothing")) {
        char *end;
        float f = strtof(v, &end);
        if ((ok = end != v && !*end && f >= 0.05f && f <= 1.0f)) g_config.camera_smoothing = f;
    } else if (!strcmp(k, "port")) {
        int p = atoi(v);
        if (p > 0 && p < 65536) g_config.ldn_port = p;
        else ok = false;
    } else if (!strcmp(k, "join")) {
        snprintf(g_config.ldn_join, sizeof g_config.ldn_join, "%s", val);  // (as written: names keep their dashes)
    } else if (!strcmp(k, "bonuscontent")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.bonus_content = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.bonus_content = 0;
        else ok = false;
    } else if (!strcmp(k, "villagequestsinhub")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.village_in_hub = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.village_in_hub = 0;
        else ok = false;
    } else if (!strcmp(k, "hubhp")) {
        float f[4];
        if ((ok = parse_factors(v, f) == 1)) g_config.hp.village = f[0];
    } else if (!strcmp(k, "syncsmallmonsters")) {
        if (!strcmp(v, "on") || !strcmp(v, "1") || !strcmp(v, "true") || !strcmp(v, "yes")) g_config.sync_small_monsters = 1;
        else if (!strcmp(v, "off") || !strcmp(v, "0") || !strcmp(v, "false") || !strcmp(v, "no")) g_config.sync_small_monsters = 0;
        else ok = false;
    } else if (k[0] >= '1' && k[0] <= '4' && (!strcmp(k + 1, "player") || !strcmp(k + 1, "players"))) {
        float f[4];
        if ((ok = parse_factors(v, f) == 1)) g_config.hp.scale[k[0] - '1'] = f[0];
    } else if (!strncmp(k, "quest", 5) && isdigit((unsigned char)k[5])) {
        HpScale *h = &g_config.hp;
        char *end;
        unsigned long q = strtoul(k + 5, &end, 10);
        float f[4];
        int n = parse_factors(v, f);
        if ((ok = !*end && q && (n == 1 || n == 4))) {
            int i = 0;
            while (i < h->quests && h->quest[i].quest != q) i++;
            if (i == HP_QUESTS_MAX) { LOG("[config] line %d: more than %d quests set apart\n", line, HP_QUESTS_MAX); return; }
            h->quest[i].quest = (uint32_t)q;
            for (int j = 0; j < 4; j++) h->quest[i].scale[j] = f[n == 4 ? j : 0];
            if (i == h->quests) h->quests++;
        }
    } else if (!strcmp(k, "fps") || !strcmp(k, "framerate")) {
        int f = atoi(v);
        if (f >= 30 && f <= 240) g_config.fps = f;
        else ok = false;
    } else {
        LOG("[config] line %d: unknown setting \"%s\" (ignored)\n", line, key);
        return;
    }
    if (!ok) LOG("[config] line %d: \"%s\" is not a value %s takes; using the default\n", line, val, key);
}

static char *trim(char *s) {
    while (isspace((unsigned char)*s)) s++;
    char *e = s + strlen(s);
    while (e > s && isspace((unsigned char)e[-1])) *--e = 0;
    return s;
}

// The largest size inside w x h matching the given aspect ratio, even numbers.
static void fit_aspect(int w, int h, float aspect, int *rw, int *rh) {
    if (aspect <= 0.0f) aspect = 16.0f / 9.0f;
    if ((float)w / (float)h > aspect)
        w = (int)((float)h * aspect + 0.5f);
    else
        h = (int)((float)w / aspect + 0.5f);
    *rw = w & ~1, *rh = h & ~1;
}

// Sections of DEFAULT_INI the file lacks (written by an older build) are added with their defaults at its end; the
// player's own settings stay as they are.
static void append_missing_sections(const char *path, const char *seen) {
    FILE *o = NULL;
    for (const char *p = strstr(DEFAULT_INI, "\n["); p;) {
        const char *start = p + 1, *end = strstr(start, "\n["), *close = strchr(start, ']');
        p = end;
        char name[64], key[70];
        snprintf(name, sizeof name, "%.*s", (int)(close - start - 1), start + 1);
        for (char *q = name; *q; q++) *q = (char)tolower((unsigned char)*q);
        snprintf(key, sizeof key, "|%s|", name);
        if (strstr(seen, key)) continue;
        if (!o && !(o = fopen(path, "a"))) return;
        fprintf(o, "\n%.*s", (int)(end ? end - start : (long)strlen(start)), start);
        LOG("[config] added the [%s] section to %s\n", name, path);
    }
    if (o) fclose(o);
}

static void migrate_default_keyboard_bindings(void) {
    int *key = g_config.keyboard;
    if (key[KEY_BUTTON_B] == SDL_SCANCODE_SPACE && key[KEY_BUTTON_A] == SDL_SCANCODE_F &&
        key[KEY_BUTTON_B_ALT] == SDL_SCANCODE_RETURN && key[KEY_BUTTON_A_ALT] == SDL_SCANCODE_BACKSPACE &&
        key[KEY_BUTTON_PLUS] == SDL_SCANCODE_RETURN && key[KEY_BUTTON_PLUS_ALT] == SDL_SCANCODE_KP_ENTER &&
        key[KEY_BUTTON_PLUS_ALT2] == SDL_SCANCODE_ESCAPE) {
        key[KEY_BUTTON_B_ALT] = SDL_SCANCODE_BACKSPACE;
        key[KEY_BUTTON_A_ALT] = SDL_SCANCODE_RETURN;
        key[KEY_BUTTON_PLUS] = SDL_SCANCODE_KP_ENTER;
        key[KEY_BUTTON_PLUS_ALT] = SDL_SCANCODE_ESCAPE;
        key[KEY_BUTTON_PLUS_ALT2] = SDL_SCANCODE_UNKNOWN;
    }
}

void config_load(void) {
    char path[1100];
    ini_path(path, sizeof path);
    FILE *f = fopen(path, "r");
    if (!f) {
        FILE *o = fopen(path, "w");
        if (o) { fputs(DEFAULT_INI, o); fclose(o); LOG("[config] wrote the default settings to %s\n", path); }
        else LOG("[config] %s is missing and could not be written; using the defaults\n", path);
    } else {
        char buf[512], seen[512] = "|";
        int line = 0;
        while (fgets(buf, sizeof buf, f)) {
            line++;
            char *s = trim(buf);
            if (*s == '[' && strchr(s, ']') && strlen(seen) + strlen(s) + 2 < sizeof seen) {
                for (char *q = s + 1; *q != ']'; q++) {
                    size_t n = strlen(seen);
                    seen[n] = (char)tolower((unsigned char)*q), seen[n + 1] = 0;
                }
                strcat(seen, "|");
            }
            if (!*s || *s == ';' || *s == '#' || *s == '[') continue;
            char *eq = strchr(s, '=');
            if (!eq) { LOG("[config] line %d: no '=' (ignored)\n", line); continue; }
            *eq = 0;
            char *val = trim(eq + 1), *cmt = strpbrk(val, ";#");
            if (cmt) *cmt = 0;
            set_key(trim(s), trim(val), line);
        }
        fclose(f);
        append_missing_sections(path, seen);
    }
    migrate_default_keyboard_bindings();

    const SDL_DisplayMode *dm = SDL_GetDesktopDisplayMode(SDL_GetPrimaryDisplay());
    desktop_w = dm ? dm->w : 1920, desktop_h = dm ? dm->h : 1080;
    if (!g_config.width) g_config.width = desktop_w, g_config.height = desktop_h;

    const char *e = getenv("MHGU_WINDOW");
    int w, h;
    if (e && parse_size(e, &w, &h)) g_config.display_mode = DISPLAY_WINDOWED, g_config.width = w, g_config.height = h;
    if (getenv("MHGU_HIDDEN")) g_config.display_mode = DISPLAY_WINDOWED;
    if ((e = getenv("MHGU_MSAA"))) g_config.msaa = atoi(e) >= 8 ? 8 : atoi(e) >= 4 ? 4 : atoi(e) >= 2 ? 2 : 1;
    if ((e = getenv("MHGU_FXAA"))) g_config.fxaa = atoi(e) != 0;
    if ((e = getenv("MHGU_SSAO"))) g_config.ssao = atoi(e) != 0;
    if ((e = getenv("MHGU_FSR"))) g_config.fsr = atoi(e) != 0;
    if ((e = getenv("MHGU_FSR_QUALITY"))) {
        char quality[32];
        snprintf(quality, sizeof quality, "%s", e);
        squash(quality);
        parse_fsr_quality(quality, &g_config.fsr_quality);
    }
    if ((e = getenv("MHGU_FSR_OBJMOTION"))) g_config.fsr_objmotion = atoi(e) != 0;
    if ((e = getenv("MHGU_FSR_ANIMSURF"))) g_config.fsr_animsurf = atoi(e) != 0;
    if ((e = getenv("MHGU_FSR_SHARPNESS")) && strtof(e, NULL) >= 0.0f && strtof(e, NULL) <= 1.0f)
        g_config.fsr_sharpness = strtof(e, NULL);
    if ((e = getenv("MHGU_FSR_REACTIVE")) && strtof(e, NULL) >= 0.0f && strtof(e, NULL) <= 1.0f)
        g_config.fsr_reactive = strtof(e, NULL);
    if ((e = getenv("MHGU_BACKGROUND_INPUT"))) g_config.background_input = atoi(e) != 0;
    if ((e = getenv("MHGU_PAD_POSITIONAL"))) g_config.face_buttons = atoi(e) ? FACE_POSITION : FACE_LABEL;
    if ((e = getenv("MHGU_LDN_PORT")) && atoi(e) > 0) g_config.ldn_port = atoi(e);
    if ((e = getenv("MHGU_SYNC_ZAKO"))) g_config.sync_small_monsters = atoi(e) != 0;
    if ((e = getenv("MHGU_LDN_JOIN"))) snprintf(g_config.ldn_join, sizeof g_config.ldn_join, "%s", e);
    if ((e = getenv("MHGU_ASPECT"))) parse_aspect(e, &g_config.aspect_ratio);
    if ((e = getenv("MHGU_FOV")) && strtof(e, NULL) >= 0.5f && strtof(e, NULL) <= 2.0f) g_config.fov = strtof(e, NULL);
    if ((e = getenv("MHGU_UI_ANCHOR"))) parse_anchor(e, &g_config.ui_anchor);
    e = getenv("MHGU_RENDER");
    if (e && parse_size(e, &w, &h)) render_auto = false, g_config.render_w = w, g_config.render_h = h;

    int tw = g_config.width, th = g_config.height;  // (Borderless shows the image at the desktop size)
    if (g_config.display_mode == DISPLAY_BORDERLESS) tw = tw < desktop_w ? tw : desktop_w, th = th < desktop_h ? th : desktop_h;
    // The picture's shape: the window's or AspectRatio's, at least 16:10. The game lays its HUD out for 16:9, which
    // still fits down to 16:10 (a little flatter, nvn/vulkan/exec.c), so a narrower window (4:3) shows 16:10 with bars.
    // Wider than 16:9 shows more to the sides, 16:9 to 16:10 more above and below (camera_fovy, gen.py PATCHES).
    float aspect = g_config.aspect_ratio > 0 ? g_config.aspect_ratio : (float)tw / (float)th;
    if (aspect < 1.6f - 0.005f) {
        if (g_config.aspect_ratio > 0)
            LOG("[config] AspectRatio %.3f is narrower than 16:10, which the HUD needs: using 16:10\n", aspect);
        aspect = 1.6f;
    }
    if (fabsf(aspect - 16.0f / 9.0f) < 0.01f) aspect = 16.0f / 9.0f;  // (1366x768 and the like)
    if (render_auto && g_config.fsr && g_config.display_mode == DISPLAY_BORDERLESS && (tw < desktop_w || th < desktop_h)) {
        fit_aspect(desktop_w, desktop_h, aspect, &w, &h);
        LOG("[config] FSR, Borderless: upscaling to %dx%d (the screen, in the picture's shape) instead of Resolution %dx%d\n",
            w, h, tw, th);
    } else if (render_auto) {
        fit_aspect(tw, th, aspect, &w, &h);
    } else {
        fit_aspect(g_config.render_w, g_config.render_h, aspect, &w, &h);
        if (w != g_config.render_w || h != g_config.render_h)
            LOG("[config] RenderResolution %dx%d is not the picture's shape (%.3f): using %dx%d\n", g_config.render_w,
                g_config.render_h, aspect, w, h);
    }
    if (g_config.fsr) {
        static const float ratios[] = {1.0f, 1.5f, 1.7f, 2.0f, 3.0f};
        static const char *const names[] = {"NativeAA", "Quality", "Balanced", "Performance", "UltraPerformance"};
        g_fsr_output[0] = (uint32_t)w;
        g_fsr_output[1] = (uint32_t)h;
        float ratio = ratios[g_config.fsr_quality];
        fit_aspect((int)(w / ratio + 0.5f), (int)(h / ratio + 0.5f), aspect, &w, &h);
        if (g_config.msaa > 1) LOG("[config] FSR is on: MSAA off (FSR does the anti-aliasing)\n");
        if (g_config.fxaa) LOG("[config] FSR is on: FXAA off (FSR does the anti-aliasing)\n");
        g_config.msaa = 1;
        g_config.fxaa = 0;
        g_fsr_lod_bias = log2f((float)w / (float)g_fsr_output[0]);
        if ((e = getenv("MHGU_FSR_LODBIAS"))) g_fsr_lod_bias = strtof(e, NULL);
        LOG("[config] FSR %s (x%.1f): the 3D picture at %dx%d, upscaled to %ux%u, sharpness %.2f, texture LOD bias %.2f\n",
            names[g_config.fsr_quality], ratio, w, h, g_fsr_output[0], g_fsr_output[1], g_config.fsr_sharpness, g_fsr_lod_bias);
    }
    g_config.render_w = w, g_config.render_h = h;
    g_render_size[0] = (uint32_t)w;
    g_render_size[1] = (uint32_t)h;
    g_render_aspect = aspect;
    g_fov_scale = g_config.fov;

    // The HUD's width: 16:9 (Center), 21:9, 32:9 or the picture's (Edges), at most the picture's; the game's 2D canvas
    // gets that width (gen.py PATCHES), exec.c places the HUD's parts in it.
    float ui = g_config.ui_anchor == UI_EDGES  ? aspect
             : g_config.ui_anchor == UI_32_9 ? 32.0f / 9.0f
             : g_config.ui_anchor == UI_21_9 ? 21.0f / 9.0f
                                             : 16.0f / 9.0f;
    g_ui_anchor_aspect = ui < aspect ? ui : aspect;
    g_ui_size[1] = 720;
    g_ui_size[0] = (uint32_t)(720.0f * g_ui_anchor_aspect + 0.5f) & ~1u;
    if (g_ui_size[0] < 1280) g_ui_size[0] = 1280;

    static const char *const modes[] = {"windowed", "fullscreen", "borderless"};
    LOG("[config] %s: %s %dx%d, 3D %dx%d (shape %.3f, FOV x%.2f, HUD %.3f wide), %d fps, vsync %s, MSAA %dx, FXAA %s, SSAO %s, "
        "face buttons by %s, controller %s in the background\n", path, modes[g_config.display_mode], g_config.width,
        g_config.height, g_config.render_w, g_config.render_h, g_render_aspect, g_fov_scale, g_ui_anchor_aspect,
        g_config.fps, g_config.vsync ? "on" : "off",
        g_config.msaa, g_config.fxaa ? "on" : "off", g_config.ssao ? "on" : "off",
        g_config.face_buttons == FACE_LABEL ? "label" : "position",
        g_config.background_input ? "on" : "off");
    const HpScale *hp = &g_config.hp;
    LOG("[config] monster HP in Hub quests: x%.2f / %.2f / %.2f / %.2f for 1-4 hunters, %d quest(s) set apart\n",
        hp->scale[0], hp->scale[1], hp->scale[2], hp->scale[3], hp->quests);
    LOG("[config] Village quests at the Hub counter: %s (their HP x%.2f first)\n", g_config.village_in_hub ? "on" : "off",
        hp->village);
}
