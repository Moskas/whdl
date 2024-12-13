use clap::Parser;
use rand::Rng;
use reqwest;
use serde_json::Value;
use std::{env, fmt::Debug, thread};

mod download;
use download::download;

#[derive(Parser, Debug)]
#[command(name = "whdl")]
#[command(version = "1.1")]
#[command(author = "Moskas minemoskas@gmail.com")]
#[command(about = "Wallhaven.cc wallpaper downloader", long_about = None)]
struct Args {
    #[arg(short, long)]
    query: String,
    #[arg(short, long)]
    ratios: Option<String>,
    #[arg(short, long)]
    iresolution: Option<String>,
    #[arg(short, long)]
    mresolution: Option<String>,
    #[arg(short, long)]
    purity: Option<String>,
    #[arg(short, long)]
    category: Option<String>,
    #[arg(short, long)]
    sorting: Option<String>,
    #[arg(short, long)]
    ai_filter: Option<bool>,
    #[arg(short, long, default_value = "desc")]
    order: Option<String>,
    #[arg(short, long)]
    exact_page: Option<String>,
    amount: Option<i32>,
}

async fn fetch_wallpapers(args: &Args) -> reqwest::Result<()> {
    let api_key = env::var("WALLHAVEN_API_KEY")
        .unwrap_or_else(|_| "".to_string())
        .replace("\"", "");

    let mut url = if api_key.is_empty() {
        "https://wallhaven.cc/api/v1/search?".to_string()
    } else {
        format!("https://wallhaven.cc/api/v1/search?apikey={}", api_key)
    };

    url.push_str(&format!("&q={}", args.query.replace("\"", "")));
    if let Some(purity) = &args.purity {
        url.push_str(&format!("&purity={}", purity));
    }
    if let Some(iresolution) = &args.iresolution {
        url.push_str(&format!("&resolutions={}", iresolution));
    }
    if let Some(mresolution) = &args.mresolution {
        url.push_str(&format!("&atleast={}", mresolution));
    }
    if let Some(ratios) = &args.ratios {
        url.push_str(&format!("&ratios={}", ratios));
    }
    if let Some(category) = &args.category {
        url.push_str(&format!("&categories={}", category));
    }
    if let Some(ai_filter) = args.ai_filter {
        let ai_art_filter = if ai_filter { "0" } else { "1" };
        url.push_str(&format!("&ai_art_filter={}", ai_art_filter));
    }
    if let Some(exact_page) = &args.exact_page {
        url.push_str(&format!("&page={}", exact_page));
    }
    if let Some(sorting) = &args.sorting {
        let mut url = format!("&sort={}", sorting);
        if sorting == "random" {
            let seed = rand::thread_rng().gen_range(100_000..1_000_000);
            url.push_str(&format!("&seed={}", seed));
        }
        url.push_str(&format!("&sort={}", sorting));
    }

    let body = reqwest::get(&url)
        .await?
        .text()
        .await?;

    let parsed_json: Value = serde_json::from_str(&body).unwrap();

    let data_array = parsed_json["data"].as_array().unwrap();
    println!("{}",&data_array.len());
    download_wallpapers(data_array).await.unwrap();

    Ok(())
}

async fn download_wallpapers(data_array: &Vec<Value>) -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = vec![];

    for object in data_array {
        let url = object["path"].as_str().unwrap().to_string();
        let id = object["id"].as_str().unwrap().to_string();
        let file_type = object["file_type"].as_str().unwrap().to_string();

        let handle = thread::spawn(move || {
            println!("Downloading: {}", url);
            download(
                url.replace("\"", ""),
                id.replace("\"", ""),
                file_type.replace("\"", ""),
            )
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap().await?;
    }
    println!("Download complete");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    fetch_wallpapers(&args).await?;
    Ok(())
}
