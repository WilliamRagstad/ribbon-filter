# Fuzz Targets

This directory contains cargo-fuzz harnesses for panic-resilience testing.

## Target

- `build_query`: exercises parameter parsing, builder construction, and membership queries across edge-sized parameter combinations.

## Run

```sh
cargo fuzz run build_query -- -max_total_time=60
```

## Notes

- The harness intentionally accepts both successful and failed constructions.
- A successful construction always performs several `contains_in` calls.
- Starter corpus is in `fuzz/corpus/build_query`.
- `cargo-fuzz` uses libFuzzer/sanitizer toolchains and may require additional platform-specific runtime/compiler components.
- On Windows/MSVC, sanitizer-linked fuzz binaries can fail to start if sanitizer runtime DLLs are unavailable.
- If this happens, run fuzzing in a Linux environment (native Linux, WSL, or CI container with clang/libfuzzer toolchain).

### Windows/MSVC PATH fix for AddressSanitizer runtime

If fuzz target execution fails with `0xc0000135 (STATUS_DLL_NOT_FOUND)`, your shell likely cannot find the MSVC AddressSanitizer runtime DLLs.

1. Install required Visual Studio components:
   - `MSVC v143 - VS 2022 C++ x64/x86 build tools`
   - `C++ AddressSanitizer`
2. Use `Developer PowerShell for VS 2022` (recommended), or add the MSVC bin directory to `PATH` manually:

```powershell
$msvc = "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC"
$latest = Get-ChildItem $msvc -Directory | Sort-Object Name -Descending | Select-Object -First 1
$env:PATH = "$($latest.FullName)\bin\Hostx64\x64;$env:PATH"
```

3. Verify the ASan runtime is discoverable:

```powershell
Get-Command clang_rt.asan_dynamic-x86_64.dll
```

4. Re-run fuzzing:

```powershell
cargo +nightly fuzz run build_query -- -max_total_time=60
```
