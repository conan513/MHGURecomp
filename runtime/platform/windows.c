// Windows host layer (runtime/platform.h).
#include <windows.h>
#include <shlobj.h>
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <timeapi.h>
#include <wchar.h>

#include "../platform.h"

// ---- guest address space
// The 4 GB window is reserved up front; pages are committed on their first touch by the exception handler below
// (64 KB at a time), so only memory the game uses is backed.
static uint64_t g_lo, g_hi;
#define COMMIT_CHUNK 0x10000ull

static bool lazy_commit(uint64_t a) {
    if (a < g_lo || a >= g_hi) return false;
    MEMORY_BASIC_INFORMATION mi;
    if (!VirtualQuery((void *)(uintptr_t)a, &mi, sizeof mi) || mi.State != MEM_RESERVE) return false;
    uint64_t base = a & ~(COMMIT_CHUNK - 1);
    return VirtualAlloc((void *)(uintptr_t)base, COMMIT_CHUNK, MEM_COMMIT, PAGE_READWRITE) != NULL;
}

void (*plat_touch_hook)(uint64_t guest_addr, int write);

static LONG CALLBACK vectored_handler(EXCEPTION_POINTERS *ep) {
    EXCEPTION_RECORD *er = ep->ExceptionRecord;
    if (er->ExceptionCode != EXCEPTION_ACCESS_VIOLATION || er->NumberParameters < 2) return EXCEPTION_CONTINUE_SEARCH;
    uint64_t fa = er->ExceptionInformation[1];
    if (lazy_commit(fa)) {
        if (plat_touch_hook) plat_touch_hook(fa - g_lo, er->ExceptionInformation[0] == 1);
        return EXCEPTION_CONTINUE_EXECUTION;
    }
    return EXCEPTION_CONTINUE_SEARCH;
}

// The window keeps write tracking (MEM_WRITE_WATCH): the renderer re-uploads only textures / buffers whose pages the
// CPU wrote (plat_written_pages). Refused: no tracking, the renderer checks contents instead.
static bool g_write_watch;

bool plat_reserve_guest(uint64_t base, uint64_t size) {
    // (MHGU_NO_WRITE_WATCH=1: none, as on Linux - the renderer's content checks, for tests)
    void *p = getenv("MHGU_NO_WRITE_WATCH") ? NULL
            : VirtualAlloc((void *)(uintptr_t)base, size, MEM_RESERVE | MEM_WRITE_WATCH, PAGE_READWRITE);
    if (p == (void *)(uintptr_t)base) g_write_watch = true;
    else p = VirtualAlloc((void *)(uintptr_t)base, size, MEM_RESERVE, PAGE_READWRITE);
    if (p != (void *)(uintptr_t)base) return false;
    g_lo = base;
    g_hi = base + size;
    AddVectoredExceptionHandler(1, vectored_handler);
    return true;
}

void plat_commit(uint64_t addr, uint64_t size) {
    if (!size) return;
    uint64_t a = addr & ~0xFFFull, e = (addr + size + 0xFFF) & ~0xFFFull;
    if (!VirtualAlloc((void *)(uintptr_t)a, e - a, MEM_COMMIT, PAGE_READWRITE))
        fprintf(stderr, "[plat] commit %llx+%llx failed (%lu)\n", (unsigned long long)addr, (unsigned long long)size,
                GetLastError());
}

int plat_written_pages(uint64_t addr, uint64_t size, uint64_t *pages, int max, uint64_t *page) {
    if (!g_write_watch || addr < g_lo || addr + size > g_hi || max <= 0) return -1;
    ULONG_PTR n = (ULONG_PTR)max;
    DWORD gran = 4096;
    if (GetWriteWatch(WRITE_WATCH_FLAG_RESET, (void *)(uintptr_t)addr, (SIZE_T)size, (PVOID *)pages, &n, &gran) != 0) return -1;
    *page = gran;
    if (n >= (ULONG_PTR)max) ResetWriteWatch((void *)(uintptr_t)addr, (SIZE_T)size);  // (more than fit: caller takes all)
    return (int)n;
}

bool plat_committed(uint64_t addr, uint64_t *end) {
    MEMORY_BASIC_INFORMATION mi;
    if (!VirtualQuery((void *)(uintptr_t)addr, &mi, sizeof mi)) { *end = addr + 0x1000; return false; }
    *end = (uint64_t)(uintptr_t)mi.BaseAddress + mi.RegionSize;
    return mi.State == MEM_COMMIT && !(mi.Protect & (PAGE_NOACCESS | PAGE_GUARD));
}

int plat_protect(uint64_t addr, uint64_t size, int prot) {
    DWORD p = prot == PLAT_PROT_NONE ? PAGE_NOACCESS : prot == PLAT_PROT_READ ? PAGE_READONLY : PAGE_READWRITE, old;
    if (addr >= g_lo && addr < g_hi) plat_commit(addr, size);
    return VirtualProtect((void *)(uintptr_t)addr, size, p, &old) ? 0 : -1;
}

// ---- time
static LARGE_INTEGER qpf;

uint64_t plat_now_ns(void) {
    if (!qpf.QuadPart) QueryPerformanceFrequency(&qpf);
    LARGE_INTEGER t;
    QueryPerformanceCounter(&t);
    return (uint64_t)((unsigned __int128)t.QuadPart * 1000000000u / (uint64_t)qpf.QuadPart);
}

#ifndef CREATE_WAITABLE_TIMER_HIGH_RESOLUTION
#define CREATE_WAITABLE_TIMER_HIGH_RESOLUTION 0x2
#endif

void plat_sleep_until_ns(uint64_t t) {
    static __thread HANDLE timer;
    if (!timer) {
        timer = CreateWaitableTimerExW(NULL, NULL, CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS);
        if (!timer) timer = CreateWaitableTimerExW(NULL, NULL, 0, TIMER_ALL_ACCESS);
    }
    for (;;) {
        uint64_t now = plat_now_ns();
        if (now >= t) return;
        uint64_t left = t - now;
        if (left < 200000) { YieldProcessor(); continue; }
        LARGE_INTEGER due = {.QuadPart = -(LONGLONG)((left - 100000) / 100)};
        if (!timer || !SetWaitableTimer(timer, &due, 0, NULL, NULL, FALSE)) { Sleep(1); continue; }
        WaitForSingleObject(timer, INFINITE);
    }
}

// ---- process
void plat_init(void) {
    _fmode = _O_BINARY;
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
    SetConsoleOutputCP(CP_UTF8);
    timeBeginPeriod(1);
}

unsigned long plat_process_id(void) { return GetCurrentProcessId(); }
bool plat_wait_process(unsigned long pid) {
    HANDLE process = OpenProcess(SYNCHRONIZE, FALSE, (DWORD)pid);
    if (!process) return GetLastError() == ERROR_INVALID_PARAMETER;
    DWORD result = WaitForSingleObject(process, 30000);
    CloseHandle(process);
    return result == WAIT_OBJECT_0;
}

bool plat_exe_path(char *buf, size_t size) {
    wchar_t path[32768];
    DWORD n = GetModuleFileNameW(NULL, path, sizeof(path) / sizeof(*path));
    if (!n || n >= sizeof(path) / sizeof(*path)) return false;
    int bytes = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, path, (int)n, buf, (int)size, NULL, NULL);
    if (!bytes || (size_t)bytes >= size) return false;
    buf[bytes] = 0;
    return true;
}

// Started on its own (no game folder given): run in the executable's folder (MHGU_HOME: in that folder instead, created
// if missing - the settings, caches, mods and log there, the build folder untouched); a console window that is ours
// alone (a double-click in Explorer, a shortcut) goes away, the log goes to log.txt and fatal errors show a message box.
static bool g_own_log;

void plat_standalone(void) {
    wchar_t home[32768];
    DWORD home_len = GetEnvironmentVariableW(L"MHGU_HOME", home, sizeof(home) / sizeof(*home));
    if (home_len && home_len < sizeof(home) / sizeof(*home)) {
        CreateDirectoryW(home, NULL);
        SetCurrentDirectoryW(home);
    } else {
        wchar_t exe[32768];
        DWORD n = GetModuleFileNameW(NULL, exe, sizeof(exe) / sizeof(*exe));
        if (n && n < sizeof(exe) / sizeof(*exe)) {
            wchar_t *slash = wcsrchr(exe, L'\\');
            if (slash) { *slash = 0; SetCurrentDirectoryW(exe); }
        }
    }
    DWORD ids[2];
    if (GetConsoleProcessList(ids, 2) == 1) {  // (from a terminal the terminal shares it: keep logging there)
        FreeConsole();
        if (freopen("log.txt", "w", stderr)) { setvbuf(stderr, NULL, _IONBF, 0); g_own_log = true; }
    }
}

static bool legacy_user_dir(wchar_t *dir, size_t size) {
    PWSTR base = NULL;
    if (SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_CREATE, NULL, &base) != S_OK) {
        CoTaskMemFree(base);
        return false;
    }
    int n = _snwprintf(dir, size, L"%ls\\MHGURecomp", base);
    CoTaskMemFree(base);
    if (n < 0 || (size_t)n >= size) return false;
    return true;
}

static bool wide_join(wchar_t *path, size_t size, const wchar_t *directory, const wchar_t *name) {
    size_t directory_len = wcslen(directory), name_len = wcslen(name);
    bool separator = directory_len && directory[directory_len - 1] != L'\\' && directory[directory_len - 1] != L'/';
    if (directory_len + separator + name_len + 1 > size) return false;
    memcpy(path, directory, directory_len * sizeof(*path));
    if (separator) path[directory_len++] = L'\\';
    memcpy(path + directory_len, name, (name_len + 1) * sizeof(*path));
    return true;
}

static bool copy_save_tree(const wchar_t *source, const wchar_t *destination) {
    if (!CreateDirectoryW(destination, NULL)) return false;
    wchar_t search[32768], from[32768], to[32768];
    if (!wide_join(search, sizeof(search) / sizeof(*search), source, L"*")) return false;
    WIN32_FIND_DATAW entry;
    HANDLE find = FindFirstFileW(search, &entry);
    if (find == INVALID_HANDLE_VALUE) return false;

    bool ok = true;
    do {
        if (!wcscmp(entry.cFileName, L".") || !wcscmp(entry.cFileName, L"..")) continue;
        if (!wide_join(from, sizeof(from) / sizeof(*from), source, entry.cFileName) ||
            !wide_join(to, sizeof(to) / sizeof(*to), destination, entry.cFileName)) {
            ok = false;
            break;
        }
        if (entry.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) {
            ok = false;
            break;
        }
        if (entry.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
            if (!copy_save_tree(from, to)) { ok = false; break; }
        } else if (!CopyFileW(from, to, TRUE)) {
            ok = false;
            break;
        }
    } while (FindNextFileW(find, &entry));
    if (GetLastError() != ERROR_NO_MORE_FILES) ok = false;
    FindClose(find);
    return ok;
}

static void remove_save_tree(const wchar_t *path) {
    DWORD attributes = GetFileAttributesW(path);
    if (attributes == INVALID_FILE_ATTRIBUTES) return;
    if (!(attributes & FILE_ATTRIBUTE_DIRECTORY) || (attributes & FILE_ATTRIBUTE_REPARSE_POINT)) {
        DeleteFileW(path);
        return;
    }
    wchar_t search[32768], child[32768];
    if (wide_join(search, sizeof(search) / sizeof(*search), path, L"*")) {
        WIN32_FIND_DATAW entry;
        HANDLE find = FindFirstFileW(search, &entry);
        if (find != INVALID_HANDLE_VALUE) {
            do {
                if (!wcscmp(entry.cFileName, L".") || !wcscmp(entry.cFileName, L"..")) continue;
                if (wide_join(child, sizeof(child) / sizeof(*child), path, entry.cFileName))
                    remove_save_tree(child);
            } while (FindNextFileW(find, &entry));
            FindClose(find);
        }
    }
    RemoveDirectoryW(path);
}

static bool ansi_path(const wchar_t *path, char *buf, size_t size) {
    BOOL lossy = FALSE;
    int len = WideCharToMultiByte(CP_ACP, WC_NO_BEST_FIT_CHARS, path, -1, buf, (int)size, NULL, &lossy);
    if (len > 0 && !lossy) return true;
    wchar_t short_path[32768];
    DWORD short_len = GetShortPathNameW(path, short_path, sizeof(short_path) / sizeof(*short_path));
    if (!short_len || short_len >= sizeof(short_path) / sizeof(*short_path)) return false;
    len = WideCharToMultiByte(CP_ACP, WC_NO_BEST_FIT_CHARS, short_path, -1, buf, (int)size, NULL, &lossy);
    return len > 0 && !lossy;
}

static bool use_legacy_user_dir(const wchar_t *dir, char *buf, size_t size) {
    if (!CreateDirectoryW(dir, NULL) && GetLastError() != ERROR_ALREADY_EXISTS) return false;
    DWORD attributes = GetFileAttributesW(dir);
    if (attributes == INVALID_FILE_ATTRIBUTES || !(attributes & FILE_ATTRIBUTE_DIRECTORY)) return false;
    return ansi_path(dir, buf, size);
}

// Save beside the game for portability. Existing AppData saves are copied once and kept as a backup.
bool plat_user_dir(char *buf, size_t size) {
    wchar_t local[32768], local_save[32768], legacy[32768], legacy_save[32768];
    DWORD local_len = GetCurrentDirectoryW(sizeof(local) / sizeof(*local), local);
    bool have_local = local_len && local_len < sizeof(local) / sizeof(*local) &&
                      wide_join(local_save, sizeof(local_save) / sizeof(*local_save), local, L"save");
    bool have_legacy = legacy_user_dir(legacy, sizeof(legacy) / sizeof(*legacy));
    have_legacy = have_legacy && wide_join(legacy_save, sizeof(legacy_save) / sizeof(*legacy_save), legacy, L"save");

    if (have_local && GetFileAttributesW(local_save) == INVALID_FILE_ATTRIBUTES) {
        DWORD legacy_attributes = have_legacy ? GetFileAttributesW(legacy_save) : INVALID_FILE_ATTRIBUTES;
        if (legacy_attributes != INVALID_FILE_ATTRIBUTES && (legacy_attributes & FILE_ATTRIBUTE_DIRECTORY)) {
            wchar_t temporary[32768];
            bool copied = false;
            for (unsigned attempt = 0; attempt < 8; attempt++) {
                int n = _snwprintf(temporary, sizeof(temporary) / sizeof(*temporary), L"%ls.migration-%lu-%u",
                                   local_save, GetCurrentProcessId(), attempt);
                if (n < 0 || (size_t)n >= sizeof(temporary) / sizeof(*temporary)) break;
                if (GetFileAttributesW(temporary) != INVALID_FILE_ATTRIBUTES) continue;
                if (!copy_save_tree(legacy_save, temporary)) {
                    remove_save_tree(temporary);
                    break;
                }
                if (MoveFileExW(temporary, local_save, MOVEFILE_WRITE_THROUGH)) {
                    copied = true;
                    break;
                }
                remove_save_tree(temporary);
                break;
            }
            if (!copied) {
                fprintf(stderr, "[save] could not copy existing saves beside the game; continuing to use AppData\n");
                return use_legacy_user_dir(legacy, buf, size);
            }
            fprintf(stderr, "[save] copied existing saves beside the game; the AppData backup was left untouched\n");
        }
        if (!CreateDirectoryW(local_save, NULL) && GetLastError() != ERROR_ALREADY_EXISTS) {
            fprintf(stderr, "[save] could not create a portable save folder; continuing to use AppData\n");
            return have_legacy && use_legacy_user_dir(legacy, buf, size);
        }
    } else if (have_local) {
        DWORD attributes = GetFileAttributesW(local_save);
        if (!(attributes & FILE_ATTRIBUTE_DIRECTORY)) {
            fprintf(stderr, "[save] portable save path is not a folder; continuing to use AppData\n");
            return have_legacy && use_legacy_user_dir(legacy, buf, size);
        }
    }

    if (!have_local) return have_legacy && use_legacy_user_dir(legacy, buf, size);
    if (strlen(".") + 1 > size) return false;
    strcpy(buf, ".");
    return true;
}

static int count_files(const char *dir, int depth) {
    char pat[MAX_PATH * 2];
    if (depth > 32 || snprintf(pat, sizeof pat, "%s\\*", dir) >= (int)sizeof pat) return 0;
    WIN32_FIND_DATAA fd;
    HANDLE h = FindFirstFileExA(pat, FindExInfoBasic, &fd, FindExSearchNameMatch, NULL, FIND_FIRST_EX_LARGE_FETCH);
    if (h == INVALID_HANDLE_VALUE) return 0;
    int n = 0;
    do {
        if (fd.cFileName[0] == '.') continue;
        if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)) { n++; continue; }
        char sub[MAX_PATH * 2];
        if (snprintf(sub, sizeof sub, "%s\\%s", dir, fd.cFileName) < (int)sizeof sub) n += count_files(sub, depth + 1);
    } while (FindNextFileA(h, &fd));
    FindClose(h);
    return n;
}
int plat_count_files(const char *dir) { return count_files(dir, 0); }

void plat_error_box(const char *msg) { MessageBoxA(NULL, msg, "MHGU Recomp", MB_OK | MB_ICONERROR); }

void plat_fatal_notice(void) {
    if (g_own_log)
        plat_error_box("The game stopped because of an error. The details are in log.txt, next to mhgurecomp.exe.");
}

// ---- diagnostics
const char *plat_symbol(uint64_t pc, uint64_t *offset) {
    static __thread char name[MAX_PATH];
    HMODULE m;
    if (!GetModuleHandleExA(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                            (LPCSTR)(uintptr_t)pc, &m))
        return NULL;
    if (m == GetModuleHandleA(NULL)) return NULL;
    char path[MAX_PATH];
    if (!GetModuleFileNameA(m, path, sizeof path)) return NULL;
    const char *b = strrchr(path, '\\');
    snprintf(name, sizeof name, "%s", b ? b + 1 : path);
    if (offset) *offset = pc - (uint64_t)(uintptr_t)m;
    return name;
}

const char *rt_host_fn_name(uint64_t pc, uint64_t *offset);  // runtime/dispatch.c

void plat_print_backtrace(void) {
    void *bt[64];
    USHORT n = RtlCaptureStackBackTrace(0, 64, bt, NULL);
    uint64_t exe = (uint64_t)(uintptr_t)GetModuleHandleA(NULL);
    for (int i = 0; i < n; i++) {
        uint64_t pc = (uint64_t)(uintptr_t)bt[i], off = 0;
        const char *s = rt_host_fn_name(pc, &off);
        if (!s) s = plat_symbol(pc, &off);
        if (s) fprintf(stderr, "  #%-2d %016llx %s+%llx\n", i, (unsigned long long)pc, s, (unsigned long long)off);
        else fprintf(stderr, "  #%-2d %016llx mhgurecomp+%llx\n", i, (unsigned long long)pc,
                     (unsigned long long)(pc - exe));
    }
}

static LONG WINAPI unhandled_filter(EXCEPTION_POINTERS *ep) {
    EXCEPTION_RECORD *er = ep->ExceptionRecord;
    uint64_t fa = er->NumberParameters >= 2 ? er->ExceptionInformation[1] : 0;
    char what[48];
    snprintf(what, sizeof what, "exception %08lx", er->ExceptionCode);
    uint64_t lr = 0;
    MEMORY_BASIC_INFORMATION mi;
    if (VirtualQuery((void *)(uintptr_t)ep->ContextRecord->Rsp, &mi, sizeof mi) && mi.State == MEM_COMMIT)
        lr = *(uint64_t *)(uintptr_t)ep->ContextRecord->Rsp;
    rt_fault_report(what, fa, ep->ContextRecord->Rip, lr);
}

void plat_crash_handler_install(void) {
    ULONG guarantee = 64 << 10;
    SetThreadStackGuarantee(&guarantee);
    static bool installed;
    if (installed) return;
    installed = true;
    SetUnhandledExceptionFilter(unhandled_filter);
}

// ---- POSIX shims (win_compat.h)
static ssize_t prw(int fd, void *buf, size_t n, off_t off, bool write) {
    HANDLE h = (HANDLE)_get_osfhandle(fd);
    if (h == INVALID_HANDLE_VALUE) return -1;
    size_t done = 0;
    while (done < n) {
        OVERLAPPED ov = {0};
        uint64_t o = (uint64_t)off + done;
        ov.Offset = (DWORD)o;
        ov.OffsetHigh = (DWORD)(o >> 32);
        DWORD chunk = n - done > 0x40000000 ? 0x40000000 : (DWORD)(n - done), got = 0;
        BOOL ok = write ? WriteFile(h, (const char *)buf + done, chunk, &got, &ov)
                        : ReadFile(h, (char *)buf + done, chunk, &got, &ov);
        if (!ok) {
            if (GetLastError() == ERROR_HANDLE_EOF) break;
            return done ? (ssize_t)done : -1;
        }
        if (!got) break;
        done += got;
    }
    return (ssize_t)done;
}
ssize_t pread(int fd, void *buf, size_t n, off_t off) { return prw(fd, buf, n, off, false); }
ssize_t pwrite(int fd, const void *buf, size_t n, off_t off) { return prw(fd, (void *)buf, n, off, true); }
