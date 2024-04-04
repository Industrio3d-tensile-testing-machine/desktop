use std::{io::BufRead, time::Duration};
use anyhow::ensure;
use serialport::{self, Error, SerialPortInfo};
use log::{info, debug, error};

#[derive(Debug, Default, Copy, Clone)]
pub struct Values {
  pub sample : u32,
  pub position : f32,
  pub tensile : i32,
}

pub struct SerialDriver {
  values : Values,
  acknowledge_pending : bool,
  is_homed: bool,
  serial_interface: Option<Box<dyn serialport::SerialPort>>,
}

impl SerialDriver {
  pub fn new() -> Self {
    Self {
      values : Values::default(),
      acknowledge_pending: false,
      is_homed: false,
      serial_interface: None,
    }
  }

  pub fn open(&mut self, serial_port: &String, baud_rate: u32) -> Result<(), Error> {
    let result = serialport::new(serial_port, baud_rate).timeout(Duration::from_millis(10)).open();

    self.serial_interface = Some(result?);

    Ok(())
  }

  pub fn close(&mut self) {
    drop(self.serial_interface.take());
    self.acknowledge_pending = false;
    self.is_homed = false;
  }

  pub fn available_ports(&mut self) -> anyhow::Result<Vec<SerialPortInfo>> {
    match serialport::available_ports() {
        Ok(res) => anyhow::Ok(res),
        Err(err) => anyhow::bail!(err.to_string()),
    }
  }

  pub fn start_home(&mut self) -> anyhow::Result<()> {
    ensure!(self.serial_interface.is_some(), "No serial interface initialized");
    ensure!(!self.acknowledge_pending, "Homing in progress");

    // if self.is_homed {
    //   anyhow::bail!("Already homed")
    // }

    self.acknowledge_pending = true;
    let res = self.send_message("G28\r\n");
    self.is_homed = true;
    res
  }

  pub fn jog(&mut self, delta: f32) -> anyhow::Result<()>  {
    ensure!(self.serial_interface.is_some(), "No serial interface initialized");
    ensure!(!self.acknowledge_pending, "Jog in progress");
    ensure!(self.is_homed, "Tensile tester not homed, please home first");

    let pos = f32::max( self.values.position + delta, 0.0 );
    // println!("current pos: {}, delta: {}, pos: {}", self.values.position, delta, pos);

    // debug!("new pos: {pos}");

    let msg = format!("G0 X{pos}\r\n" );
    self.send_message(&msg)
  }

  pub fn start_test(&mut self, speed: f64) -> anyhow::Result<()> {
    ensure!(self.serial_interface.is_some(), "No serial interface initialized");
    ensure!(!self.acknowledge_pending, "Test in progress");
    ensure!(self.is_homed, "Tensile tester not homed, please home first");

    self.acknowledge_pending = true;

    let s = speed as f32;
    let msg = format!("M700 S{s}\r\n" );
    let res = self.send_message(&msg);

    res
  }

  pub fn cancel_test(&mut self) {
    let _ = self.start_home();
  }

  pub fn update(&mut self) -> Option<Values> {
    if let Some(s) = self.serial_interface.as_deref_mut() {

      let mut reader = std::io::BufReader::new(s);

      loop {
          let mut line = String::new();
        
          let res = reader.read_line(&mut line);
    
          match res {
            Ok(n) => {
              // debug!("RECV:{n}>{}", &line);
              let mut line_parts = line.split_whitespace();
    
                if let Some(tag) = line_parts.next() {
                  if tag.contains("ok") {
                    self.acknowledge_pending = false;
                    break;
                  }

                  if tag.contains("X:") {
                    let val = &tag[2..];
                    self.values.position = val.parse().unwrap();
                  }
                }
    
                if let Some(tag) = line_parts.next() {
                  if tag.contains("T:") {
                    let val = &tag[2..];
                    self.values.tensile = val.parse().unwrap();
                    break;
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
    let bts = msg.as_bytes();

    match self.serial_interface.as_deref_mut() {
      Some(s) => {
        let bytes_written = s.write(bts)?;
         s.flush()?;
         Ok(())
      },
      None => anyhow::bail!("No serial interface"),
    }
  }

  pub fn is_acknowledge_pending(&self) -> bool {
    self.acknowledge_pending
  }

  pub fn is_homed(&self) -> bool {
    self.is_homed
  }

  pub fn values(&self) -> Values {
    self.values
  }
}
