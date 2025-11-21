mod upload_core;
mod helper_functions;
mod manifest_info;
use crate::manifest_info::{get_manifest_info, manifest_status_message};

use crate::upload_core::{download_save, upload_save, UploadProgress};
use dotenvy::dotenv;
use eframe::{egui, App, CreationContext};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

enum UploadEvent {
    Started,
    Finished,
    Failed(String),
    Progress(UploadProgress),
}

enum DownloadEvent {
    Started,
    Finished,
    Failed(String),
}

struct CloudApp {
    folder_bucket: String,


    upload_status: String,
    upload_progress: f32,
    upload_error: Option<String>,
    uploading: bool,
    upload_receiver: Receiver<UploadEvent>,
    upload_sender: Sender<UploadEvent>,

    download_status: String,
    download_progress: f32,
    download_error: Option<String>,
    downloading: bool,
    download_receiver: Receiver<DownloadEvent>,
    download_sender: Sender<DownloadEvent>,
}

impl CloudApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let (upload_sender, upload_receiver) = mpsc::channel();
        let (download_sender, download_receiver) = mpsc::channel();
        let initial_status = manifest_status_message().unwrap();
        let manifest = get_manifest_info().unwrap();
        let folder_bucket = manifest.current_used_bucket;

        CloudApp {
            folder_bucket: folder_bucket,
            upload_status: initial_status,
            upload_progress: 0.0,
            upload_error: None,
            uploading: false,
            upload_receiver,
            upload_sender,
            download_status: "Idle".to_string(),
            download_progress: 0.0,
            download_error: None,
            downloading: false,
            download_receiver,
            download_sender,
        }
    }

    fn handle_events(&mut self) {
        while let Ok(event) = self.upload_receiver.try_recv() {
            match event {
                UploadEvent::Started => {
                    self.uploading = true;
                    self.upload_status = "Uploading saves...".to_string();
                    self.upload_error = None;
                    self.upload_progress = 0.1;
                }
                UploadEvent::Progress(progress) => {
                    self.uploading = true;
                    let pct = if progress.total_bytes > 0 {
                        progress.uploaded_bytes as f32 / progress.total_bytes as f32
                    } else {
                        0.0
                    };
                    self.upload_progress = pct;
                    if progress.total_bytes > 0 {
                        let speed_mb_s = if progress.elapsed_secs > 0.0 {
                            (progress.uploaded_bytes as f32 / 1024.0 / 1024.0) / progress.elapsed_secs
                        } else {
                            0.0
                        };
                        self.upload_status = format!(
                            "Uploading {} ({:.1}% @ {:.2} MB/s)",
                            progress.current_file,
                            pct * 100.0,
                            speed_mb_s
                        );
                    }
                }
                UploadEvent::Finished => {
                    self.uploading = false;
                    self.upload_status = "Upload complete".to_string();
                    self.upload_progress = 1.0;
                }
                UploadEvent::Failed(err) => {
                    self.uploading = false;
                    self.upload_status = "Upload failed".to_string();
                    self.upload_error = Some(err);
                    self.upload_progress = 0.0;
                }
            }
        }
    }
    fn handle_download_events(&mut self) {
        while let Ok(event) = self.download_receiver.try_recv() {
            match event {
                DownloadEvent::Started => {
                    self.downloading = true;
                    self.download_status = "Downloading saves...".to_string();
                    self.download_error = None;
                    self.download_progress = 0.1;
                }
                DownloadEvent::Finished => {
                    self.downloading = false;
                    self.download_status = "Download complete".to_string();
                    self.download_progress = 1.0;
                }
                DownloadEvent::Failed(err) => {
                    self.downloading = false;
                    self.download_status = "Download failed".to_string();
                    self.download_error = Some(err);
                    self.download_progress = 0.0;
                }
            }
        }
    }
}

impl App for CloudApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.handle_events();
        self.handle_download_events();
        egui::CentralPanel::default().show(ctx, |ui| {
            let program_data = get_manifest_info().unwrap_or_default();
            //println!("program_data: {:?}", program_data);
            //println!("folder_bucket: {:?}", self.folder_bucket);
            let manifest = program_data.all_file_info
                .get(&self.folder_bucket)
                .map(|m| m.files.clone())
                .unwrap_or_default();
            let total_saves = manifest.len();
            let total_size = manifest
                .values()
                .map(|file_info| file_info.file_size.unwrap_or(0))
                .sum::<u64>();
            let total_playtime = manifest
                .values()
                .map(|file_info| file_info.playtime)
                .sum::<u64>();
            let mut sorted_entries: Vec<_> = manifest.iter().collect();
            sorted_entries.sort_by_key(|(key, _)| (*key).clone());

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.heading("Vintage Story Cloud Uploader");
                if ui.button("Upload now").clicked() && !self.uploading {
                    let tx = self.upload_sender.clone();
                    let folder = self.folder_bucket.clone();
                    thread::spawn(move || {
                        let (progress_tx, progress_rx) = mpsc::channel();
                        let progress_forward_tx = tx.clone();
                        thread::spawn(move || {
                            while let Ok(progress) = progress_rx.recv() {
                                let _ = progress_forward_tx.send(UploadEvent::Progress(progress));
                            }
                        });

                        let _ = tx.send(UploadEvent::Started);
                        let result = upload_save(folder, Some(progress_tx));
                        match result {
                            Ok(_) => {
                                let _ = tx.send(UploadEvent::Finished);
                            }
                            Err(err) => {
                                let _ = tx.send(UploadEvent::Failed(err.to_string()));
                            }
                        }
                    });
                }
            });

            ui.add_space(10.0);
            egui::Grid::new("main_grid")
                .num_columns(2)
                .spacing([25.0, 8.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    // Left column: Statistics
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label("Statistics:");
                        ui.label(&format!("Total saves: {}", total_saves));
                        ui.label(&format!("Total size: {} mb", total_size / 1024 / 1024));
                        ui.label(&format!("Total playtime: {}s", total_playtime));
                    });

                    // Right column: Controls and manifest
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {

                        ui.columns(2, |columns| {
                            columns[0].set_max_height(0.0);
                            columns[0].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.heading("Select folder:");
                                ui.text_edit_singleline(&mut self.folder_bucket);
                            });
                            
                            columns[1].set_max_height(0.0);
                            columns[1].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                if ui.button("Download now").clicked() && !self.downloading {
                                    let tx = self.download_sender.clone();
                                    let folder = self.folder_bucket.clone();
                                    thread::spawn(move || {
                                        let _ = tx.send(DownloadEvent::Started);
                                        let result = download_save(folder);
                                        match result {
                                            Ok(_) => {
                                                let _ = tx.send(DownloadEvent::Finished);
                                            }
                                            Err(err) => {
                                                let _ = tx.send(DownloadEvent::Failed(err.to_string()));
                                            }
                                        }
                                    });
                                }
                            });
                        });
                        if self.uploading {
                            ui.add(egui::ProgressBar::new(self.upload_progress).show_percentage());
                            ui.label(&self.upload_status);
                        } else {
                            ui.label(&self.upload_status);
                        }

                        if let Some(error) = &self.upload_error {
                            ui.colored_label(egui::Color32::RED, error);
                        }

                        if self.downloading {
                            ui.add(egui::ProgressBar::new(self.download_progress).show_percentage());
                            ui.label(&self.download_status);
                        } else {
                            ui.label(&self.download_status);
                        }

                        if let Some(error) = &self.download_error {
                            ui.colored_label(egui::Color32::RED, error);
                        }

                        ui.heading(&format!("Cloud Saves in folder {} :{}", self.folder_bucket, total_saves));

                        egui::Grid::new("manifest_grid")
                            .num_columns(4)
                            .spacing([10.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for (key, file_info) in &sorted_entries {
                                    ui.label((*key).as_str());
                                    ui.label(&format!("{} )", file_info.world_name));
                                    ui.label(&format!("{} mb", file_info.file_size.unwrap_or(0) / 1024 / 1024));
                                    ui.label(&format!("{}s", file_info.playtime));
                                    ui.end_row();
                                }
                            });
                    });

                    ui.end_row();
                });
        });
    }
    
}



fn main() -> eframe::Result<()> {
    println!("Starting Cloud Save Uploader");
    dotenv().ok();
    let viewport = egui::ViewportBuilder::default()
        .with_resizable(false)
        .with_inner_size(egui::vec2(700.0, 500.0));             

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Cloud Save Uploader v0.0.1",
        native_options,
        Box::new(|cc| Ok(Box::new(CloudApp::new(cc)))),
    )?;
    Ok(())
}
