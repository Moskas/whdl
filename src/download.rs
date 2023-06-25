use reqwest;
use std::io;
use std::fs::File;

pub async fn download(url: String, id: String, file_type: String) -> reqwest::Result<()> {
    let resp = reqwest::get(url).await?;
    //println!("{resp:?}");
    if file_type == "image/png".to_string() {
        let mut file = File::create(format!("{}.png", id)).expect("Failed to create the file");
        let content = resp.bytes().await?;
        io::copy(&mut content.as_ref(), &mut file).expect("Failed to write data to the file");
    } else {
        let mut file = File::create(format!("{}.jpg", id)).expect("Failed to create the file");
        let content = resp.bytes().await?;
        io::copy(&mut content.as_ref(), &mut file).expect("Failed to write data to the file");
    }
    Ok(())
}
