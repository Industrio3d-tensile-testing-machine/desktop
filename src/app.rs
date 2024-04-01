use std::{fmt::{self, Formatter}, time::Duration};
use serialport;
use egui::{Ui, Response, WidgetText};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use egui_plot::{Line, Plot, PlotPoints};

mod serial_driver;
use self::serial_driver::SerialDriver;

#[derive(PartialEq, Debug, Clone, Copy)]
enum ConnectionState {
    Connecting,
    Connected,
    Disconnected
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, EnumIter)]
enum BaudRate {
    B9600,
    B19200,
    B38400,
    B57600,
    B115200,
    B230400,
    B250000,
}

impl fmt::Display for BaudRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BaudRate::B9600 => write!(f, "9600"),
            BaudRate::B19200 => write!(f, "19200"),
            BaudRate::B38400 => write!(f, "38400"),
            BaudRate::B57600 => write!(f, "57600"),
            BaudRate::B115200 => write!(f, "115200"),
            BaudRate::B230400 => write!(f, "230400"),
            BaudRate::B250000 => write!(f, "250000"),
        }
    }
}


#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct UserPreferences {
    save_connection_settings: bool,
    auto_connect_on_startup: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        UserPreferences {
            save_connection_settings: true,
            auto_connect_on_startup: false
        }
    }
}


#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TensileTestingApp {
    user_preferences: UserPreferences,

    #[serde(skip)]
    connection_state: ConnectionState,
    serial_port: String,
    baud_rate: String,

    #[serde(skip)]
    driver : SerialDriver,

    #[serde(skip)]
    data_points : Vec<[f64;2]>,
}

impl Default for TensileTestingApp {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            user_preferences: UserPreferences::default(),
            serial_port: Default::default(),
            baud_rate: Default::default(),
            driver : SerialDriver::new(),
            data_points : Vec::new(),
        }
    }
}

impl TensileTestingApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn plot_ui(&mut self, ui: &mut egui::Ui) {
        // let sin: PlotPoints = (0..1000).map(|i| {
        //     let x = i as f64 * 0.01;
        //     [x, x.sin() * 3.0]
        // }).collect();
        // //let sin = PlotPoints::default();
        let sin : PlotPoints = self.data_points.clone().into();

        let line = Line::new(sin);
        Plot::new("my_plot").view_aspect(2.0)
        .label_formatter(|name, value| {
            if !name.is_empty() {
                format!("{}: {:.*}%", name, 1, value.y)
            } else {
                "".to_owned()
            }
        })
        .show(ui, |plot_ui| plot_ui.line(line));
    }
    
    fn panel_ui(&mut self, ui: &mut egui::Ui) {


        let ports = serialport::available_ports().expect("No ports found!");

        egui::CollapsingHeader::new("Connection settings").default_open(true).show(ui, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                egui::Grid::new("state_grid").num_columns(2).spacing([50.0, 8.0]).show(ui, |ui| {
                    let state = ConnectionState::to_string(&self.connection_state);
                    ui.label("State:");
                    ui.label(state);
                    ui.end_row()
                })
            });
    
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.add_enabled_ui(!is_serial_connected(&self.connection_state), |ui| {
                egui::Grid::new("connection_settings_grid")
                    .num_columns(2)
                    .spacing([50.0, 8.0])
                    .show(ui, |ui| {
                            // serial port
                            ui.label("Serial port:");
                            egui::ComboBox::new("serial_port_combobox", "")
                                .selected_text(&self.serial_port)
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(60.0);
                                    for p in ports {

                                        ui.selectable_value(&mut self.serial_port, p.port_name.to_string(), p.port_name);
                                    }
                                });
                            ui.end_row();

                            // baudrate
                            ui.label("Baudrate:");
                            egui::ComboBox::new("baud_rate_combobox", "")
                                .selected_text(&self.baud_rate)
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(60.0);
                                    for br in BaudRate::iter() {
                                        ui.selectable_value(&mut self.baud_rate, br.to_string(), br.to_string());
                                    }
                                });
                            ui.end_row();
                        })
                    });
                    ui.add_space(8.0);
                    // save connection settings checkox
                    ui.checkbox(&mut self.user_preferences.save_connection_settings, "Save connection settings");
                    ui.checkbox(&mut self.user_preferences.auto_connect_on_startup, "Auto connect on startup");
                    
                    ui.separator();

                    match self.connection_state {
                        // TODO:
                        ConnectionState::Connecting => {
                            ui.add(egui::Spinner::new());
                        },
                        ConnectionState::Connected => {
                            if full_centered_button(ui, "Disconnect").clicked() {
                                self.connection_state = ConnectionState::Disconnected
                            }
                        },
                        ConnectionState::Disconnected => {
                            if full_centered_button(ui, "Connect").clicked() {
                                self.connection_state = ConnectionState::Connecting;
    
                                let bru32 = self.baud_rate.parse::<u32>().unwrap();
    
                                
                                match serialport::new(&self.serial_port, bru32).timeout(Duration::from_millis(10)).open() {
                                    Ok(mut serial_interface)  => {

                                        self.driver.set_serial(serial_interface);
                                        //self.serial_interface = Some(serial_interface);
                                        // match self.serial_interface.as_deref_mut(){
                                        //      Some(s) => {
                                        //         let output = "This is a test. This is only a test.".as_bytes();                                        
                                        //         s.write(output).expect("Can't write to serial");
        
                                        //      },
                                        //     None => todo!(),
                                        // }
                                        self.connection_state = ConnectionState::Connected;
                                    }
                                    Err(err) => {
                                        self.connection_state = ConnectionState::Disconnected;
                                        eprintln!("Failed {err}")
                                    },
                                }
    
                            }
                        },
                    }
            });
        }).fully_open();

        egui::CollapsingHeader::new("Controls").default_open(true).show(ui, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.vertical(|ui| {
                ui.add_enabled_ui(!matches!(self.connection_state, ConnectionState::Disconnected), |ui| {
                    ui.columns(3, |columns| {
                        columns[0].add(egui::Button::image_and_text(PLAY_ARROW_ICON, "Start").fill(egui::Color32::from_rgb(0, 68, 204)));
                        columns[1].add(egui::Button::image_and_text(PAUSE_ICON, "Pause"));
                        columns[2].add(egui::Button::image_and_text(STOP_ICON, "Cancel"))
                    });

                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(egui::Button::image(BACK_ARROW_ICON)).on_hover_text("Jog back").clicked() {
                            // actie wanneer de knop wordt ingedrukt
                            let _ = self.driver.jog(-10);
                        }
        
                        if ui.add(egui::Button::image(HOME_ICON)).on_hover_text("Home").clicked() {
                            // actie wanneer de knop wordt ingedrukt
                            let _ = self.driver.start_home();
                        }
        
                        if ui.add(egui::Button::image(FORWARD_ARROW_ICON)).on_hover_text("Jog forward").clicked() {
                            // actie wanneer de knop wordt ingedrukt
                            let _ = self.driver.jog(10);
                        }
        
                    });
                });
                });
            });
        });
    }

    fn update_data(&mut self) {
        let opt_values = self.driver.update();

        if let Some( values ) = opt_values {

            let x = values.position as f64/ 100.0;
            let y = values.tensile as f64/ 10.0;

            self.data_points.push( [x, y] );
            //println!("Updated values: {:?}", v);
        }

    }


}

impl eframe::App for TensileTestingApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self);
    }

  
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        self.update_data();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.plot_ui(ui)
        });

        egui::SidePanel::new(egui::panel::Side::Right, "side_panel").show(ctx, |ui| {
            self.panel_ui(ui)
        });

        // request new repaint
        ctx.request_repaint();

    }
}




// trait CustomUi {
//     fn full_centered_button(ui: &mut Ui, text: impl Into<WidgetText>) -> Response;
// }

// impl CustomUi for egui::Ui {
//     fn full_centered_button(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
//         ui.add_sized(egui::vec2(ui.available_width(), 0.0),egui::Button::new(text))
//     }
// }

fn full_centered_button(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    full_width(ui, egui::Button::new(text))
}

fn full_width(ui: &mut Ui, widget: impl egui::Widget) -> Response {
    ui.add_sized(egui::vec2(ui.available_width(), 0.0), widget)
}

fn is_serial_connected(connection_state: &ConnectionState) -> bool {
    matches!(&connection_state, ConnectionState::Connected)
}