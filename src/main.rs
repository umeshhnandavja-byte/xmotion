use std::env;
use std::fs;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(600.0, 300.0)),
        ..Default::default()
    };
    eframe::run_native(
        "xmotion",
        options,
        Box::new(|_cc| Box::new(XmotionApp::default())),
    )
}

#[derive(PartialEq)]
enum ActiveWindow {
    Home,
    Saving,
    Wallpaper,
    Settings,
}

pub struct XmotionApp {
    current_window: ActiveWindow,
    pub folder_path: String,
    pub top_bar: bool,
    pub transparency: bool,
}

impl Default for XmotionApp {
    fn default() -> Self {
        Self {
            current_window: ActiveWindow::Home,
            folder_path: String::new(),
            top_bar: true,
            transparency: true,
        }
    }
}

impl eframe::App for XmotionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("xmotion - a Wallpaper Manager");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙").clicked() {
                        self.current_window = ActiveWindow::Settings;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_window {
                ActiveWindow::Home => {
                    ui.vertical_centered(|ui| {
                        ui.label("Welcome to xmotion");
                        ui.add_space(50.0);
                        ui.label("Wallpaper Folder Location:");
                        ui.add(egui::TextEdit::singleline(&mut self.folder_path).desired_width(400.0).hint_text(env::var("HOME").unwrap_or_else(|_| "Error".to_string())));
                        ui.add_space(20.0);
                        
                        if ui.button("Save").clicked() {
                            self.current_window = ActiveWindow::Saving;
                            println!("Saved path: {}", self.folder_path);
                        }
                    });                    
                }
                ActiveWindow::Settings => {
                    ui.vertical_centered(|ui| {
                        ui.label("Settings");
                        ui.add_space(20.0);
                        ui.label("Wallpaper Folder Location: ");
                        ui.add_space(20.0);
                        ui.label("Transparency: ");
                        ui.add_space(20.0);
                        ui.label("Top Bar: ");
                        ui.add_space(20.0);
                        

                        if ui.button("Save").clicked() {
                            self.current_window = ActiveWindow::Saving;
                            println!("Saved path: {}", self.folder_path);
                        }
                    });                    

                }
                ActiveWindow::Saving => {
                    ui.vertical_centered(|ui| {
                        ui.label("Saving....");
                    });
                }
                ActiveWindow::Wallpaper => {
                    ui.vertical_centered(|ui| {
                        ui.label("Wallpapers"); 
                    });                    
                }
            }
        });
    }
}
