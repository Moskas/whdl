use reqwest;
use std::fs::File;
use std::io::copy;
use tempfile::Builder;

#[tokio::main]
pub async fn download(url: String) -> Result<(), reqwest::Error> {
    let tmp_dir = Builder::new().prefix("example").tempdir().unwrap();
    let target = url;
    let response = reqwest::get(target).await?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        println!("file to download: '{}'", fname);
        let fname = tmp_dir.path().join(fname);
        println!("will be located under: '{:?}'", fname);
        File::create(fname).unwrap()
    };
    let content = response.text().await?;
    std::io::copy(&mut content.as_bytes(), &mut dest)?;
    Ok(())
}
