#![feature(proc_macro_hygiene, decl_macro)]
//! Library for sampic core functionality that is shared by both the client and server.
//! This includes:
//! - Configuration
//! - Storage backend management
//! - Image compression and storage
//! - URL generation
//! - Server endpoint
//! - Authentication (TODO)
mod region;

extern crate piston;

pub mod config {
    extern crate confy;
    use serde::{Deserialize, Serialize};

    #[derive(Debug)]
    pub enum ConfigError {
        InvalidStorageValue,
        APIKeyNotDefined,
        APISecretKeyNotDefined,
        IOError,
    }

    impl From<std::io::Error> for ConfigError {
        fn from(_: std::io::Error) -> ConfigError {
            ConfigError::IOError
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct SampConf {
        pub api_key: String,
        pub api_secret_key: String,
        pub region: String,
        pub endpoint: String,
        pub bucket: String,
        pub local_path: String,
        pub sampic_endpoint: String,
    }

    impl ::std::default::Default for SampConf {
        fn default() -> Self {
            Self {
                api_key: "".into(),
                region: "fr-par".into(),
                endpoint: "https://s3.fr-par.scw.cloud".into(),
                bucket: "sampic-store".into(),
                api_secret_key: "".into(),
                local_path: "/tmp/".into(),
                sampic_endpoint: "https://sampic.xyz/upload".to_string(),
            }
        }
    }

    pub fn api_key() -> Result<String, ConfigError> {
        let cfg: SampConf = confy::load("sampic")?;
        match cfg.api_key.as_ref() {
            "" => Err(ConfigError::APIKeyNotDefined),
            _ => Ok(cfg.api_key),
        }
    }

    pub fn list() -> Result<String, ConfigError> {
        let cfg = config()?;
        return Ok(format!(
            "{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n",
            "api_key",
            cfg.api_key,
            "api_secret_key",
            cfg.api_secret_key,
            "region",
            cfg.region,
            "endpoint",
            cfg.endpoint,
            "bucket",
            cfg.bucket,
            "local_path",
            cfg.local_path,
            "sampic_endpoint",
            cfg.sampic_endpoint
        ));
    }

    pub fn api_secret_key() -> Result<String, ConfigError> {
        let cfg: SampConf = confy::load("sampic")?;
        match cfg.api_secret_key.as_ref() {
            "" => Err(ConfigError::APISecretKeyNotDefined),
            _ => Ok(cfg.api_secret_key),
        }
    }

    pub fn local_path() -> Result<String, ConfigError> {
        let cfg: SampConf = confy::load("sampic")?;
        Ok(cfg.local_path)
    }

    pub fn config() -> Result<SampConf, ConfigError> {
        let cfg: SampConf = confy::load("sampic")?;
        return Ok(cfg);
    }

    pub fn set(key: String, value: String) -> Result<(), ConfigError> {
        let mut cfg = config()?;
        match key.as_str() {
            "api_key" => {
                cfg.api_key = value;
            }
            "region" => {
                cfg.region = value;
            }
            "endpoint" => {
                cfg.endpoint = value;
            }
            "bucket" => {
                cfg.bucket = value;
            }
            "api_secret_key" => {
                cfg.api_secret_key = value;
            }
            "local_path" => {
                cfg.local_path = value;
            }
            "sampic_endpoint" => {
                cfg.sampic_endpoint = value;
            }
            _ => return Err(ConfigError::InvalidStorageValue),
        };
        confy::store("sampic", cfg)?;
        return Ok(());
    }
}

pub mod storage {
    extern crate futures;
    use std::collections::hash_map::DefaultHasher;
    use std::fmt;
    use std::fs;
    use std::hash::Hasher;
    use std::io::prelude::*;
    use std::path::Path;
    use std::path::PathBuf;
    extern crate image;
    use super::config;
    use std::io;
    extern crate rusoto_core;
    extern crate rusoto_credential;
    extern crate rusoto_s3;
    use rusoto_core::request::{HttpClient, TlsError};
    use rusoto_core::{Region, RusotoError};
    use rusoto_credential::StaticProvider;
    use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};
    use tokio::io::AsyncReadExt;

    #[derive(Debug)]
    pub enum StorageError {
        SaveError,
        IOError,
        ReadError,
        ConfigError,
        CredentialsError,
        UnknownError,
    }

    impl From<TlsError> for StorageError {
        fn from(_: TlsError) -> StorageError {
            StorageError::IOError
        }
    }

    impl<T> From<RusotoError<T>> for StorageError {
        fn from(e: RusotoError<T>) -> StorageError {
            match e {
                RusotoError::Service(_) => StorageError::UnknownError,
                RusotoError::HttpDispatch(_) => StorageError::IOError,
                RusotoError::Credentials(_) => StorageError::CredentialsError,
                RusotoError::Validation(_)
                | RusotoError::ParseError(_)
                | RusotoError::Unknown(_)
                | RusotoError::Blocking => StorageError::UnknownError,
            }
        }
    }
    impl From<std::io::Error> for StorageError {
        fn from(_: std::io::Error) -> StorageError {
            StorageError::IOError
        }
    }

    impl From<config::ConfigError> for StorageError {
        fn from(_: config::ConfigError) -> StorageError {
            StorageError::ConfigError
        }
    }

    impl From<minreq::Error> for StorageError {
        fn from(_: minreq::Error) -> StorageError {
            StorageError::ReadError
        }
    }

    impl fmt::Display for StorageError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                StorageError::SaveError => write!(f, "Save error"),
                StorageError::IOError => write!(f, "IO error"),
                StorageError::ReadError => write!(f, "Read error"),
                StorageError::UnknownError => write!(f, "Unknown error"),
                StorageError::ConfigError => write!(f, "Configuration error"),
                StorageError::CredentialsError => write!(f, "Configuration error"),
            }
        }
    }
    type StorageResult<I> = Result<I, StorageError>;

    pub trait Storage {
        fn save(&self, buffer: &[u8], extension: String, w: u32, h: u32) -> StorageResult<String>;
        fn read_to(&self, name: &String, to: &mut Vec<u8>) -> StorageResult<()>;
        fn link(&self, name: &String) -> StorageResult<String>;
        fn hash(&self, buffer: &[u8]) -> String {
            let mut hasher = DefaultHasher::new();
            hasher.write(&buffer);
            format!("{:x}", hasher.finish())
        }
    }
    #[derive(Debug)]
    pub struct Local {
        pub path: PathBuf,
    }

    impl Local {
        pub fn new() -> Self {
            let local_path = config::local_path().unwrap();
            let path = Path::new(&local_path);
            return Local {
                path: path.to_owned(),
            };
        }
    }

    impl Storage for Local {
        fn save(&self, buffer: &[u8], extension: String, w: u32, h: u32) -> StorageResult<String> {
            let hash = self.hash(buffer);
            let filename = format!("{}.{}", hash, extension);
            let file_path = self.path.join(Path::new(&filename));
            let ret = file_path.as_path().display().to_string();
            image::save_buffer(
                file_path,
                buffer,
                w.into(),
                h.into(),
                image::ColorType::Rgba8,
            )
            .expect("Failure");
            Ok(ret)
        }

        fn read_to(&self, name: &String, to: &mut Vec<u8>) -> StorageResult<()> {
            let file_path = self.path.join(Path::new(&name));
            fs::File::open(file_path)?.read_to_end(to)?;
            Ok(())
        }

        fn link(&self, name: &String) -> StorageResult<String> {
            let file_path = self.path.join(Path::new(&name));
            Ok(file_path.display().to_string())
        }
    }

    pub struct S3Store {
        pub bucket: String,
        pub endpoint: String,
        runtime: tokio::runtime::Runtime,
        client: S3Client,
    }
    impl S3Store {
        pub fn new() -> StorageResult<Self> {
            let sampconf = config::config()?;
            let region = Region::Custom {
                name: sampconf.region.to_owned(),
                endpoint: sampconf.endpoint.to_owned(),
            };
            let runtime = tokio::runtime::Runtime::new()?;
            Ok(S3Store {
                bucket: sampconf.bucket,
                endpoint: sampconf.endpoint,
                runtime,
                client: S3Client::new_with(
                    HttpClient::new()?,
                    StaticProvider::new_minimal(sampconf.api_key, sampconf.api_secret_key),
                    region,
                ),
            })
        }
    }

    impl Storage for S3Store {
        fn save(&self, buffer: &[u8], extension: String, w: u32, h: u32) -> StorageResult<String> {
            let local_storage = Local {
                path: Path::new("/tmp/").to_path_buf(),
            };
            let name = format!("{}.{}", self.hash(buffer), extension);
            let link = self.link(&name);
            let local_path = local_storage.save(&buffer, extension, w, h)?;
            println!("{}", local_path);
            let mut file = std::fs::File::open(local_path)?;
            let mut buf: Vec<u8> = vec![];
            file.read_to_end(&mut buf)?;

            self.runtime
                .block_on(self.client.put_object(PutObjectRequest {
                    body: Some(buf.into()),
                    bucket: self.bucket.clone(), // I clone because of E0507, Is there any better way to do this?
                    key: name,
                    acl: Some("public-read".into()),
                    content_type: Some("image/png".into()),
                    ..Default::default()
                }))
                .expect("Error while uploading file");
            return link;
        }
        fn read_to(&self, name: &String, to: &mut Vec<u8>) -> StorageResult<()> {
            let object = self
                .runtime
                .block_on(self.client.get_object(GetObjectRequest {
                    bucket: self.bucket.clone(),
                    key: name.into(),
                    ..Default::default()
                }));
            let body = object?.body.ok_or(StorageError::ReadError)?;
            io::copy(&mut body.into_blocking_read(), to)?;
            Ok(())
        }
        fn link(&self, name: &String) -> StorageResult<String> {
            return Ok(format!(
                "{}/{}",
                self.endpoint
                    .replace("://", &format!("://{}.", &self.bucket)),
                name
            ));
        }

        fn hash(&self, buffer: &[u8]) -> String {
            let mut hasher = DefaultHasher::new();
            hasher.write(&buffer);
            format!("{:x}", hasher.finish())
        }
    }

    pub struct SampicServer {
        pub endpoint: String,
        s3: S3Store,
    }

    impl SampicServer {
        pub fn new() -> StorageResult<Self> {
            let sampconf = config::config()?;
            Ok(SampicServer {
                endpoint: sampconf.sampic_endpoint,
                s3: S3Store::new()?,
            })
        }
    }
    use minreq;
    impl Storage for SampicServer {
        fn save(&self, buffer: &[u8], extension: String, w: u32, h: u32) -> StorageResult<String> {
            let name = format!("{}.{}", self.hash(buffer), extension);
            let link = self.link(&name);
            let endpoint = format!("{}?extension={}&w={}&h={}", self.endpoint, extension, w, h);
            println!("UPLOADING to {}", endpoint);
            minreq::post(endpoint).with_body(buffer).send()?;
            println!("UPLOADED!");
            link
        }

        fn read_to(&self, name: &String, to: &mut Vec<u8>) -> StorageResult<()> {
            self.s3.read_to(name, to)
        }

        fn link(&self, name: &String) -> StorageResult<String> {
            self.s3.link(name)
        }
    }
}

pub mod img {
    use image::io::Reader;
    use scrap::{Capturer, Display};
    use std::io::ErrorKind::WouldBlock;
    use std::thread;
    use std::time::Duration;

    pub fn screenshot() -> (Vec<u8>, usize, usize) {
        let one_second = Duration::new(1, 0);
        let one_frame = one_second / 60;

        let display = Display::primary().expect("Couldn't find primary display.");
        let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
        let (w, h) = (capturer.width(), capturer.height());

        loop {
            // Wait until there's a frame.

            let buffer = match capturer.frame() {
                Ok(buffer) => buffer,
                Err(error) => {
                    if error.kind() == WouldBlock {
                        // Keep spinning.
                        thread::sleep(one_frame);
                        continue;
                    } else {
                        panic!("Error: {}", error);
                    }
                }
            };
            let mut bitflipped = Vec::with_capacity(w * h * 4);
            let stride = buffer.len() / h;

            for y in 0..h {
                for x in 0..w {
                    let i = stride * y + 4 * x;
                    bitflipped.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i], 255]);
                }
            }
            return (bitflipped, w, h);
        }
    }

    pub fn crop(path: &str, region: [f64; 4]) -> Option<(Vec<u8>, usize, usize)> {
        let mut image = Reader::open(path).ok()?.decode().ok()?;
        Some((
            image
                .crop(
                    region[0] as u32,
                    region[1] as u32,
                    region[2] as u32,
                    region[3] as u32,
                )
                .to_rgba8()
                .to_vec(),
            region[2] as usize,
            region[3] as usize,
        ))
    }
}
extern crate scrap;

use img::screenshot;
use std::convert::TryFrom;
use storage::Storage;
extern crate arboard;
use arboard::Clipboard;

pub mod server {
    use super::storage::{S3Store, Storage, StorageError};
    use rocket::Data;
    use std::io::Read;
    const LIMIT: u64 = 50000000000;

    #[rocket::post("/upload?<extension>&<w>&<h>", data = "<data>")]
    pub fn upload(extension: String, w: u32, h: u32, data: Data) -> Result<String, StorageError> {
        println!("UPLOADING");
        let mut buffer = Vec::new();
        data.open().take(LIMIT).read_to_end(&mut buffer)?;
        S3Store::new()?.save(&buffer, extension.into(), w, h)
    }
}

use notify_rust::{Hint, Notification};
fn notify(path: &str, message: &str) {
    let mut notif = Notification::new();
    notif
        .summary("Sampic screenshot taken.")
        .body(message)
        .icon("camera")
        .image_path(&path)
        .timeout(5)
        .sound_name("message-new-instant");
    #[cfg(target_os = "linux")]
    notif.hint(Hint::Transient(true));
    notif.show().expect("Notification Failure!");
}

pub fn sampic_screenshot<T: 'static + Storage + std::marker::Send>(storage: T) -> String {
    const EXTENSION: &str = "png";
    let (buffer, w, h) = screenshot();
    let local_storage = storage::Local::new();
    let fullscreenshot = local_storage.save(
        &buffer,
        EXTENSION.into(),
        u32::try_from(w).unwrap(),
        u32::try_from(h).unwrap(),
    ).unwrap();
    let region = region::get_region(&fullscreenshot);
    let (buffer, w, h) = img::crop(&fullscreenshot, region.unwrap()).unwrap();
    let name = format!("{}.{}", storage.hash(&buffer), EXTENSION);
    let destination = match storage.link(&name) {
        Ok(it) => it,
        _ => unreachable!(),
    };
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(destination.clone()).unwrap();
    notify(
        &destination,
        "Copied URL to clipboard. Uploading to server...",
    );
    storage
        .save(
            &buffer,
            EXTENSION.into(),
            u32::try_from(w).unwrap(),
            u32::try_from(h).unwrap(),
        )
        .unwrap_or("error while saving file".to_string());
    notify(&destination, "Uploaded!");
    return destination;
}

pub fn local_screenshot() -> String {
    sampic_screenshot(storage::Local::new())
}

pub fn s3_screenshot() -> String {
    sampic_screenshot(storage::S3Store::new().expect("Error while stablishing S3 connection"))
}

pub fn upload_screenshot() -> String {
    sampic_screenshot(
        storage::SampicServer::new()
            .expect("Error while stablishing connection with sampic server."),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
