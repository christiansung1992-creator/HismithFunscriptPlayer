// src/handlers/calibration.rs

use actix_files::NamedFile;
use actix_web::{Error};

pub async fn handle_calibration_page() -> Result<NamedFile, Error> {
    Ok(NamedFile::open("./static/calibration.html")?
        .customize()
        .insert_header(("Cache-Control", "no-cache")))
}