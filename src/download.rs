use reqwest;
use std::fs::File;
use std::io;

pub async fn download(url: String, id: String, file_type: String) -> reqwest::Result<()> {
    let resp = reqwest::get(url).await?;
    if file_type == "image/png".to_string() {
        let mut file =
            File::create(format!("wallhaven-{}.png", id)).expect("Failed to create the file");
        let content = resp.bytes().await?;
        io::copy(&mut content.as_ref(), &mut file).expect("Failed to write data to the file");
    } else {
        let mut file =
            File::create(format!("wallhaven-{}.jpg", id)).expect("Failed to create the file");
        let content = resp.bytes().await?;
        io::copy(&mut content.as_ref(), &mut file).expect("Failed to write data to the file");
    }
    Ok(())
}
