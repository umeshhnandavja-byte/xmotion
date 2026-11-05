use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Child};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
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
        Box::new(cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(XmotionApp::default());
        }
    ))
}

#[derive(PartialEq)]
enum ActiveWindow {
    Home,
    Saving,
    Wallpaper,
    Settings,
}

struct Wallpaper_Video {
    video_path : String,
    thumb_path : String,
}

pub struct XmotionApp {
    current_window: ActiveWindow,
    pub folder_path: String,
    pub top_bar: bool,
    pub transparency: bool,
    gallery: Vec<Wallpaper_Video>,
    thumb_receiver: Option<Receiver<Wallpaper_Video>>,
    current_wallpaper: Option<Child>,
}

impl Default for XmotionApp {
    fn default() -> Self {
        Self {
            current_window: ActiveWindow::Home,
            folder_path: String::new(),
            top_bar: true,
            transparency: true,
            gallery: Vec::new(),
            thumb_receiver: None,
            current_wallpaper: None,
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
                            let (tx, rx) = channel();
                            self.thumb_receiver = Some(rx);
                            self.current_window = ActiveWindow::Saving;
                            let folder = self.folder_path.clone();

                            thread::spawn(move || {
                               let cache_folder = "/tmp/xmotion_cache";
                               let _ = fs::create_dir_all(cache_folder);

                               if let Ok(files) = fs::read_dir(&folder_path) {
                                    for file in files.flatten() {
                                        let path = gile.path();

                                        if path.extension().and_then(|e| e.to.str()) == Some("mp4") {
                                            let video_path = path.to_string_lossy().to_string();
                                            let video_name = path.file_name().unwrap().to_string_lossy();
                                            let thumb_path = format!("{}/{}.jpg", cache_folder, video_name);

                                            if !Path::new(&thumb_path).exists() {
                                                let _ = Command::new("ffmpeg")
                                                    .args([
                                                        "-y",
                                                        "-i", &video_path,
                                                        "-ss", "00:00:00.000",
                                                        "-vframes", "1",
                                                        &thumb_path
                                                    ])
                                                    .output();
                                            }

                                            let item = Wallpaper_Video { video_path, thumb_path};
                                            let _ = tx.send(item);
                                        }
                                    }
                               }
                            });
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
                        ui.add_space("100.0");
                        ui.spinner();
                        ui.label("Saving....");
                        ui.label(format!("Saving {}", self.gallery.len()));
                    });

                    if let Some(rx) = &self.thumb_receiver{
                        match rx.try_recv() {
                            Ok(item) => self.gallery.push(item),

                            Err(std::sync::mpsc::TryRecvError::Empty) => {}

                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                self.state = ActiveWindow::Wallpaper;
                            }
                        }
                    }

                    ctx_request_repaint();
                }
                ActiveWindow::Wallpaper => {
                    egui::ScrollArea::vertical().show(ui, |ui|{
                        let available_width = ui.available_width();

                        let spacing = ui.spacing().item_spacing.x;
                        let columns = 4.0;
                        let thumb_width = ((available_width - (spacing * (columns - 1.0))) / columns ) - 0.1;
                        let thumb_height = thumb_width * (9 / 16);

                        ui.horizontal_wrapped(|ui|{
                            for item in &self.gallery {
                                let thumb = egui::Image::new(format!("file://{}", item.thumb_path))
                                .fit_to_exact_size(egui::vec2(thumb_width, thumb_height))
                                .rounding(8.0);
                                
                                if ui.add(egui::ImageButton::new(thumb)).clicked(){
                                    if let Some(mut old_process) = self.current_process.take() {
                                        let _ = old_process.kill();
                                        let _ = old_process.wait();
                                    }

                                    let new_process = Command::new("xwinwrap")
                                        .args([
                                            "-ni", "-b", "-fs", "-s", "-st", "-sp", "-nf", "-ov",
                                            "--", 
                                            "mpv", "-wid", "WID", "--loop", "--no-audio", 
                                            &item.video_path
                                        ])
                                        .spawn();

                                    if let Ok(child) = new_process {
                                        self.current_process = Some(child);
                                    }
                                }

                            }
                        });

                    });
                }
            }
        });
    }
}
