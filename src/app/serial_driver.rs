use std::io::BufRead;
use anyhow::ensure;
use log::{info, debug, error};

#[derive(Debug, Default, Copy, Clone)]
pub struct Values {
  pub sample : u32,
  pub position : i32,
  pub tensile : i32,
}

pub struct SerialDriver {
  values : Values,
  wait_for_response : bool,
  serial_interface: Option<Box<dyn serialport::SerialPort>>,
}

impl SerialDriver {
  pub fn new() -> Self {
    Self {
      values : Values::default(),
      serial_interface : None,
      wait_for_response : false,
    }
  }

  pub fn set_serial(&mut self, serial_interface : Box<dyn serialport::SerialPort>) {
    self.serial_interface = Some( serial_interface);
  }


  pub fn start_home(&mut self) -> anyhow::Result<()> {
    ensure!(!self.wait_for_response, "Homing in progress");
    self.wait_for_response = true;
    self.send_message("G28\r\n")
  }

  pub fn jog(&mut self, delta: i32 ) -> anyhow::Result<()>  {
    let new_pos = i32::max( self.values.position + delta, 0 );
    let msg = format!("G0 X{new_pos}\r\n" );
    self.send_message(&msg)
  }
  

  pub fn update(&mut self) -> Option<Values> {
    
    if let Some(s) = self.serial_interface.as_deref_mut() {

      let mut reader = std::io::BufReader::new(s);

      loop {
          let mut line = String::new();
        
          let res = reader.read_line(&mut line);
    
          match res {
            Ok(n) => {
              debug!("RECV:{n}>{}", &line);
              let mut line_parts = line.split_whitespace();
    
                if let Some(tag) = line_parts.next() {
                  if tag.contains("X:") {
                    let val = &tag[2..];
                    self.values.position = val.parse().unwrap();
                  }
                }
    
                if let Some(tag) = line_parts.next() {
                  if tag.contains("T:") {
                    let val = &tag[2..];
                    self.values.tensile = val.parse().unwrap();
                }
                }
            },
            Err(e) => {
                if e.kind() != std::io::ErrorKind::TimedOut { 
                  error!("Got serial error {e}");
                  panic!("Got error: {e}") 
                }
                break;
            }
        } 
      }
      Some(self.values)
    } 
    else {
      None
    }
  }


  pub fn send_message(&mut self, msg : &str) -> anyhow::Result<()> {

    info!("SENDING MESSAGE:{}:", msg);
    let bts = msg.as_bytes();

    match self.serial_interface.as_deref_mut(){
      Some(s) => {
        let bytes_written = s.write(bts)?;
        debug!("Wrote: {bytes_written} bytes");
         s.flush()?;
         Ok(())
      },
      None => anyhow::bail!("No serial interface"),
    }
  }
}
