use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Child};
use std::sync::mpsc::{Sender, channel, Receiver};
use std::thread;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(800.0, 400.0)),
        ..Default::default()
    };
    eframe::run_native(
        "xmotion",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(XmotionApp::default())
        }),
    )
}

#[derive(PartialEq)]
enum ActiveWindow {
    Home,
    Saving,
    Wallpaper,
    Settings,
}

struct WallpaperVideo {
    video_path : String,
    thumb_path : String,
}

pub struct XmotionApp {
    current_window: ActiveWindow,
    pub folder_path: String,
    pub top_bar: bool,
    pub transparency: bool,
    gallery: Vec<WallpaperVideo>,
    thumb_receiver: Option<Receiver<WallpaperVideo>>,
    current_wallpaper: Option<Child>,
}

impl Default for XmotionApp {
    fn default() -> Self {
        let config_file = format!("{}/.xmotion_save", std::env::var("HOME").unwrap_or_default());
        
        let mut starting_window = ActiveWindow::Home;
        let mut starting_folder = String::new();
        let mut starting_receiver = None;

        if let Ok(saved_path) = fs::read_to_string(config_file) {
            let folder_path = saved_path.trim().to_string();
            
            if Path::new(&folder_path).is_dir() {
                starting_folder = folder_path.clone();
                starting_window = ActiveWindow::Saving;

                let (tx, rx) = channel();
                starting_receiver = Some(rx);

                save_thumb_thread(folder_path.clone(), tx);
            } else if !folder_path.is_empty() {
                starting_folder = folder_path;
            }

        }

        Self {
            current_window: starting_window,
            folder_path: starting_folder,
            top_bar: true,
            transparency: true,
            gallery: Vec::new(),
            thumb_receiver: starting_receiver,
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
                    
                    if ui.button("G").clicked() {
                        self.current_window = ActiveWindow::Wallpaper;
                    }
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
                            
                            change_folder_path(self.folder_path.clone());

                            let (tx, rx) = channel();
                            self.thumb_receiver = Some(rx);
                            self.current_window = ActiveWindow::Saving;
                            let folder = self.folder_path.clone();

                            save_thumb_thread(folder, tx);
                        }
                    });
                }
                ActiveWindow::Settings => {
                    ui.vertical_centered(|ui| {
                        ui.label("Settings");
                        ui.add_space(20.0);
                        ui.label("Wallpaper Folder Location: ");
                        ui.add(egui::TextEdit::singleline(&mut self.folder_path).desired_width(400.0).hint_text(format!("{}/.xmotion_save", std::env::var("HOME").unwrap_or_default())));
                        if ui.button("Change").clicked() {
                            change_folder_path(self.folder_path.clone());
                        }
                        ui.add_space(20.0);
                        ui.label("Transparency: ");
                        ui.add_space(20.0);
                        ui.label("Top Bar: ");
                        ui.add_space(20.0);
                        ui.label("Size: ");
                        ui.add_space(20.0);
                        ui.label("Random Wallpaper Every Launch: ");
                        ui.add_space(20.0);
                        ui.label("Random Wallpaper Every Time Interval: ");
                        ui.add_space(20.0);
                        

                        if ui.button("Save").clicked() {
                            self.current_window = ActiveWindow::Saving;
                            println!("Saved path: {}", self.folder_path);
                        }
                    });                    

                }
                ActiveWindow::Saving => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.spinner();
                        ui.label("Saving....");
                        ui.label(format!("Saving {}", self.gallery.len()));
                    });

                    if let Some(rx) = &self.thumb_receiver{
                        match rx.try_recv() {
                            Ok(item) => self.gallery.push(item),

                            Err(std::sync::mpsc::TryRecvError::Empty) => {}

                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                self.current_window = ActiveWindow::Wallpaper;
                            }
                        }
                    }

                    ctx.request_repaint();
                }
                ActiveWindow::Wallpaper => {
                    egui::ScrollArea::vertical().show(ui, |ui|{
                        let available_width = ui.available_width();

                        let spacing = ui.spacing().item_spacing.x;
                        let columns = 4.0;
                        let thumb_width = ((available_width - (spacing * (columns - 1.0))) / columns ) - 10.0;
                        let thumb_height = thumb_width * (9.0 / 16.0);

                        ui.horizontal_wrapped(|ui|{
                            for item in &self.gallery {
                                let thumb = egui::Image::new(format!("file://{}", item.thumb_path))
                                .fit_to_exact_size(egui::vec2(thumb_width, thumb_height))
                                .rounding(8.0);
                                
                                if ui.add(egui::ImageButton::new(thumb)).clicked(){
                                    if let Some(mut old_wallpaper) = self.current_wallpaper.take() {
                                        let _ = old_wallpaper.kill();
                                        let _ = old_wallpaper.wait();
                                    }

                                    let new_wallpaper = Command::new("xwinwrap")
                                        .args([
                                            "-ni", "-b", "-fs", "-s", "-st", "-sp", "-nf", "-ov",
                                            "--", 
                                            "mpv", "-wid", "WID", "--loop", "--no-audio", 
                                            &item.video_path
                                        ])
                                        .spawn();

                                    if let Ok(child) = new_wallpaper {
                                        self.current_wallpaper = Some(child);
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

fn save_thumb_thread(folder: String, tx: Sender<WallpaperVideo>) {
    thread::spawn(move || {
        let cache_folder = format!("{}/.cache/xmotion", std::env::var("HOME").unwrap_or_default());
        let _ = fs::create_dir_all(&cache_folder);

       if let Ok(files) = fs::read_dir(&folder) {
            for file in files.flatten() {
                let path = file.path();

                if path.extension().and_then(|e| e.to_str()) == Some("mp4") {
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

                    let item = WallpaperVideo { video_path, thumb_path};
                    let _ = tx.send(item);
                }
            }
       }
    });
}

fn change_folder_path(new_folder_path: String) {
        
    let config_file = format!("{}/.xmotion_save", std::env::var("HOME").unwrap_or_default());
    let _ = fs::write(config_file, new_folder_path);
}
