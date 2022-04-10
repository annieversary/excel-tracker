use crate::frame::*;
use crate::resampling::resample_frames;
use hound::SampleFormat;
use hound::WavReader;
use hound::WavWriter;
use std::fs::File;
use std::io::BufReader;

pub fn open_file(path: &str, sample_rate: u32) -> Vec<Frame> {
    let reader =
        hound::WavReader::open(path).unwrap_or_else(|_| panic!("File {} should exist", path));
    let spec = reader.spec();

    // Check if the file has the same sample rate as the song
    // If it doesn't we resample the file
    // If it does, we just return the file
    if spec.sample_rate != sample_rate {
        // Otherwise we resample it, save it as a new file, and return it
        let spec = reader.spec();
        let orig = transform_samples_to_frames(
            reader,
            spec.channels,
            spec.sample_format,
            spec.bits_per_sample,
        );

        resample_frames(orig, spec.sample_rate, sample_rate)
    } else {
        transform_samples_to_frames(
            reader,
            spec.channels,
            spec.sample_format,
            spec.bits_per_sample,
        )
    }
}

fn transform_samples_to_frames(
    samples: WavReader<BufReader<File>>,
    num_channels: u16,
    sample_format: SampleFormat,
    bits_per_sample: u16,
) -> Vec<Frame> {
    match sample_format {
        SampleFormat::Float => samples
            .into_samples::<f32>()
            .map(Result::unwrap)
            .map(f64::from)
            .collect::<Vec<f64>>()
            .chunks(num_channels.into())
            .map(|sample| match sample {
                [left, right] => Frame::new(*left, *right),
                [a, ..] => Frame::mono(*a),
                [] => panic!("Sample has 0 channels"),
            })
            .collect::<Vec<Frame>>(),
        SampleFormat::Int => samples
            .into_samples::<i32>()
            .map(Result::unwrap)
            .map(|val| i_to_f(val, bits_per_sample))
            .collect::<Vec<f64>>()
            .chunks(num_channels.into())
            .map(|sample| match sample {
                [left, right] => Frame::new(*left, *right),
                [a, ..] => Frame::mono(*a),
                [] => panic!("Sample has 0 channels"),
            })
            .collect::<Vec<Frame>>(),
    }
}

pub fn save_file(audio: &[Frame], path: &str, sample_rate: u32, bits_per_sample: u16) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)
        .unwrap_or_else(|_| panic!("File could not be saved at {}", path));

    for val in audio {
        let val = val.clamp(-1., 1.);

        let left = f_to_i(val.left, bits_per_sample);
        writer
            .write_sample(left)
            .expect("Frame's left value could not be written");
        let right = f_to_i(val.right, bits_per_sample);
        writer
            .write_sample(right)
            .expect("Frame's right value could not be written");
    }
}

fn i_to_f(val: i32, bits_per_sample: u16) -> f64 {
    val as f64 / (1_usize << (bits_per_sample - 1)) as f64
}

fn f_to_i(val: f64, bits_per_sample: u16) -> i32 {
    (val * (1_usize << (bits_per_sample - 1)) as f64) as i32
}
