use rodio::{OutputStream, OutputStreamHandle, Source, SpatialSink};

use crate::audio_source::{self, Decodable};

pub struct AudioPlayer {
    stream_handle: Option<OutputStreamHandle>,
    left_ear: [f32; 3],
    right_ear: [f32; 3],
    emitter_translation: [f32; 3],
}

impl Default for AudioPlayer {
    fn default() -> Self {
        let emitter_translation = [0.0, 0.0, 0.0];
        let (left_ear, right_ear) = ([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            std::mem::forget(stream);
            Self {
                stream_handle: Some(stream_handle),
                left_ear,
                right_ear,
                emitter_translation,
            }
        } else {
            println!("找不到输出设备!");
            Self {
                stream_handle: None,
                left_ear,
                right_ear,
                emitter_translation,
            }
        }
    }
}

//TODO: 这里暂停和恢复有问题，目前看来还是需要单开线程去做
impl AudioPlayer {
    pub fn play(&self, audio_source: &audio_source::AudioSource) {
        let sink = self.create().unwrap();
        sink.append(audio_source.decoder().repeat_infinite());
        sink.detach();
    }

    pub fn pause(&self) {
        let sink = self.create().unwrap();
        sink.pause();
        sink.detach();
    }

    pub fn resume(&self, audio_source: &audio_source::AudioSource) {
        let sink = self.create().unwrap();
        if sink.is_paused() {
            println!("111");
            sink.play();
        }else{
            sink.append(audio_source.decoder().repeat_infinite());
        }
        sink.detach();
    }

    pub fn set_speed(&self, speed: f32) {
        let sink = self.create().unwrap();
        sink.set_speed(speed);
        sink.detach();
    }

    fn create(&self) -> Option<SpatialSink> {
        let Some(stream_handle) = self.stream_handle.as_ref() else {
            return None;
        };
        match SpatialSink::try_new(
            stream_handle,
            self.emitter_translation,
            self.left_ear,
            self.right_ear,
        ) {
            Ok(sink) => Some(sink),
            Err(err) => {
                println!("创建音轨失败: {err:?}");
                None
            }
        }
    }
}
