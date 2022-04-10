use crate::frame::{Frame, GetSidesExtension};

use dasp::interpolate::linear::Linear;
use dasp::{signal, Signal};

/// Passing factor = 2 will make the audio length be twice as long
pub fn stretch(frames: Vec<f64>, factor: f64) -> Vec<f64> {
    let len = (frames.len() as f64 * factor) as usize;

    let mut source = signal::from_iter(frames);
    let a = source.next();
    let b = source.next();
    let interp = Linear::new(a, b);

    source.scale_hz(interp, 1. / factor).take(len).collect()
}

/// Passing factor = 2 will make the audio length be twice as long
pub fn stretch_frames(frames: &[Frame], factor: f64) -> Vec<Frame> {
    // Get each side
    let (left, right) = frames.split_sides();

    // Resample them
    let left = stretch(left, factor);
    let right = stretch(right, factor);

    // Join them together
    join_left_and_right_channels(left, right)
}

pub fn resample(frames: Vec<f64>, old_sample_rate: u32, new_sample_rate: u32) -> Vec<f64> {
    stretch(frames, new_sample_rate as f64 / old_sample_rate as f64)
}

pub fn resample_frames(
    frames: Vec<Frame>,
    old_sample_rate: u32,
    new_sample_rate: u32,
) -> Vec<Frame> {
    // Get each side
    let (left, right) = frames.split_sides();

    // Resample them
    let left = resample(left, old_sample_rate, new_sample_rate);
    let right = resample(right, old_sample_rate, new_sample_rate);

    // Join them together
    join_left_and_right_channels(left, right)
}

pub fn join_left_and_right_channels(left: Vec<f64>, right: Vec<f64>) -> Vec<Frame> {
    left.iter()
        .zip(right.iter())
        .map(|(l, r)| Frame::new(*l, *r))
        .collect()
}
