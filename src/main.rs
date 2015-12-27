extern crate fivier;

use std::io;

use fivier::error::Error;
use fivier::Synth;

fn read_line(input: &mut io::Stdin) -> String {
    let mut line_buf = "".to_string();
    let _ = input.read_line(&mut line_buf).unwrap();
    
    line_buf.trim_right().to_string()
}

fn main_result() -> Result<(), Error> {
  let synth = try!(Synth::new(256));
  try!(synth.play());
  
  let mut stdin = io::stdin();
  loop {
    let line = read_line(&mut stdin);
    match &line[..] {
      "q" => break,
      _ => println!("Unknown command")
    }
  }
  
  Ok(())
}

fn main() {
  main_result().unwrap();
}
