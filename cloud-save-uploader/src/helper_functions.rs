use std::env;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

fn to_io_error<E: std::fmt::Display>(err: E) -> Error {
    Error::new(ErrorKind::Other, err.to_string())
}

pub fn resolve_save_dir() -> Result<PathBuf, Error> {
    if let Ok(overridden) = env::var("VS_SAVE_DIR") {
        return Ok(PathBuf::from(overridden));
    }

    #[cfg(target_os = "windows")]
    {
        use std::fs;
        let appdata = env::var("APPDATA").map_err(to_io_error)?;

        // create the VintagestoryData folder if it doesn't exist
        if !Path::new(&appdata).join("VintagestoryData").exists() || !Path::new(&appdata).join("VintagestoryData").join("Saves").exists() {
            fs::create_dir_all(Path::new(&appdata).join("VintagestoryData").join("Saves")).map_err(to_io_error)?;
        }        

        return Ok(Path::new(&appdata)
            .join("VintagestoryData")
            .join("Saves"));
    }

    #[cfg(target_os = "macos")]
    {
        let home = env::var("HOME").map_err(to_io_error)?;
        return Ok(Path::new(&home)
            .join("Library")
            .join("Application Support")
            .join("VintagestoryData")
            .join("Saves"));
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err(Error::new(
            ErrorKind::Unsupported,
            "automatic save directory detection not implemented for this OS; set VS_SAVE_DIR",
        ))
    }
}