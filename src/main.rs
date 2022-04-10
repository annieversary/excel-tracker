use calamine::{open_workbook_auto, DataType, Range, Reader};
use mumuse::music::{common::find_letter_idx, note::Note};

mod frame;
use frame::Frame;
use resampling::stretch_frames;
mod io;
mod resampling;

struct Config {
    input: String,
    output: String,
    bpm: u16,
    sample_rate: u32,
    beat_length: f64,
}

fn main() {
    let config = Config {
        input: "test.xlsx".to_string(),
        output: "out.wav".to_string(),
        bpm: 120,
        sample_rate: 44100,
        beat_length: 1.0,
    };
    generate(&config)
}

struct Track {
    sample_path: String,
    notes: Vec<Option<Note>>,
    // TODO effects or smth
}
fn generate(config: &Config) {
    let tracks = parse(config)
        .iter()
        .map(|t| render_track(t, config))
        .collect::<Vec<_>>();

    let render = join_tracks(tracks);

    save_file(render, config);
}

fn render_track(track: &Track, config: &Config) -> Vec<Frame> {
    let sample = io::open_file(&track.sample_path, config.sample_rate);

    let bps = config.bpm as f64 / 60.;
    let beat = (config.sample_rate as f64 * config.beat_length / bps) as usize;

    let mut vec = vec![];
    for (i, note) in track.notes.iter().enumerate() {
        if let Some(n) = note {
            let p = note_to_pitch(n);
            let factor = 261.63 / p;
            let sample = stretch_frames(&sample, factor);

            let start = i * beat;
            vec = add_vecs_starting_from(vec, start, &sample);
        }
    }

    vec
}

fn note_to_pitch(note: &Note) -> f64 {
    let i = find_letter_idx(note.letter);
    let midi = note.octave * 12 + i - 57;

    let a = (midi as f64 / 12.).exp2();
    440. * a
}

/// Adds two vectors, but starts `other` from `start`
fn add_vecs_starting_from(mut base: Vec<Frame>, start: usize, other: &[Frame]) -> Vec<Frame> {
    let end = start + other.len(); // Not the actual end, just of other
    let new_len = base.len().max(end);

    base.resize(new_len, Frame::default());

    for i in start..end {
        base[i] += other[i - start];
    }

    base
}

fn save_file(render: Vec<Frame>, config: &Config) {
    io::save_file(&render, &config.output, config.sample_rate, 32);
}

fn parse(config: &Config) -> Vec<Track> {
    let mut workbook = open_workbook_auto(&config.input).expect("Cannot open file");
    let sheets = workbook.worksheets();
    let (_name, sheet) = sheets.first().expect("No sheets were found");
    let height = sheet.height();

    let sample_cols = parse_sample_columns(sheet);
    let mut sample_notes = vec![];
    for (sample, col) in sample_cols {
        let notes: Vec<_> = (1..=height)
            .map(|row| {
                if let Some(DataType::String(v)) = &sheet.get((row, col)) {
                    Note::try_from(v.as_str()).ok()
                } else {
                    None
                }
            })
            .collect();
        sample_notes.push(Track {
            sample_path: sample,
            notes,
        });
    }

    sample_notes
}

fn parse_sample_columns(sheet: &Range<DataType>) -> Vec<(String, usize)> {
    let width = sheet.width();

    // TODO ideally, instead of just leaving a blank col, we check what stuff is a sample
    // then check the in between for params
    // so like:
    // audio.wav | volume | arp | other.wav | attack

    let mut r = vec![];
    for i in 0..=(width / 2) {
        if let Some(DataType::String(v)) = sheet.get((0, i * 2)) {
            r.push((v.clone(), i * 2));
        }
    }

    r
}

pub fn join_tracks(tracks: Vec<Vec<Frame>>) -> Vec<Frame> {
    // Get the max length of the tracks
    let len = &tracks
        .iter()
        .map(|track| track.len())
        .max()
        .expect("There should be at least one track to join");

    (0..*len)
        .map(|i| {
            let mut val = Frame::default();
            for track in &tracks {
                if let Some(value) = track.get(i) {
                    val += value;
                }
            }
            val
        })
        .collect()
}
