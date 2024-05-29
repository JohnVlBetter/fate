use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use audio::audio_player::AudioPlayer;
use audio::audio_source::{AudioLoader, AudioSource};
use resource::resource_mgr::ResourceMgr;
use rodio::queue::SourcesQueueOutput;
use rodio::{Decoder, OutputStream, Sink, Source};

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    let sound_duration = Duration::from_millis(1000);

    let source = rodio::source::SineWave::new(1000.0).take_duration(sound_duration);
    sink.append(source);

    sink.sleep_until_end();

    /*  std::thread::sleep(std::time::Duration::from_secs(5));
    let mut mgr = ResourceMgr::new();
    mgr.register_loader(AudioLoader::default());
    let loader = mgr.get_asset_loader_with_extension("ogg");
    println!("{}", loader.unwrap().extensions()[0]);
    let resource = mgr.load(&Path::new("assets/audio/breakout_collision.ogg"));
    let binding = resource.unwrap();
    let resource = binding.as_any().downcast_ref::<AudioSource>().unwrap();
    println!("{}", resource.bytes.len());
    AudioPlayer::default().play(resource.clone());*/
}
