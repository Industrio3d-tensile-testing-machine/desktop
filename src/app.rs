use crate::{
    button,
    ButtonVariant, SECONDARY, GREY_WHITE, PRIMARY_COLOR, icon_button
};

use std::{fmt::{self, Formatter}, ops::RangeInclusive};
use egui::{Ui, Response, WidgetText, Align2, Vec2, Color32, style::WidgetVisuals, Rounding, Stroke};
use log::debug;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use egui_plot::{Line, Plot, PlotPoints};
use egui_toast::{self, Toasts, Toast, ToastOptions, ToastKind};

mod serial_driver;
mod icons;
use self::serial_driver::SerialDriver;

const INDUSTRIO_LOGO: egui::ImageSource<'_> = egui::include_image!("../assets/logo.png");
const SPECIMEN_DIAGRAM: egui::ImageSource<'_> = egui::include_image!("../assets/specimen.png");

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
pub struct TestParamters {
    speed: f64,
    area: f64,
    max_distance: f64
}

impl Default for TestParamters {
    fn default() -> Self {
        Self {
            speed: 1.0,
            area: Default::default(),
            max_distance: Default::default()
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
    
    test_parameters: TestParamters,
    jog_control_step_distance: f32,

    is_testing: bool,
    #[serde(skip)]
    start_data_point: Option<(f64, f64)>,

    #[serde(skip)]
    driver : SerialDriver,

    #[serde(skip)]
    toast: Toasts,

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
            test_parameters: Default::default(),
            jog_control_step_distance: 1.0,
            is_testing: false,
            start_data_point: Default::default(),
            driver : SerialDriver::new(),
            toast: Toasts::new().anchor(Align2::LEFT_TOP, (10.0, 10.0)).direction(egui::Direction::TopDown),
            data_points : Vec::new(),
        }
    }
}

impl TensileTestingApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.selection.bg_fill = PRIMARY_COLOR;

        cc.egui_ctx.set_style(style);

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
        let ports = self.driver.available_ports().expect("No ports available");

        ui.add_space(10.0);
        ui.vertical_centered_justified(|ui| {
            ui.add(egui::Image::new(INDUSTRIO_LOGO).max_height(15.0));
        });
        ui.add_space(10.0);

        egui::CollapsingHeader::new("Connection settings").default_open(true).show(ui, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                egui::Grid::new("state_grid").num_columns(2).spacing([50.0, 8.0]).show(ui, |ui| {
                    let state = ConnectionState::to_string(&self.connection_state);

                    ui.label("State:");
                    ui.label(state);
                    ui.end_row();
                    ui.label("Current position:");
                    ui.label(format!("{}mm", self.driver.values().position));
                    ui.end_row();
                });

                ui.separator();

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
                            if render_full_width(ui, button("Disconnect", ButtonVariant::Secondary)).clicked() {
                                self.connection_state = ConnectionState::Disconnected;
                                self.driver.close(); 
                            }
                            // if full_centered_button(ui, "Disconnect").clicked() {
                            //     self.connection_state = ConnectionState::Disconnected;
                            //     self.driver.close();
                            // }
                        },
                        ConnectionState::Disconnected => {
                            if render_full_width(ui, button("Connect", ButtonVariant::Secondary)).clicked() {
                                self.connection_state = ConnectionState::Connecting;
    
                                let bru32 = self.baud_rate.parse::<u32>().unwrap();

                                // self.driver.connect(self.serial_port, bru32);
                                
                                // match serialport::new(&self.serial_port, bru32).timeout(Duration::from_millis(10)).open() {
                                //     Ok(mut serial_interface)  => {
                                //         self.driver.set_serial(serial_interface);
                                //         self.connection_state = ConnectionState::Connected;
                                //     }
                                //     Err(err) => {
                                //         self.connection_state = ConnectionState::Disconnected;
                                //         self.toast.add(Toast { text: err.to_string().into(), kind: ToastKind::Info, options: ToastOptions::default().duration_in_seconds(3.0)});
                                //     },
                                // }
    
                                match self.driver.open(&self.serial_port, bru32) {
                                    Ok(_)  => {                    
                                        self.connection_state = ConnectionState::Connected;
                                    }
                                    Err(err) => {
                                        self.connection_state = ConnectionState::Disconnected;
                                        self.toast.add(Toast { text: err.to_string().into(), kind: ToastKind::Info, options: ToastOptions::default().duration_in_seconds(3.0)});
                                    },
                                }
                            }
                        },
                    }
            });
        }).fully_open();

        self.specimen_settings_panel(ui);

        egui::CollapsingHeader::new("Controls").open(Some(is_serial_connected(&self.connection_state))).show(ui, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.add_enabled_ui(is_serial_connected(&self.connection_state), |ui| {
                        ui.columns(3, |columns| {
                            if columns[0].add(icon_button(icons::PLAY_ARROW_ICON, "Start", ButtonVariant::Primary)).clicked() {
                                let _ = self.driver.start_test(self.test_parameters.speed);
                                self.is_testing = true;
                                self.data_points.clear();
                                self.start_data_point = None;
                            };
                            columns[1].add(icon_button(icons::PAUSE_ICON, "Pause", ButtonVariant::Secondary));
                            if columns[2].add(icon_button(icons::STOP_ICON, "Cancel", ButtonVariant::Secondary)).clicked() {
                                self.driver.cancel_test();
                                self.is_testing = false;
                            }
                        });

                        ui.add_space(8.0);
                        ui.add_enabled_ui(!self.driver.is_acknowledge_pending(), |ui| {
                            ui.horizontal_wrapped(|ui| {
                                if ui.add_enabled(self.driver.is_homed(), egui::Button::image(icons::BACK_ARROW_ICON)).on_hover_text("Jog back").clicked() {
                                    // TODO: error
                                    let _ = self.driver.jog(self.jog_control_step_distance).is_err();
                                }

                                // debug!("ackknowledge pending in ui: {}", self.driver.is_acknowledge_pending());
                
                                if ui.add(egui::Button::image(icons::HOME_ICON)).on_hover_text("Home").clicked() {
                                    self.is_testing = false;
                                    // actie wanneer de knop wordt ingedrukt
                                    let result = self.driver.start_home();

                                    match result {
                                        Ok(_) => {},
                                        Err(err) => {
                                            self.toast.add(Toast {
                                                text: err.to_string().into(),
                                                kind: ToastKind::Info,
                                                options: ToastOptions::default().duration_in_seconds(1.0)
                                            });
                                        },
                                    };
                                }
                
                                if ui.add_enabled(self.driver.is_homed(), egui::Button::image(icons::FORWARD_ARROW_ICON)).on_hover_text("Jog forward").clicked() {
                                    // actie wanneer de knop wordt ingedrukt
                                    let _ = self.driver.jog(-self.jog_control_step_distance);
                                }
                            });

                            ui.horizontal_centered(|ui| {
                                if ui.add(egui::SelectableLabel::new(self.jog_control_step_distance == 0.1, "0.1")).clicked() { self.jog_control_step_distance = 0.1 };
                                if ui.add(egui::SelectableLabel::new(self.jog_control_step_distance == 1.0, "1")).clicked() { self.jog_control_step_distance = 1.0 };
                                if ui.add(egui::SelectableLabel::new(self.jog_control_step_distance == 10.0, "10")).clicked() { self.jog_control_step_distance = 10.0 };
                            });
                        })
                    });
                });
            });
        });
    }

    fn specimen_settings_panel(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Specimen settings").show(ui, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.allocate_ui(Vec2{x: 125.0, y: 80.0}, |ui| {
                        ui.add(egui::Image::new(SPECIMEN_DIAGRAM));

                        ui.add(egui::DragValue::new(&mut self.test_parameters.area))
                    });

                    ui.add_space(25.0);

                    egui::Grid::new("specimen_settings_grid")
                    .num_columns(2)
                    .spacing([0.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Speed:");
                        ui.add(egui::DragValue::new(&mut self.test_parameters.speed).suffix("mm/s").clamp_range(RangeInclusive::new(0.01, 10.0)));
                        ui.end_row();

                        ui.label("Max distance:");
                        ui.add(egui::DragValue::new(&mut self.test_parameters.max_distance).suffix("mm"));
                        ui.end_row();

                        ui.label("Specimen area:");
                        ui.add(egui::DragValue::new(&mut self.test_parameters.area).suffix("mm²"));
                        ui.end_row();
                    });
                })
            })
        });
    }

    fn update_data(&mut self) {
        let opt_values = self.driver.update();

        if let Some( values ) = opt_values {
            if (self.is_testing) {
                let x = values.position as f64/ 100.0;
                let f = values.tensile as f64/ 10.0;

                // debug!("data update x:{}, f:{}", x, f);

                if let Some((start_pos, start_force)) = self.start_data_point {
                    let pos = start_pos - x;
                    let force = f - start_force;
                    self.data_points.push([pos, force]);
                } else {
                    self.start_data_point = Some((x, f));
                    self.data_points.push([0.0, 0.0]);
                }
    
                if !self.driver.is_acknowledge_pending() {
                    self.is_testing = false
                }
            }
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
        // debug!("ui update");
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::H)) {
            let result = self.driver.start_home();

            match result {
                Ok(_) => {},
                Err(err) => {
                    self.toast.add(Toast {
                        text: err.to_string().into(),
                        kind: ToastKind::Info,
                        options: ToastOptions::default().duration_in_seconds(1.0)
                    });
                },
            };
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)) {
            let result = self.driver.jog(self.jog_control_step_distance);

            match result {
                Ok(_) => {},
                Err(err) => {
                    self.toast.add(Toast {
                        text: err.to_string().into(),
                        kind: ToastKind::Info,
                        options: ToastOptions::default().duration_in_seconds(1.0)
                    });
                },
            };
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)) {
            let result = self.driver.jog(-self.jog_control_step_distance);

            match result {
                Ok(_) => {},
                Err(err) => {
                        self.toast.add(Toast {
                        text: err.to_string().into(),
                        kind: ToastKind::Info,
                        options: ToastOptions::default().duration_in_seconds(1.0)
                    });
                },
            };
        }

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
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                self.panel_ui(ui);
                ui.vertical_centered_justified(|ui| {
                    ui.small("v1.0.0 - ©Industrio");
                })
            })
        });

        // debug!("request repaint");
        // request new repaint
        ctx.request_repaint();
        self.toast.show(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.driver.close();
    }
}

fn render_full_width(ui: &mut Ui, widget: impl egui::Widget) -> Response {
    ui.add_sized(egui::vec2(ui.available_width(), 0.0), widget)
}

fn is_serial_connected(connection_state: &ConnectionState) -> bool {
    matches!(&connection_state, ConnectionState::Connected)
}