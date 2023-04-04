use std::error::Error;
use hound;
use op_engine::{Clip, Session};

fn main() -> Result<(), Box<dyn Error>> {
    let mut session = Session::new();
    session.track.add_clip(Clip::from_file(0, "samples/hat1_mono.wav")?);
    session.track.add_clip(Clip::from_file(35097, "samples/hat1_mono.wav")?);
    session.track.add_clip(Clip::from_file(35097 + (35097/2), "samples/hat1_mono.wav")?);

    let result = session.render_all();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("out.wav", spec)?;

    for sample in result {
        writer.write_sample((sample * i16::MAX as f32) as i16)?;
    }

    writer.finalize()?;

    Ok(())
}
