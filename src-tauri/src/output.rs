use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("无法创建输出文件夹：{0}")]
    Io(#[from] std::io::Error),
    #[error("源文件没有有效文件名")]
    MissingFileName,
}

pub fn default_output_dir_for_executable(executable: &Path) -> PathBuf {
    executable
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("GIF")
}

pub fn ensure_output_dir(output_dir: &Path) -> Result<PathBuf, OutputError> {
    fs::create_dir_all(output_dir)?;
    Ok(output_dir.to_path_buf())
}

pub fn is_webp_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("webp"))
        .unwrap_or(false)
}

pub fn unique_gif_path(source: &Path, output_dir: &Path) -> Result<PathBuf, OutputError> {
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or(OutputError::MissingFileName)?;

    let mut candidate = output_dir.join(format!("{stem}.gif"));
    let mut index = 1;

    while candidate.exists() {
        candidate = output_dir.join(format!("{stem} ({index}).gif"));
        index += 1;
    }

    Ok(candidate)
}
