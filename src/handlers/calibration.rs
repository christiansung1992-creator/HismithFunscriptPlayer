// src/handlers/calibration.rs

use actix_files::NamedFile;
use actix_web::{web, Error, Responder, HttpResponse};
use serde::Deserialize;
use std::{collections::HashMap, env, path::PathBuf};
use tokio::fs;
use log::{error, info};

pub type CalibrationProfiles = HashMap<String, HashMap<String, f64>>;

#[derive(Deserialize)]
pub struct SaveProfilePayload {
    pub name: String,
    pub multipliers: HashMap<String, f64>,
}

fn calibration_profiles_path() -> PathBuf {
    let base = env::var("FUNSCRIPT_SHARE_PATH").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join(".calibration_profiles.json")
}

async fn load_profiles() -> Result<CalibrationProfiles, String> {
    let path = calibration_profiles_path();
    match fs::read_to_string(&path).await {
        Ok(s) => serde_json::from_str(&s).map_err(|e| format!("Failed parse calibration json: {}", e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(format!("Failed read calibration file {:?}: {}", path, e)),
    }
}

async fn save_profiles(profiles: &CalibrationProfiles) -> Result<(), String> {
    let path = calibration_profiles_path();
    let s = serde_json::to_string_pretty(profiles).map_err(|e| format!("Ser failed: {}", e))?;
    if let Some(parent) = path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return Err(format!("Failed create dirs for {:?}: {}", parent, e));
        }
    }
    fs::write(&path, s)
        .await
        .map_err(|e| format!("Failed write calibration file {:?}: {}", path, e))
}

pub async fn handle_calibration_page() -> Result<impl Responder, Error> {
    Ok(NamedFile::open("./static/calibration.html")?
        .customize()
        .insert_header(("Cache-Control", "no-cache")))
}

pub async fn get_bpm_mapping() -> impl Responder {
    let mapping = crate::buttplug::funscript_utils::get_bpm_intensity_mapping();
    HttpResponse::Ok().json(mapping)
}

pub async fn get_profiles() -> impl Responder {
    match load_profiles().await {
        Ok(profiles) => HttpResponse::Ok().json(profiles),
        Err(e) => {
            error!("Failed to load calibration profiles: {}", e);
            HttpResponse::InternalServerError().body("Failed to load calibration profiles")
        }
    }
}

pub async fn save_profile(payload: web::Json<SaveProfilePayload>) -> impl Responder {
    let mut profiles = load_profiles().await.unwrap_or_default();
    profiles.insert(payload.name.clone(), payload.multipliers.clone());
    match save_profiles(&profiles).await {
        Ok(_) => {
            info!("Saved calibration profile: {}", payload.name);
            HttpResponse::Ok().json(serde_json::json!({ "ok": true }))
        }
        Err(e) => {
            error!("Failed to save calibration profile {}: {}", payload.name, e);
            HttpResponse::InternalServerError().body("Failed to save calibration profile")
        }
    }
}