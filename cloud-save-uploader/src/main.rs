mod uploadcore;

use crate::uploadcore::{download_save, manifest_status_message, upload_save};
use dotenvy::dotenv;
use eframe::{egui, App, CreationContext};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

enum UploadEvent {
    Started,
    Finished,
    Failed(String),
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
        let initial_status = manifest_status_message();
        CloudApp {
            folder_bucket: "vintagestory".to_string(),
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
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                ui.heading("Vintage Story Cloud Uploader");
                ui.add_space(10.0);

                ui.label("Select folder bucket:");
                ui.text_edit_singleline(&mut self.folder_bucket);
                ui.columns(2, |columns| {
                    columns[0].set_max_height(0.0);
                    columns[0].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Upload now").clicked() && !self.uploading {
                            let tx = self.upload_sender.clone();
                            let folder = self.folder_bucket.clone();
                            thread::spawn(move || {
                                let _ = tx.send(UploadEvent::Started);
                                let result = upload_save(folder);
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
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    println!("Starting Cloud Save Uploader");
    dotenv().ok();
    let viewport = egui::ViewportBuilder::default()
        .with_resizable(false)
        .with_inner_size(egui::vec2(600.0, 165.0));

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Cloud Save Uploader",
        native_options,
        Box::new(|cc| Ok(Box::new(CloudApp::new(cc)))),
    )?;
    Ok(())
}
