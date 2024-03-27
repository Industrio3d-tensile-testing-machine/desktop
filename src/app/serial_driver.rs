use anyhow::ensure;

#[derive(Debug, Default, Copy, Clone)]
pub struct Values {
  pub tensile : i32,
  pub position : i32,
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


  
  /// Start the homing procedure
  ///
  /// # Errors
  ///
  /// This function will return an error if homing is already in progress, or if 
  /// can't send the message using the serial_interface
  pub fn start_home(&mut self) -> anyhow::Result<()> {
    ensure!(self.wait_for_response, "Homing in progress");
    self.wait_for_response = true;
    self.send_message("G28\n".as_bytes())
  }

  /// Checks for new serial data, updates its state
  /// and Returns the update of this [`SerialDriver`].
  pub fn update(&mut self) -> Option<Values> {
    
    if let Some(_s) = self.serial_interface.as_deref_mut() {
      // todo read and parse serial result and update values
      // check if homing is done, then reset wait_for_response
      Some(self.values)
    } 
    else {
      None
    }
  }


  pub fn send_message(&mut self, msg : &[u8]) -> anyhow::Result<()> {
    match self.serial_interface.as_deref_mut(){
      Some(s) => {
         match s.write(msg) {
          Ok(n) => Ok(println!("Wrote: {n} bytes")),
          Err(e) => anyhow::bail!("Error writing to serial interface"), 
         }
      },
      None => anyhow::bail!("No serial interface"),
    }
  }
}
