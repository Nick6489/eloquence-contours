@echo off
setlocal
cd /d "%~dp0"
rem Match the existing host's CRT setting, with no runtime redistribution needed.
set "RUSTFLAGS=-C target-feature=+crt-static"
for %%T in (i686-pc-windows-msvc x86_64-pc-windows-msvc) do (
    cargo build --release --target %%T --manifest-path Cargo.toml
    if errorlevel 1 exit /b 1
)
