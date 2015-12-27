extern crate portaudio;

pub mod error;
pub mod math;

use std::sync::Arc;
use std::sync::RwLock;

use portaudio::pa;

use error::Error;

const SAMPLE_RATE: f32 = 44100.0;

const SAMPLE_FORMAT: pa::SampleFormat = pa::SampleFormat::Float32;
pub type Sample = f32;

pub type Stream = pa::Stream<Sample, Sample>;
pub type Callback = pa::StreamCallbackFn<Sample, Sample>;

struct SynthState {
  stream: Stream,
  index: usize
}

pub struct Synth {
  buffer_size: u32,
  state: Arc<RwLock<SynthState>>,
  stream_params: pa::StreamParameters
}

impl Synth {
  pub fn new(buffer_size: u32) -> Result<Synth, Error> {
    try!(pa::initialize());
    
    let (_output_name, stream_params, stream) =
      try!(Self::init_audio(SAMPLE_RATE as f64));
    
    let state = Arc::new(RwLock::new(
      SynthState {
        stream: stream,
        index: 0
      }
    ));
    
    let callback_state = state.clone();
    let callback = Box::new(move |
      _input: &[Sample],
      output: &mut[Sample],
      _frames: u32,
      _time_info: &pa::StreamCallbackTimeInfo,
      _flags: pa::StreamCallbackFlags
    | -> pa::StreamCallbackResult {
      let mut state = callback_state.write().unwrap();
            
      for output_sample in output.iter_mut() {
        let (ch, ch_index) = (state.index % 2, state.index / 2);
        
        let time = ch_index as f32 / SAMPLE_RATE;
        let fundamental = 140.0;
        
        let wave1 = {
          let amp = math::scale((-1.0, 1.0), (0.1, 0.5), 
            math::sine(2.11, 0.0, time)
          );
          let pan = 0.5 * math::sine(0.21, 0.25*math::TAU, time);
          let ch_amp = match (ch, math::pan(pan)) {
            (0, (l, _)) => l,
            (_, (_, r)) => r
          };
          ch_amp * amp * math::sine(fundamental, 0.0, time)
        };
        
        let wave2 = {
          let amp = math::scale((-1.0, 1.0), (0.1, 0.3), 
            math::sine(2.3, 0.5*math::TAU, time)
          );
          let pan = 0.6 * math::sine(0.17, 0.0, time);
          let ch_amp = match (ch, math::pan(pan)) {
            (0, (l, _)) => l,
            (_, (_, r)) => r
          };
          ch_amp * amp * math::sine(1.5 * fundamental, 0.5*math::TAU, time)
        };
        
        let mix = wave1 + wave2;
        
        *output_sample = mix;
        
        state.index += 1;
      }

      pa::StreamCallbackResult::Continue
    });
    
    {
        let mut state = (&state).write().unwrap();
        try!(state.stream.open(
            None,
            Some(&stream_params),
            SAMPLE_RATE as f64,
            buffer_size,
            pa::StreamFlags::empty(),
            Some(callback)                
        ));
    }
    
    Ok(Synth {
      buffer_size: buffer_size,
      state: state,
      stream_params: stream_params
    })
  }
  
  fn init_audio(sample_rate: f64) -> Result<(String, pa::StreamParameters, Stream), Error> {
    try!(pa::initialize());

    let default_output = pa::device::get_default_output();
    let output_info = try!(pa::device::get_info(default_output));

    let stream_params = pa::StreamParameters {
      device:             default_output,
      channel_count:      2,
      sample_format:      SAMPLE_FORMAT,
      suggested_latency:  output_info.default_low_output_latency,
    };
    try!(pa::is_format_supported(None, Some(&stream_params), sample_rate));

    let stream = pa::Stream::new();
    
    Ok((output_info.name, stream_params, stream))
  }
  
  pub fn play(&self) -> Result<(), Error> {
    let mut state = self.state.write().unwrap();
    try!(state.stream.start());
    
    Ok(())
  }
}

impl Drop for Synth {
  fn drop(&mut self) {
    pa::terminate()
      .unwrap_or_else(|e| {
        println!("pa::terminate() failed: {:?}", e);
      });
  }
}