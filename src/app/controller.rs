
// pub enum Axis {
//     X,
//     Y,
//     Z,
// }

// pub trait Controller {
//     fn jog_left(&mut self);
//     fn jog_right(&mut self);
//     fn jog(&mut self, axis: Axis);
//     fn home(&mut self);
// }

// impl Controller {
//     fn jog_left(&mut self) {
//         // Implementeer hier de logica voor jog naar links
//     }

//     fn jog_right(&mut self) {
//         // Implementeer hier de logica voor jog naar rechts
//     }

//     fn jog(&mut self, axis: Axis) {
//       let command = format "G0 {}"
//     }

//     fn home(&mut self) {
//         // Implementeer hier de logica voor home
//     }
// }

// use std::io::Write;

pub struct Controller {
  serial_interface: Option<Box<dyn serialport::SerialPort>>,
}


impl Controller {
  pub fn new() -> Self {
    Self {
      serial_interface : None
    }
  }

  pub fn sset_serial(&mut self, serial_interface : Box<dyn serialport::SerialPort>) {
    self.serial_interface = Some( serial_interface);
  }

  pub fn send_message(&mut self, msg : &str) {
  //  s
    
    // match self.serial_interface {
    //     Some(s) => {
    //       let x = s.as_mut();

    //       s.as_mut().write(output).expect("baaa")
    //     },
    //     None => todo!(),
    // }
    // if Some(ser) = self.serial_interface {
    //   ser.write()
    // }

  }
}


// https://github.com/OctoPrint/OctoPrint/blob/65a5533c5d645f7033f01056c8ef735a938339f0/src/octoprint/printer/standard.py#L480
// jog(axis, amount) --> G0 {axis}{amount}