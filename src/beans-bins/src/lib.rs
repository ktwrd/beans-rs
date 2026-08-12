use include_flate::flate;

#[cfg(target_os = "windows")]
flate!(pub static BUTLER_BINARY: [u8] from "data/butler.exe");
#[cfg(not(target_os = "windows"))]
flate!(pub static BUTLER_BINARY: [u8] from "data/butler");
#[cfg(target_os = "windows")]
flate!(pub static BUTLER_LIB_1: [u8] from "data/7z.dll");
#[cfg(not(target_os = "windows"))]
flate!(pub static BUTLER_LIB_1: [u8] from "data/7z.so");
#[cfg(target_os = "windows")]
flate!(pub static BUTLER_LIB_2: [u8] from "data/c7zip.dll");
#[cfg(not(target_os = "windows"))]
flate!(pub static BUTLER_LIB_2: [u8] from "data/libc7zip.so");
#[cfg(target_os = "windows")]
flate!(pub static ARIA2C_BINARY: [u8] from "data/aria2c.exe");
