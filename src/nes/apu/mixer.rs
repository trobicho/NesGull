use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

static mut stream_data: Vec<f32> = vec![];

pub struct Mixer {
  device: cpal::Device,
  config: cpal::StreamConfig,
  sample_format: cpal::SampleFormat,
  stream: Option<cpal::Stream>,
  mute: bool,
  volume: f32,
}

impl Mixer {
  pub fn new(device: cpal::Device, config: cpal::SupportedStreamConfig) -> Self {
    let mut mixer = Self {
      device,
      sample_format: config.sample_format(),
      config: config.into(),
      stream: None,
      mute: false,
      volume: 0.3,
    };
    mixer.config.sample_rate = cpal::SampleRate(44100);
    mixer
  }

  pub fn add_to_stream(&mut self, data: f32) {
    unsafe {
      stream_data.push(data * self.volume);
    }
  }

  pub fn set_mute(&mut self, m: bool) {
    if !m {
      unsafe {
        stream_data.clear();
      }
      match self.sample_format {
        cpal::SampleFormat::F32 => self.run::<f32>(),
        cpal::SampleFormat::I16 => self.run::<i16>(),
        cpal::SampleFormat::U16 => self.run::<u16>(),
      }
    }
    else {
      self.stream = None;
    }
  }

  fn run<T: cpal::Sample>(&mut self) {
    let sample_rate = self.config.sample_rate.0 as f32;
    let channels = self.config.channels as usize;

    let mut sample_clock = 0f32;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    unsafe {
      let mut next_value = move || {
        match stream_data.pop() {
          Some(f) => f,
          None => 0f32,
        }
      };
      self.stream = Some(self.device.build_output_stream(
        &self.config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
          Self::write_data(data, channels, &mut next_value)
        },
        err_fn,
      ).unwrap());
      if let Some(stream) = &self.stream {stream.play().unwrap();}
    }
  }

  fn write_data<T: cpal::Sample>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32) {
    for frame in output.chunks_mut(channels) {
      let value: T = cpal::Sample::from::<f32>(&next_sample());
      for sample in frame.iter_mut() {
        *sample = value;
      }
    }
  }
}
