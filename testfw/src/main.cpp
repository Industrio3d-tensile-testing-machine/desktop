#include <Arduino.h>


void setup() {
  //Initialize serial and wait for port to open:
  Serial.begin(9600);
  while (!Serial) {
    ; // wait for serial port to connect. Needed for native USB port only
  }

  // prints title with ending line break
  Serial.println("Tensile Dummy V0.000002");
}

static long x_pos = 50;
static long to_pos = x_pos;
static long tensile = 0;


const byte recv_buffer_size = 32;
static char recv_buffer[recv_buffer_size];
static byte recv_index = 0;
static bool command_active = false;

static bool recv_line() {
  while (Serial.available() > 0) {
    int rchar = Serial.read();

    if ( rchar <= 0 ) {
      return false;
    }

    Serial.print("$");
    Serial.println(rchar);

    if (recv_index < recv_buffer_size - 1) {
      recv_buffer[recv_index] = (byte)rchar;
      recv_index += 1;
      recv_buffer[recv_index + 1] = 0;
    }
    
    if (rchar == 10) return true;
  }

  return false;
}


static unsigned int loop_idx = 0;

void loop() {

  
  if ( (loop_idx % 2000 ) == 0 ) {
    tensile = random(-100, 1000);
    Serial.print("X:" );
    Serial.print(x_pos);
    Serial.print(" T:");
    Serial.print(tensile);
    Serial.println();
  }

  if ( recv_line() ) {
      // Serial.print("LINE#");
      // Serial.print(recv_buffer);
      // Serial.println("#");

      if ( memcmp( recv_buffer, "M0" , 2 ) == 0 ) {
        Serial.println("#STOP#");
        command_active = false;
        to_pos = x_pos;
        Serial.println("ok");
      }

      if ( memcmp( recv_buffer, "G0 X" , 4 ) == 0 ) {

        Serial.print("#GOTO#");
        to_pos = atol(&recv_buffer[4]);
        Serial.println(to_pos);
        command_active = true;
      }
      if ( memcmp( recv_buffer, "G28" , 3 ) == 0 ) {
        Serial.println("#HOME#");
        command_active = true;
      }
      recv_index = 0;
      recv_buffer[0] = 0;
  }

  // if (x_pos > 0 && is_homing) {
  //   x_pos -= 1;
  // }

  if ( (loop_idx % 100 ) == 0 ) {
    if ( command_active && (x_pos != to_pos ) ) {
      if (x_pos > to_pos ) {
        x_pos -= 1;
      } else {
        x_pos += 1;
      }
      if (x_pos == to_pos ) {
        command_active = false;
        Serial.println("ok");
      }
    }
  }

  delay(1);

  loop_idx += 1;
}
