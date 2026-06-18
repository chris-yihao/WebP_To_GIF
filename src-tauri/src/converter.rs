use std::fs;
use std::path::Path;

use gif::{ColorOutput, DecodeOptions, Encoder as GifEncoder, Frame as GifFrame, Repeat};
use thiserror::Error;
use webp_animation::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionSummary {
    pub frame_count: usize,
    pub total_delay_cs: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GifInspection {
    pub frame_count: usize,
    pub total_delay_cs: u32,
}

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("无法读取文件：{0}")]
    Io(#[from] std::io::Error),
    #[error("无法读取 WebP 动画：{0}")]
    WebP(String),
    #[error("无法写入 GIF：{0}")]
    Gif(#[from] gif::EncodingError),
    #[error("无法检查 GIF：{0}")]
    GifDecode(#[from] gif::DecodingError),
    #[error("WebP 没有可转换的画面")]
    EmptyAnimation,
    #[error("图片尺寸太大，无法写入 GIF")]
    ImageTooLarge,
}

pub fn convert_webp_to_gif(source: &Path, output: &Path) -> Result<ConversionSummary, ConvertError> {
    let buffer = fs::read(source)?;
    let decoder = Decoder::new(&buffer).map_err(|error| ConvertError::WebP(error.to_string()))?;
    let (width, height) = decoder.dimensions();
    let width_u16 = u16::try_from(width).map_err(|_| ConvertError::ImageTooLarge)?;
    let height_u16 = u16::try_from(height).map_err(|_| ConvertError::ImageTooLarge)?;
    let frames: Vec<_> = decoder.into_iter().collect();

    if frames.is_empty() {
        return Err(ConvertError::EmptyAnimation);
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(output)?;
    let mut encoder = GifEncoder::new(&mut file, width_u16, height_u16, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    let mut previous_timestamp = 0;
    let mut total_delay_cs = 0;

    for frame in frames.iter() {
        let timestamp = frame.timestamp().max(previous_timestamp);
        let delay_ms = (timestamp - previous_timestamp).max(20);
        let delay_cs = ((delay_ms as f32) / 10.0).round().max(2.0) as u16;
        previous_timestamp = timestamp;
        total_delay_cs += u32::from(delay_cs);

        let mut rgba = frame.data().to_vec();
        let mut gif_frame = GifFrame::from_rgba_speed(width_u16, height_u16, &mut rgba, 10);
        gif_frame.delay = delay_cs;
        encoder.write_frame(&gif_frame)?;
    }

    drop(encoder);
    drop(file);

    Ok(ConversionSummary {
        frame_count: frames.len(),
        total_delay_cs,
    })
}

pub fn inspect_gif(path: &Path) -> Result<GifInspection, ConvertError> {
    let file = fs::File::open(path)?;
    let mut options = DecodeOptions::new();
    options.set_color_output(ColorOutput::RGBA);
    let mut decoder = options.read_info(file)?;
    let mut frame_count = 0;
    let mut total_delay_cs = 0;

    while let Some(frame) = decoder.read_next_frame()? {
        frame_count += 1;
        total_delay_cs += u32::from(frame.delay);
    }

    Ok(GifInspection {
        frame_count,
        total_delay_cs,
    })
}
