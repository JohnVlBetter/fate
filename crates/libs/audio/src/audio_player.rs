use rodio::{OutputStream, OutputStreamHandle, Source, SpatialSink};

use crate::audio_source::{self, Decodable};

pub struct AudioPlayer {
    stream_handle: Option<OutputStreamHandle>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            std::mem::forget(stream);
            Self {
                stream_handle: Some(stream_handle),
            }
        } else {
            println!("找不到输出设备!");
            Self {
                stream_handle: None,
            }
        }
    }
}

impl AudioPlayer {
    pub fn play(&self, audio_source: audio_source::AudioSource) {
        let Some(stream_handle) = self.stream_handle.as_ref() else {
            return;
        };
        let emitter_translation = [0.0, 0.0, 0.0];
        let (left_ear, right_ear) = ([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        let sink =
            match SpatialSink::try_new(stream_handle, emitter_translation, left_ear, right_ear) {
                Ok(sink) => sink,
                Err(err) => {
                    println!("Error creating spatial sink: {err:?}");
                    return;
                }
            };
        sink.set_speed(1.0);
        sink.set_volume(1.0);
        sink.append(audio_source.decoder().repeat_infinite());
    }
}
