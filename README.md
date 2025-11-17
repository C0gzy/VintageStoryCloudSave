# VintageStoryCloudSave
### Version 0.0.1 Beta

Very Early Version of a basic upload download system for Vintage Story to keep your saves in the cloud

You are free to fork and continue work!
Made in Rust using Egui

## Dependencies
- eframe = "0.33.2"
- egui = "0.33.2"
- aws-config = { version = "1.5.1", features = ["behavior-version-latest"] }
- aws-sdk-s3 = { version = "1.38.0", features = ["rustls"] }
- tokio = { version = "1.40.0", features = ["macros", "rt-multi-thread"] }
- serde = { version = "1.0", features = ["derive"] }
- serde_json = "1.0"
- dotenvy = "0.15"

## How to Build

Create you own .env in /cloud-save-uploader an put in these values from aws S3
```
B2_KEY_ID=
B2_APPLICATION_KEY=
B2_BUCKET=
B2_REGION=
B2_ENDPOINT
```

## To-Do
- [ ] update UI
- [ ] Drop Down for Cloud Save selection
- [ ] Select specific files to upload and download
- [ ] Add profiles upload
- [ ] Add mod upload
- [ ] Upadte progess bar as it goes


