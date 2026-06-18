use std::fs;

use tempfile::tempdir;
use webp_animation::prelude::*;
use webp_to_gif::converter::{convert_webp_to_gif, inspect_gif};

fn write_animated_webp(path: &std::path::Path) {
    let dimensions = (4, 4);
    let red = [255, 0, 0, 255].repeat(16);
    let blue = [0, 0, 255, 255].repeat(16);
    let mut encoder = Encoder::new(dimensions).unwrap();
    encoder.add_frame(&red, 0).unwrap();
    encoder.add_frame(&blue, 120).unwrap();
    let webp = encoder.finalize(240).unwrap();
    fs::write(path, webp).unwrap();
}

#[test]
fn animated_webp_converts_to_multi_frame_gif() {
    let base = tempdir().unwrap();
    let source = base.path().join("moving.webp");
    let output = base.path().join("moving.gif");
    write_animated_webp(&source);

    let summary = convert_webp_to_gif(&source, &output).unwrap();
    let inspected = inspect_gif(&output).unwrap();

    assert_eq!(summary.frame_count, 2);
    assert_eq!(inspected.frame_count, 2);
    assert!(inspected.total_delay_cs >= 20);
    assert!(output.exists());
}

#[test]
fn malformed_webp_returns_readable_error() {
    let base = tempdir().unwrap();
    let source = base.path().join("broken.webp");
    let output = base.path().join("broken.gif");
    fs::write(&source, b"not a webp").unwrap();

    let error = convert_webp_to_gif(&source, &output).unwrap_err();

    assert!(error.to_string().contains("无法读取 WebP 动画"));
    assert!(!output.exists());
}
