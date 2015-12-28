extern crate portaudio;

pub mod error;
pub mod math;

use std::f32;
use std::sync::Arc;
use std::sync::RwLock;

use portaudio::pa;

use error::Error;

const SAMPLE_RATE: f32 = 44100.0;

const SAMPLE_FORMAT: pa::SampleFormat = pa::SampleFormat::Float32;
pub type Sample = f32;

pub type Stream = pa::Stream<Sample, Sample>;
pub type Callback = pa::StreamCallbackFn<Sample, Sample>;

pub struct Sine {
  phase_inc: f32,
  phase: f32
}

impl Sine {
  pub fn new(frequency: f32, phase: f32) -> Sine {
    let mut sine = Sine {
      phase_inc: 0.0,
      phase: phase % math::TAU
    };
    sine.set_frequency(frequency);
    
    sine
  }
  
  pub fn read(&self) -> f32 {
    f32::sin(self.phase)
  }
  
  pub fn set_frequency(&mut self, frequency: f32) {
    self.phase_inc = math::TAU / SAMPLE_RATE * frequency;
  }
  
  pub fn advance(&mut self) {        
    self.phase = (self.phase + self.phase_inc) % math::TAU;
  }
}

struct Delay {
  buffer: Vec<Sample>,
  index: usize
}

impl Delay {
  pub fn new(size: usize) -> Self {
    let stereo_size = 2 * size;
    let mut buffer = Vec::with_capacity(stereo_size);
    for _ in 0 .. stereo_size {
      buffer.push(0.0);
    }
    
    Delay {
      buffer: buffer,
      index: 0
    }
  }
  
  pub fn read(&self) -> Sample {
    self.buffer[self.index]
  }
  
  pub fn advance(&mut self, input: Sample) {
    self.buffer[self.index] = input;
    self.index = (self.index + 1) % self.buffer.len();
  }
}

struct SynthState {
  stream: Stream,
  channel: u32,
  
  sine_wave1: Sine,
  sine_wave1_freq: Sine,
  sine_wave1_amp: Sine,
  sine_wave1_pan: Sine,
  
  sine_wave2: Sine,
  sine_wave2_freq: Sine,
  sine_wave2_amp: Sine,
  sine_wave2_pan: Sine,
  
  delay: Delay
}

pub struct Synth {
  buffer_size: u32,
  state: Arc<RwLock<SynthState>>,
  stream_params: pa::StreamParameters
}

impl Synth {
  pub fn new(buffer_size: u32) -> Result<Self, Error> {
    let (_output_name, stream_params, stream) =
      try!(Self::init_audio(SAMPLE_RATE as f64));
    
    let fundamental = 140.0;
    
    let delay_time = 2.2;
    let delay_buf_len = (delay_time * SAMPLE_RATE) as usize;
    let state = Arc::new(RwLock::new(
      SynthState {
        stream: stream,
        channel: 0,
        
        sine_wave1: Sine::new(fundamental, 0.0),
        sine_wave1_freq: Sine::new(6.33, 0.0),
        sine_wave1_amp: Sine::new(2.11, 0.0),
        sine_wave1_pan: Sine::new(0.21, 0.25 * math::TAU),
        
        sine_wave2: Sine::new(1.5 * fundamental, 0.5 * math::TAU),
        sine_wave2_freq: Sine::new(6.12, 0.333 * math::TAU),
        sine_wave2_amp: Sine::new(2.3, 0.5 * math::TAU),
        sine_wave2_pan: Sine::new(0.17, 0.0),
        
        delay: Delay::new(delay_buf_len)
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
        let wave1 = {
          let freq = {
            let k = 1.07;
            math::scale(
              (-1.0, 1.0), (fundamental/k, fundamental*k),
              state.sine_wave1_freq.read()
            )
          };
          state.sine_wave1.set_frequency(freq);
          
          let wave = state.sine_wave1.read();
          let amp = math::scale(
            (-1.0, 1.0), (0.1, 0.5),
            state.sine_wave1_amp.read()
          );
          let pan = 0.5 * state.sine_wave1_pan.read();
          let ch_amp = match (state.channel, math::pan_constant_power(pan)) {
            (0, (l, _)) => l,
            (_, (_, r)) => r
          };

          ch_amp * amp * wave
        };
        
        let wave2 = {
          let freq = {
            let base = 1.5 * fundamental;
            let k = 1.07;
            math::scale(
              (-1.0, 1.0), (base/k, base*k),
              state.sine_wave2_freq.read()
            )
          };
          state.sine_wave2.set_frequency(freq);
          
          let wave = state.sine_wave2.read();
          let amp = math::scale(
            (-1.0, 1.0), (0.1, 0.5),
            state.sine_wave2_amp.read()
          );
          let pan = 0.6 * state.sine_wave2_pan.read();
          let ch_amp = match (state.channel, math::pan_constant_power(pan)) {
            (0, (l, _)) => l,
            (_, (_, r)) => r
          };

          ch_amp * amp * wave
        };
        
        let dry_mix = wave1 + wave2;
        let wet_mix = state.delay.read();
        
        *output_sample = 0.6*dry_mix + 0.4*wet_mix;
        
        state.channel = (state.channel + 1) % 2;
        
        if state.channel == 0 {
          state.sine_wave1.advance();
          state.sine_wave1_freq.advance();
          state.sine_wave1_amp.advance();
          state.sine_wave1_pan.advance();
          
          state.sine_wave2.advance();
          state.sine_wave2_freq.advance();
          state.sine_wave2_amp.advance();
          state.sine_wave2_pan.advance();
        }
        
        state.delay.advance(dry_mix);
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