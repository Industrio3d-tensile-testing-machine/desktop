use std::fmt;
use serialport;
use egui::{Ui, Response, WidgetText};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use egui_plot::{Line, Plot, PlotPoints};

#[derive(PartialEq, Debug)]
enum ConnectionState {
    Connecting,
    Connected,
    Disconnected
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

const BACK_ARROW_ICON: egui::ImageSource = egui::include_image!("../assets/arrow_back_FILL1_wght400_GRAD0_opsz24.png");
const FORWARD_ARROW_ICON: egui::ImageSource = egui::include_image!("../assets/arrow_forward_FILL1_wght400_GRAD0_opsz24.png");
const HOME_ICON: egui::ImageSource = egui::include_image!("../assets/home_FILL1_wght400_GRAD0_opsz24.png");
const PLAY_ARROW_ICON: egui::ImageSource = egui::include_image!("../assets/play_arrow_FILL1_wght400_GRAD0_opsz24.png");
const STOP_ICON: egui::ImageSource = egui::include_image!("../assets/stop_FILL1_wght400_GRAD0_opsz24.png");
const PAUSE_ICON: egui::ImageSource = egui::include_image!("../assets/pause_FILL1_wght400_GRAD0_opsz24.png");
const USB_ICON: egui::ImageSource = egui::include_image!("../assets/usb_FILL1_wght400_GRAD0_opsz24.png");




#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TensileTestingApp {
    #[serde(skip)]
    connection_state: ConnectionState,
    save_connection_settings: bool,
    auto_connect_on_startup: bool,
    serial_port: String,
    baud_rate: String,
}

impl Default for TensileTestingApp {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            save_connection_settings: false,
            auto_connect_on_startup: false,
            serial_port: Default::default(),
            baud_rate: Default::default(),
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
        let sin: PlotPoints = (0..1000).map(|i| {
            let x = i as f64 * 0.01;
            [x, x.sin()]
        }).collect();
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
        ui.add_visible_ui(!matches!(self.connection_state, ConnectionState::Connected), |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                let state = ConnectionState::to_string(&self.connection_state);
                ui.label(state);
            });
        });

        let ports = serialport::available_ports().expect("No ports found!");

        egui::CollapsingHeader::new("Connection settings").default_open(true).show(ui, |ui| {
            egui::Frame::group(ui.style()).inner_margin(egui::Vec2::splat(10.0)).show(ui, |ui| {
                egui::Grid::new("my_grid")
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
                        // let resp = ui.add(egui::TextEdit::singleline(&mut self.baud_rate).hint_text("115200"));


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
                    });
                    ui.add_space(8.0);
                    // save connection settings checkox
                    ui.checkbox(&mut self.save_connection_settings, "Save connection settings");
                    ui.checkbox(&mut self.auto_connect_on_startup, "Auto connect on startup");
                    
                    ui.separator();
                    
                    if (matches!(self.connection_state, ConnectionState::Connected)) {

                        if full_centered_button(ui, "Disconnect").clicked() {
                            self.connection_state = ConnectionState::Disconnected
                        }
                    } else {
                        if full_centered_button(ui, "Connect").clicked() {
                            self.connection_state = ConnectionState::Connecting;

                            let bru32 = self.baud_rate.parse::<u32>().unwrap();

                            let openresult = serialport::new(&self.serial_port, bru32).open();
                            match openresult {
                                Ok(_)  => {
                                    self.connection_state = ConnectionState::Connected;
                                }
                                Err(err) => {
                                    self.connection_state = ConnectionState::Disconnected;
                                    eprintln!("Failed {err}")
                                },
                            }

                        }
                    }
                    
            });
        }).fully_open();

        egui::CollapsingHeader::new("Controls").default_open(true).show(ui, |ui| {
            egui::Frame::group(ui.style()).inner_margin(egui::Vec2::splat(10.0)).show(ui, |ui| {
                ui.vertical(|ui| {
                ui.add_enabled_ui(!matches!(self.connection_state, ConnectionState::Disconnected), |ui| {
                    ui.columns(3, |columns| {
                        columns[0].add(egui::Button::image_and_text(PLAY_ARROW_ICON, "Start"));
                        columns[1].add(egui::Button::image_and_text(PAUSE_ICON, "Pause"));
                        columns[2].add(egui::Button::image_and_text(STOP_ICON, "Cancel"))
                    });

                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(egui::Button::image(BACK_ARROW_ICON)).on_hover_text("Jog left").clicked() {
                            // actie wanneer de knop wordt ingedrukt
                        }
        
                        if ui.add(egui::Button::image(HOME_ICON)).on_hover_text("Home").clicked() {
                            // actie wanneer de knop wordt ingedrukt
                        }
        
                        if ui.add(egui::Button::image(FORWARD_ARROW_ICON)).on_hover_text("Jog left").clicked() {
                            // actie wanneer de knop wordt ingedrukt
                        }
        
                    });
                });
                });
            });
        });
    }
}

impl eframe::App for TensileTestingApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {

            self.plot_ui(ui)
        });

        egui::SidePanel::new(egui::panel::Side::Right, "side_panel").show(ctx, |ui| {
            self.panel_ui(ui)
        });

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
