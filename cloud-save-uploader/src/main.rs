mod Uploadcore;

use crate::Uploadcore::{manifest_status_message, upload_save};
use dotenvy::dotenv;
use eframe::{egui, App, CreationContext};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

enum UploadEvent {
    Started,
    Finished,
    Failed(String),
}

struct CloudApp {
    upload_status: String,
    upload_progress: f32,
    upload_error: Option<String>,
    uploading: bool,
    receiver: Receiver<UploadEvent>,
    sender: Sender<UploadEvent>,
}

impl CloudApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let initial_status = manifest_status_message();
        CloudApp {
            upload_status: initial_status,
            upload_progress: 0.0,
            upload_error: None,
            uploading: false,
            receiver,
            sender,
        }
    }

    fn handle_events(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
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
}

impl App for CloudApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.handle_events();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Vintage Story Cloud Uploader");
            if ui.button("Upload now").clicked() && !self.uploading {
                let tx = self.sender.clone();
                thread::spawn(move || {
                    let _ = tx.send(UploadEvent::Started);
                    let result = upload_save();
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

            if self.uploading {
                ui.add(egui::ProgressBar::new(self.upload_progress).show_percentage());
                ui.label(&self.upload_status);
            } else {
                ui.label(&self.upload_status);
            }

            if let Some(error) = &self.upload_error {
                ui.colored_label(egui::Color32::RED, error);
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    println!("Starting Cloud Save Uploader");
    dotenv().ok();
    let viewport = egui::ViewportBuilder::default()
        .with_resizable(false)
        .with_inner_size(egui::vec2(300.0, 250.0));

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
