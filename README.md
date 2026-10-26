h# MHGU Recomp

A native PC port of **Monster Hunter Generations Ultimate** (Nintendo Switch, version 1.4.0), made by statically
recompiling the game's 32-bit ARM code to C and running it on a host runtime: Switch system services and the NVN
graphics API are reimplemented on Windows, with Vulkan for rendering.

**No game files are included.** You need your own copy of the game, dumped from your own Switch. This project does
not explain how to dump it and does not provide game files, keys or links to them.

Status: fully playable, but there may be some bugs. I've played a dozen hunts with no remaining issues. Multiplayer also works.

## What it adds over the Switch version

- Graphics settings (resolution, MSAA, FXAA, Supersampling, VSync)
- Any screen shape from 16:10 to ultrawide (21:9, 32:9 and wider), with the HUD unstretched: centered or spread to
  the screen's sides
- Frame rate cap anywhere between 30-240 fps
- Supports almost every controller under the sun
- Experimental KB+M controls
- Optional: small monsters are properly synced between players in multiplayer
- Optional: monster HP scaling by playercount
- Fast-forward unskippable cutscenes by pressing A
- Optional: village quests are playable in multiplayer (press L/R at the quest counter), with appropriately re-scaled monster HP
- Mod support. The vast majority of mods work by dropping them into the `mods` folder, including Forge mods. (see [Mods](#mods))
- Cheat codes in the same format as Ryujinx and Atmosphère, in the `cheats` folder (see [Cheats](#cheats))

## Requirements

- OS: Windows 10/11 64-bit, or Linux
- CPU: Any CPU from 2014 or newer (needs AVX2 support)
- GPU: Any GPU from 2016 or newer (needs Vulkan 1.2 support), and at least 1GB of VRAM
- The game files for Monster Hunter Generations Ultimate, version 1.4.0. You can extract these from your Switch or Ryujinx with patches pre-applied.
    - (Title ID `0100770008DD8000`, the `main` executable's build ID starts with `FB08F1D20FD1204F`)

## Installation

[Download the latest prebuilt release from the Releases page. Make sure you download the right ZIP file for your desired platform.](https://github.com/morerokk/MHGURecomp/releases)

You need the `exefs` and `romfs` folders from Monster Hunter Generations Ultimate 1.4.0. If you are coming from Ryujinx, you can right-click the game (make sure the 1.4.0 title update is applied), and Extract the game files with the patches pre-applied.

The launcher requires the Microsoft Edge WebView2 Runtime. It is included with current Windows 11 installations;
on other systems, install the [Evergreen Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

1. Extract the release zip anywhere, for example `C:\Games\MHGURecomp\`.
2. Put the game's `exefs` and `romfs` folders in the `data` subfolder. It should look like this:

   ```
   MHGURecomp\
     mhgurecomp.exe
     MHGU-Launcher.exe
     WebView2Loader.dll
     mhgugame.dll
     (other .dll files)
     data\
       exefs\
         main
         main.npdm
         rtld
         sdk
         subsdk0
       romfs\
         nativeNX\
         (the rest of the game's romfs)
   ```

   `exefs` must come from the 1.4.0 update, and `romfs` must be the base game's romfs with the update's files
   merged over it.
3. Start `MHGU-Launcher.exe` for the launcher home screen and game settings, or start `mhgurecomp.exe` directly.

If the files are missing or the wrong version, a message box says so.

The package is portable: keep the files together when moving the folder. Settings, caches, mods, and saves stay
beside the game; spaces and non-English characters in the folder path are supported. The only external requirement
is the Windows WebView2 Runtime.

### Where things are

| What | Where |
| --- | --- |
| Settings | `mhgurecomp.ini` next to the exe (written with the defaults on first game start; the launcher can edit it) |
| Save data | `save\` beside the exe (existing AppData saves are copied here on first launch; the original is kept as a backup) |
| Log | `log.txt` next to the exe (look here first when something goes wrong) |
| Screenshots | `shots\` next to the exe (F12) |
| Mods | `mods\` next to the exe: one folder per mod (see below) |
| Cheats | `cheats\` next to the exe: `.txt` files with cheat codes (see below) |
| Shader cache | `cache\` next to the exe (safe to delete; the first minutes after that may stutter) |

To move an existing Switch save over, put its `system` and `system_backup` files in `save\save_0\` beside the exe.
On the first launch after this update, an existing `%APPDATA%\MHGURecomp\save\` is copied there automatically if
the portable `save\` folder does not already exist. The AppData copy is left untouched as a backup.

## Controls

Almost every controller under the sun should work, as long as it has SDL support. `FaceButtons = Position` uses the Switch layout: the bottom button acts as B and the right button as A, whatever is printed on them. Set `FaceButtons = Label` to match the printed letters instead (for example, an Xbox A button acts as A).

Keyboard and mouse support (not recommended):

| Key | Switch button |
| --- | --- |
| W A S D | left stick (hold Left Shift to walk) |
| Mouse | right stick (camera) |
| Mouse buttons | Configurable Switch buttons (default: left = X, right = A) |
| Space / R | B / Y |
| Q / E | L / R |
| Tab / T | ZL / ZR |
| Enter / Backspace | A (confirm) / B (back) |
| Keypad Enter / Escape | Plus / Plus (alternate) |
| 1 / 2 / 3 / 4 and arrow keys | d-pad up / right / down / left |
| Ctrl / Alt | left / right stick click |

Click in the game window to capture the mouse for the camera; Escape lets it go (so does switching to another
window). `MouseSensitivity` in the ini's `[Mouse]` section sets its speed; the game's own camera options (speed,
inverted axes) apply to it too.

Keyboard bindings can be changed in the launcher's **Controls** settings by clicking a key field and pressing a
replacement key. The same bindings are stored in the ini's `[Keyboard]` section (for example, `DpadUp = 1` and
`DpadUpAlt = Up`); the alternate D-pad keys let both the number row and arrow keys work at once. Key names use SDL's
usual names, such as `Left Shift` or `Keypad Enter`.

Mouse button assignments can also be changed in **Controls**. The left, middle, right and two side buttons can each
activate any Switch button or be set to `None`; the first click that captures the mouse is not sent to the game.
These assignments are stored in `[Mouse]` as `LeftButton`, `MiddleButton`, `RightButton`, `X1Button` and `X2Button`.

When the game asks for text (names, chat), Enter confirms, Escape cancels, and Ctrl+V pastes.

Other keys: Alt+Enter switches between windowed and full screen, F6 toggles SSAO, F7 cycles MSAA, F12 takes a screenshot.

## Settings

Everything is in `mhgurecomp.ini`; each setting is explained in the file itself. On Windows, the
`MHGU-Launcher.exe` companion has a home screen for starting the game and a separate settings view for all options,
including per-quest HP multipliers. It follows the Windows light/dark preference and saves the ini while preserving
settings it does not manage. The ini file can still be edited manually. In short:

- `[Display]`: `Mode` (Borderless / Fullscreen / Windowed), `Resolution`, `RenderResolution` (the 3D resolution),
  `AspectRatio` (Auto follows the window: 16:10 and anything wider fill it, narrower gets 16:10 with bars), `FOV`
  (1.0 = the game's), `UIAnchor` (Center keeps the HUD in the middle 16:9 on an ultrawide screen, Edges spreads it
  to the sides)
- `[Anti-aliasing]`: `MSAA` (Off / 2x / 4x / 8x), `FXAA` (On / Off), `SSAO` (On / Off; experimental screen-space contact shadows, only when a matching depth buffer is available)
- `[Performance]`: `FPS` (30 to 240), `VSync` (On / Off; above your monitor's refresh rate, turn VSync off)
- `[Keyboard]`: configurable keys for the keyboard's Switch buttons and stick directions
- `[Controls]`: `FaceButtons` (Position / Label), `BackgroundInput` (On / Off: the controller keeps working while
  the game's window is in the background, such as with a wiki in front of it; default On)
- `[Mouse]`: `MouseSensitivity` (1 = default, 0.1 to 10) and configurable mouse button assignments
- `[Multiplayer]`: `Port`, `Join` (addresses of hosts outside your LAN), `SyncSmallMonsters`
- `[Monster HP]`: HP factors for 1 to 4 hunters in Hub quests, and per-quest exceptions (default: all 1, the
  original game)
- `[Village Quests]`: `VillageQuestsInHub` (default On), `HubHP` (default 2.0: Village quests posted in the Hub
  multiply their monsters' HP by this first, then by the `[Monster HP]` factor)
- `[Bonus Content]`: `BonusContent` (default On), see below

### Building on Windows

With the MSYS2 UCRT64 build dependencies installed, run `make -j2` from the repository. On Windows, a successful
build also copies the complete runtime package (the executables, required DLLs, and any built game/data files) to
`F:\mhgu-build-tools\build`. Override the destination with `make PACKAGE_DIR=/f/another/path -j2` if needed. The package
does not include game files unless you supplied your own dump for the build; the release still needs your own game data.

### Multiplayer

Over LAN or a VLAN, this should work out of the box.

Over the internet, the host needs to forward the right port (39500 TCP/UDP by default), the client needs to configure the host's IP address in their .ini file. (`Join = xxx.xxx.xxx.xxx`)

Choose the "Local Wireless" multiplayer options, as this port pretends to be a local nearby Switch with a wireless connection. The online options are unavailable and are unnecessary.

### Bonus content

By default, the `bonus` subfolder in releases contains a savegame with all the free online content (event quests, item packs, palicoes), which will be copied into your savegame when the game starts the 2nd time.

If you don't want this, set `BonusContent = Off` or don't provide a savegame in the `bonus` subfolder. You can also provide your own savegame instead if you want. Note that this port will not attempt to copy bonus content if such bonus content already exists.

### Mods

To install a mod, put the mod folder in `mods\`. Remove the folder to uninstall said mod. Mods are layered over the game at startup.
- romfs mods (files that replace or add game data) are found inside the mod's folder, including the
  `atmosphere\contents\0100770008DD8000\romfs` layout.
- exefs patches (`.pchtxt`, `.ips`) are applied when they change data. Patches to game code can't be applied, because
  the code is precompiled; the log lists them.
- Forge plugins (`.nro` in `romfs\nativeNX\plugins\`) are loaded by the runtime as 32-bit ARM modules; Forge's API
  and ImGui calls are adapted to the recompiled game and this port's overlay, not run through Forge's Switch module.
  ImGui font paths in the game/mod RomFS are resolved on the host. The host adapter supports Forge 1.5.1; its
  socket API is not implemented. The default Roboto atlas includes its extended Latin, Greek, Cyrillic and symbol
  glyphs, but not CJK/Japanese characters. The FF14 Damage Text plugin's move-name font is separated from the
  damage number with a vertical rule. Forge's menu with the plugins' settings opens with Insert (or the key in Forge's
  own `config.ini`, if a mod includes Forge's files);
  use the mouse, keyboard or controller in it. While it is open the game gets no controller input. Window
  positions are kept in `forge_imgui.ini`.
- You do not need any resolution, FPS cap or connection fix mods. These features are included with this port.

If you are newer to the game, I very strongly recommend the [Key Quest Markers](https://gamebanana.com/mods/727960) mod by PurplefinNeptuna. This marks out the Village and Hub quests you need to complete for the next tier directly in the quest menu, which means you no longer need a wiki to play the damn game.

### Cheats

Put `.txt` files with cheat codes in `cheats\`. Every cheat in them is on from the start. Codes made for Ryujinx or a
modded Switch work as they are, as long as they are for version 1.4.0. A file can hold any number of cheats, each
starting with its name in square brackets:

```
[Skill points x2]
04000000 000DFAB4 E1A00080
```

- To turn off a file without deleting it, start its name with `_`.
- Cheats that change the game's code also work: the port runs each changed function in an interpreter. That
  function runs slower, which only matters for functions the game calls very often.
- `log.txt` lists the cheats it loaded, and any it couldn't read.
- `extras\cheats\` in the source has two cheats that double the skill points from armor, weapon and talisman
  decorations, and talismans (with Double Talisman, talisman points go x4). `Skill points x2` doubles negative
  points too; `Skill points x2 (positive only)` leaves them as they are. Use only one of the two.
- `Valor Sheathe original damage reduction` sets the damage you take in a Valor Sheathe evade back to the game's
  own values (70% less for Blademasters, 85% for Gunners), for mods that change it, such as GU Rebalanced (50% / 65%).
  The rest of the mod stays as it is. To pick other values, see `docs/research/valor-sheathe.md`.

## Building from source

The build recompiles the game code from your own copy of the game. Nothing in this repository contains game code or
data, and the build never uploads anything.

### 1. Tools (Windows)

1. Install [MSYS2](https://www.msys2.org/) to `C:\msys64`.
2. Install [Rust](https://www.rust-lang.org/tools/install), the Microsoft C++ Build Tools, and the
   [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/). The WebView2 Runtime is included with
   current Windows 11 installations; install the Evergreen Runtime separately if it is missing.
3. Install [Node.js 22 LTS](https://nodejs.org/) for Windows and run `npm ci` from the repository folder. The launcher
   uses the Windows WebView2 runtime instead of bundling a browser engine, keeping its executable small and avoiding
   per-launch browser-engine extraction.
4. In the **MSYS2 UCRT64** shell, install the compiler and libraries:

   ```bash
   pacman -S --needed make git \
     mingw-w64-ucrt-x86_64-clang mingw-w64-ucrt-x86_64-lld mingw-w64-ucrt-x86_64-pkgconf \
     mingw-w64-ucrt-x86_64-sdl3 mingw-w64-ucrt-x86_64-ffmpeg mingw-w64-ucrt-x86_64-shaderc \
     mingw-w64-ucrt-x86_64-vulkan-headers mingw-w64-ucrt-x86_64-vulkan-loader \
     mingw-w64-ucrt-x86_64-vulkan-memory-allocator
   ```

5. Install [Python](https://www.python.org/) 3.10 or newer for Windows. Then, in a Command Prompt in the repository
   folder, make the tools' virtual environment (the Makefile uses `tools\venv` when it exists):

   ```bat
   py -3 -m venv tools\venv
   tools\venv\Scripts\pip install -r tools\requirements.txt
   ```

### 2. Recompile and build

From the UCRT64 shell, in the repository folder, with your game files in a folder holding `exefs\` and `romfs\` (as
under "Installing a release"):

```bash
make gen GAME=/c/path/to/game      # translate the game code to C in generated/ (~300 MB)
make -j4 GAME=/c/path/to/game      # compile everything into build/
```

`make gen` is required before the full build: it creates the game sources used to build `mhgugame.dll`. The game code
is not included in the repository. If you only want to build the runtime without your game files, use
`make -j4 build/mhgurecomp.exe bundle SKIP_GAMELIB=1`.

The GitHub Actions workflow builds and tests the launcher and runtime only. Its
`MHGURecomp-Runtime-Windows` artifact does not include `mhgugame.dll` or game data: generated game code is not stored
in the repository, and building it requires the user's own game files. To make a playable build, run `make gen` and
the full build above with your own game files.

The first full build takes a while (about 8 minutes on a 4-core Haswell with `-j4`; more cores: higher `-j`). Later
builds only recompile what changed.

`make` turns `build\` into a ready-to-run game folder, and puts nothing else there (the compiler's own files go
to `obj\`):
- `MHGU-Launcher.exe`, the lightweight WebView2 desktop launcher (requires the WebView2 Runtime);
- `mhgurecomp.exe`, the runtime;
- `mhgugame.dll`, the recompiled game code;
- the MSYS2 DLLs they need;
- `data\exefs` and `data\romfs`, hard links to your game files, so they take no extra disk space.

To play from another folder, copy the contents of `build\` there; to update that folder, copy them over it (`data\`
can be skipped when it is already there). The build never contains settings, caches, mods or logs, so your
`mhgurecomp.ini` and the rest stay as they are. Or start `run-game.bat`: it runs `build\mhgurecomp.exe` with the
settings, caches, `mods\` and `log.txt` in `run\` (`MHGU_HOME`), keeping `build\` clean. Starting
`build\mhgurecomp.exe` directly also works, but then it writes those into `build\`.

Useful targets and options:

- `make test-lift`: checks the recompiler against the Unicorn CPU emulator on thousands of the game's instructions.
- `make clean`: removes the compiled files (`obj\` and the program files in `build\`). It keeps `build\data`.
- `GENARCH=-march=x86-64-v2`: compiles the game code without AVX2, for older CPUs. Untested.

### Making a release

A release is the contents of `build\` without the game files:
- Include `MHGU-Launcher.exe`, `mhgurecomp.exe`, `mhgugame.dll` and all other `.dll` files.
- Leave out `data\`.

The exe creates the `mods` and `cheats` folders and the ini on first start. Players then add their own `data\` as
described above.

### Linux

The Windows build runs under Proton / Wine. A native Linux build is supported as well: the Makefile picks
`runtime/platform/linux.c` when `uname -s` isn't Windows and everything else is shared. Tested on Gentoo
x86_64 with clang under Wayland (NVIDIA). Besides `make`, `git`, `clang`, `lld` and Python 3.10 or newer it
needs:

- SDL3, the Vulkan loader and headers, shaderc and ffmpeg's avformat / avcodec / avutil — the same libraries
  the MSYS2 list above installs, under your distro's names;
- `vk_mem_alloc.h` (3.4.0) in `third_party/vk_mem_alloc/`, which the Makefile has on the include path (a
  distro package that installs the header where the compiler looks by default works just as well);
- `zenity` or `kdialog` for the dialog on a fatal error. Without either, the message goes to the terminal and
  to `log.txt` as usual.

```bash
python3 -m venv tools/venv && tools/venv/bin/pip install -r tools/requirements.txt
make gen GAME=../mhgu/game       # as above: the folder holding exefs/ and romfs/
make -j8 GAME=../mhgu/game
cd build && ./mhgurecomp
```

`build/` becomes the game folder exactly as on Windows: `mhgurecomp`, `mhgugame.so` with the recompiled game
code and `data/` as hard links to your game files, nothing else; the ini, caches and `mods/` are written beside
them when it runs (`MHGU_HOME=<folder>` puts those there instead). Saves go to
`$XDG_DATA_HOME/MHGURecomp/save`, default `~/.local/share/MHGURecomp/save` — the folder the Windows build
keeps them in under `%APPDATA%`, same layout, so moving a save across is copying `save_0`.

From Windows, the Linux build can be made in WSL with Arch Linux (`wsl --install -d archlinux`), straight from the
Windows checkout, into its own folders so the Windows build is left alone:

```bash
pacman -S --needed make git clang lld python sdl3 vulkan-headers vulkan-icd-loader shaderc ffmpeg vulkan-swrast
mkdir -p /usr/local/include/vma && cp /mnt/c/msys64/ucrt64/include/vma/vk_mem_alloc.h /usr/local/include/vma/
make -j16 B=build-linux O=obj-linux     # in the repository, under /mnt/c/...
```

Arch has no package for `vk_mem_alloc.h`; MSYS2's (3.4.0) is the one the Windows build uses. `build-linux/data` can
hold links to the game files (`ln -s /mnt/c/.../exefs build-linux/data/exefs`, same for `romfs`, then
`touch build-linux/data/.complete`) instead of a second copy. It also runs there, in a WSLg window, slowly: WSL's
own Vulkan driver (dzn, Vulkan on Direct3D 12) can't create the renderer's pipeline layouts, so use Mesa's software
one with `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json`. WSLg can't capture the mouse (clicks still work as
buttons), and screenshots come out black there.

## Repository layout

| Folder | What's in it |
| --- | --- |
| `tools/` | the recompiler: `lifter.py` (ARM to C), `neon.py` (NEON / VFP), `gen.py` (whole modules), `nso.py`, test and research tools |
| `runtime/` | the host side: CPU state, kernel and threads, Switch services (`services/`, `hle/`), networking (`net/`), the NVN graphics API on Vulkan (`nvn/`), settings, input, mods |
| `docs/` | `STATUS.md`: progress, design notes, reverse-engineering findings, debug options |
| `extras/` | `cheats/`: cheat files players can copy into their `cheats` folder |

## License

MHGU Recomp's own code is under the MIT License (`LICENSE`). It builds on code from LAS Recomp and Forge, and
bundles cimgui and Dear ImGui, all under the MIT License too: see `THIRD_PARTY_NOTICES`. The license does not cover
the game or its code, including the recompiled game code a build produces (`mhgugame.dll` / `mhgugame.so`).

## Legal

This project is not affiliated with or endorsed by Capcom or Nintendo. Monster Hunter is a trademark of Capcom. The
repository contains no game code, data or keys; the build reads them from a copy you provide. Please support the
developers by owning the game.
