use std::f32;

pub const TAU: f32 = 2.0 * f32::consts::PI;

pub fn scale((min0, max0): (f32, f32), (min1, max1): (f32, f32), value: f32) -> f32 {
  (value - min0) / (max0 - min0) * (max1 - min1) + min1
}

pub fn sine(frequency: f32, phase: f32, time: f32) -> f32 {
  f32::sin(frequency * time * TAU + phase)
}

pub fn pan_linear(balance: f32) -> (f32, f32) {
  let p = scale(
    (-1.0, 1.0), (0.0, 1.0),
    f32::max(-1.0, f32::min(1.0, balance))
  );
  
  (1.0 - p, p)
}

pub fn pan_constant_power(balance: f32) -> (f32, f32) {
  let angle = scale(
    (-1.0, 1.0), (0.0, 0.25 * TAU),
    f32::max(-1.0, f32::min(1.0, balance))
  );
  
  (f32::cos(angle), f32::sin(angle))
}

pub fn clip_hard(threshold: f32, input: f32) -> f32 {
  if input < -threshold {
    -threshold
  } else if input > threshold {
    threshold
  } else {
    input
  }
}