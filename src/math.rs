use std::f32;

pub const TAU: f32 = 2.0 * f32::consts::PI;

pub fn scale((min0, max0): (f32, f32), (min1, max1): (f32, f32), value: f32) -> f32 {
  (value - min0) / (max0 - min0) * (max1 - min1) + min1
}

pub fn sine(frequency: f32, phase: f32, time: f32) -> f32 {
  f32::sin(frequency * time * TAU + phase)
}

pub fn pan(balance: f32) -> (f32, f32) {
  let balance_norm = scale(
    (-1.0, 1.0), (0.0, 1.0),
    f32::max(-1.0, f32::min(1.0, balance))
  );
  
  (1.0 - balance_norm, balance_norm)  
}
