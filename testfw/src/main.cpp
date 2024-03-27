#include <Arduino.h>


void setup() {
  //Initialize serial and wait for port to open:
  Serial.begin(9600);
  while (!Serial) {
    ; // wait for serial port to connect. Needed for native USB port only
  }

  // prints title with ending line break
  Serial.println("Tensile Dummy V0.000001");
}

static long x_pos = 50;
static long tensile = 0;


const byte recv_buffer_size = 32;
static char recv_buffer[recv_buffer_size];
static byte recv_index = 0;
static bool is_homing = false;

static bool recv_line() {
  byte bt = 0;
  if (Serial.available() > 0) {
    bt = Serial.read();
    if (recv_index < recv_buffer_size - 1) {
      recv_buffer[recv_index] = bt;
      recv_index += 1;
      recv_buffer[recv_index + 1] = 0;
    }
  }
  return bt == 10;
}

void loop() {

  tensile = random(-100, 1000);
  
  Serial.print("X: " );
  Serial.print(x_pos);
  Serial.print(" T: ");
  Serial.print(tensile);
  Serial.println();

  if ( recv_line() ) {

      if ( memcmp( recv_buffer, "G28" , 3 ) == 0 ) {
        Serial.println("HOMING ... ");
        is_homing = true;
      }
      if ( memcmp( recv_buffer, "G00" , 3 ) == 0 ) {
        Serial.println("RESET X... ");
        is_homing = false;
        x_pos = 80;
      }
      recv_index = 0;
      recv_buffer[0] = 0;
  }

  if (x_pos > 0 && is_homing) {
    x_pos -= 1;
  }

  if (x_pos == 0 )  {
    is_homing = false;
  }
  delay(100);
}
