use clap::Parser;
use rand::Rng;
use reqwest;
use serde_json::Value;
use std::{env, fmt::Debug, thread};

mod download;
use download::download;

#[derive(Parser, Debug)]
#[command(name = "whdl")]
#[command(version = "1.2")]
#[command(author = "Moskas minemoskas@gmail.com")]
#[command(
    about = "Wallhaven.cc wallpaper downloader",
    long_about = "Wallhaven.cc wallpaper downloader. For exact values and query format checkout official api docs https://wallhaven.cc/help/api"
)]
struct Args {
    /// Query to search for
    #[arg(short, long)]
    query: String,
    ///  List of aspect ratios, can be a list comma separated
    #[arg(short = 'R', long)]
    ratios: Option<String>,
    /// Exact resolution(s), can be a list comma separated
    #[arg(short = 'r', long)]
    resolution: Option<String>,
    /// Minimal resolution to search for
    #[arg(short, long)]
    atleast: Option<String>,
    /// Purity filter in xxx format
    #[arg(
        short,
        long,
        default_value = "100",
        help = "100/110/111 (sfw/sketchy/nsfw)"
    )]
    purity: Option<String>,
    /// Categories in xxx format
    /// 100/101/111 (general/anime/people)
    #[arg(short, long, default_value = "111")]
    category: Option<String>,
    /// Method of sorting results, possible values:
    /// date_added, relevance, random, views, favorites, toplist
    #[arg(short, long, default_value = "date_added")]
    sorting: Option<String>,
    /// Order of sorting results, possible values: desc, asc
    #[arg(short, long, default_value = "desc")]
    order: Option<String>,
    /// Colors to search for
    #[arg(short = 'C', long)]
    colors: Option<String>,
    /// Download from specified page of results
    #[arg(short = 'P', long)]
    page: Option<String>,
    // /// Download Specified amount of images
    // amount: Option<i32>,
    // /// Download directory
    // #[arg(short, long)]
    // directory: Option<String>,
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
    if let Some(resolution) = &args.resolution {
        url.push_str(&format!("&resolutions={}", resolution));
    }
    if let Some(atleast) = &args.atleast {
        url.push_str(&format!("&atleast={}", atleast));
    }
    if let Some(ratios) = &args.ratios {
        url.push_str(&format!("&ratios={}", ratios));
    }
    if let Some(category) = &args.category {
        url.push_str(&format!("&categories={}", category));
    }
    if let Some(page) = &args.page {
        url.push_str(&format!("&page={}", page));
    }
    if let Some(sorting) = &args.sorting {
        if sorting == "random" {
            let seed = rand::thread_rng().gen_range(100_000..1_000_000);
            url.push_str(&format!("&seed={}", seed));
        }
        url.push_str(&format!("&sorting={}", sorting));
    }
    if let Some(colors) = &args.colors {
        url.push_str(&format!("&colors={}", colors));
    }

    let body = reqwest::get(&url).await?.text().await?;

    let parsed_json: Value = serde_json::from_str(&body).unwrap();

    let data_array = parsed_json["data"].as_array().unwrap();
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
