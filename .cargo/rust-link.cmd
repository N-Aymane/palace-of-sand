@echo off
for /f "delims=" %%i in ('rustc --print sysroot') do set "PALACE_SYSROOT=%%i"
"%PALACE_SYSROOT%\lib\rustlib\x86_64-pc-windows-gnu\bin\rust-lld.exe" -flavor gnu %*
