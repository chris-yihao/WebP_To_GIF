#![windows_subsystem = "windows"]

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const MAGIC: &[u8; 8] = b"WPGIFPK1";
const FOOTER_LEN: usize = 24;
const APP_NAME: &str = "webPToGif.exe";
const DLL_NAME: &str = "WebView2Loader.dll";

fn read_footer(data: &[u8]) -> io::Result<(usize, usize)> {
    if data.len() < FOOTER_LEN {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "portable payload is missing"));
    }

    let footer_start = data.len() - FOOTER_LEN;
    if &data[footer_start..footer_start + MAGIC.len()] != MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "portable payload is invalid"));
    }

    let mut app_len = [0_u8; 8];
    let mut dll_len = [0_u8; 8];
    app_len.copy_from_slice(&data[footer_start + 8..footer_start + 16]);
    dll_len.copy_from_slice(&data[footer_start + 16..footer_start + 24]);

    Ok((
        u64::from_le_bytes(app_len) as usize,
        u64::from_le_bytes(dll_len) as usize,
    ))
}

fn payload_dir(app_len: usize, dll_len: usize) -> PathBuf {
    env::temp_dir()
        .join("webptogif-portable")
        .join(format!("{app_len:x}-{dll_len:x}"))
}

fn run() -> io::Result<()> {
    let self_path = env::current_exe()?;
    let data = fs::read(&self_path)?;
    let (app_len, dll_len) = read_footer(&data)?;
    let footer_start = data.len() - FOOTER_LEN;
    let payload_start = footer_start
        .checked_sub(app_len + dll_len)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "portable payload is truncated"))?;
    let dll_start = payload_start + app_len;

    let out_dir = payload_dir(app_len, dll_len);
    fs::create_dir_all(&out_dir)?;

    let app_path = out_dir.join(APP_NAME);
    let dll_path = out_dir.join(DLL_NAME);
    fs::write(&app_path, &data[payload_start..dll_start])?;
    fs::write(&dll_path, &data[dll_start..footer_start])?;

    let args: Vec<String> = env::args().skip(1).collect();
    Command::new(app_path).args(args).spawn()?;

    Ok(())
}

fn main() {
    let _ = run();
}
