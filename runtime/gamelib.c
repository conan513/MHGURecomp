// Loads the recompiled game code library (runtime/gamelib.h).
#include "gamelib.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <dlfcn.h>
#endif

#include "rt.h"

static const GameLibTables *g_tables;

bool gamelib_load(char *why, size_t why_size) {
    if (g_tables) return true;
    char path[1100];
    const char *env = getenv("MHGU_GAME_LIB");
    if (env && *env) {
        snprintf(path, sizeof path, "%s", env);
    } else {
        char exe[1024];
        if (!plat_exe_path(exe, sizeof exe)) exe[0] = 0;
        char *slash = strrchr(exe, '/'), *bs = strrchr(exe, '\\');
        if (bs > slash) slash = bs;
        if (slash) slash[1] = 0; else exe[0] = 0;
        snprintf(path, sizeof path, "%s%s", exe, GAMELIB_NAME);
    }
#ifdef _WIN32
    int wide_size = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, NULL, 0);
    wchar_t *wide_path = wide_size ? malloc((size_t)wide_size * sizeof(*wide_path)) : NULL;
    if (!wide_path || !MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, wide_path, wide_size)) {
        free(wide_path);
        snprintf(why, why_size, "invalid UTF-8 game library path: %s", path);
        return false;
    }
    HMODULE h = LoadLibraryW(wide_path);
    DWORD load_error = h ? ERROR_SUCCESS : GetLastError();
    free(wide_path);
    if (!h) { snprintf(why, why_size, "can't load %s (error %lu): run `make gen` and `make` first", path, load_error); return false; }
    GameLibEntry entry = (GameLibEntry)(void *)GetProcAddress(h, GAMELIB_ENTRY);
#else
    void *h = dlopen(path, RTLD_NOW | RTLD_LOCAL);
    if (!h) { snprintf(why, why_size, "%s", dlerror()); return false; }
    GameLibEntry entry = (GameLibEntry)dlsym(h, GAMELIB_ENTRY);
#endif
    if (!entry) { snprintf(why, why_size, "%s: not game code", path); return false; }
    const GameLibTables *t = entry();
    if (!t || t->abi != GAMELIB_ABI || !t->id || strcmp(t->id, GAMELIB_ID)) {
        snprintf(why, why_size, "%s was built for another version of the runtime: rebuild it (make)", path);
        return false;
    }
    g_tables = t;
    LOG("[gamelib] %s (%u + %u functions)\n", path, *t->main_count, *t->sdk_count);
    return true;
}

const GameLibTables *gamelib_tables(void) { return g_tables; }
