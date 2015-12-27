use ::portaudio::pa;

#[derive(Debug)]
pub enum Error {
  PaError(pa::Error),
  FivierError(String)
}

impl From<pa::Error> for Error {
  fn from(err: pa::Error) -> Error {
    Error::PaError(err)
  }
}
