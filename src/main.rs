use clap::Parser;
use rand;
use reqwest;
use serde_json::Value;
use std::{env, fmt::Debug};
use subprocess::{self, Popen, PopenConfig};

#[derive(Parser, Debug)]
#[command(name = "whdl")]
#[command(version = "1.0")]
#[command(author = "Moskas minemoskas@gmail.com")]
#[command(about="Wallhaven.cc wallpaper downloader",long_about=None)]
struct Args {
    #[arg(short, long)]
    ///The query to search on wallhaven.cc
    query: String,
    #[arg(short, long)]
    ///Set the image ratios
    ratios: Option<String>,
    #[arg(short, long)]
    ///Set the exact image resolution
    iresolution: Option<String>,
    #[arg(short, long)]
    ///Set the minimal image resolution
    mresolution: Option<String>,
    #[arg(short, long)]
    ///Set the purity sfw/sketchy/nsfw
    purity: Option<String>,
    #[arg(short, long)]
    ///Set the category general/anime/people
    category: Option<String>,
    #[arg(short, long)]
    ///Set the sorting of the results
    sorting: Option<String>,
    #[arg(short, long, default_value = "desc")]
    ///Set the sorting order
    order: Option<String>,
}

//async fn download_wallpaper(url: String) -> reqwest<()> {
//    println!("{url}");
//    let tmp_dir = Builder::new().prefix("wallhaven-").tempdir().unwrap();
//    let target = "https://www.rust-lang.org/logos/rust-logo-512x512.png";
//    let response = reqwest::get(target).await?;
//
//    let mut dest = {
//        let fname = response
//            .url()
//            .path_segments()
//            .and_then(|segments| segments.last())
//            .and_then(|name| if name.is_empty() { None } else { Some(name) })
//            .unwrap_or("tmp.bin");
//
//        println!("file to download: '{}'", fname);
//        let fname = tmp_dir.path().join(fname);
//        println!("will be located under: '{:?}'", fname);
//        File::create(fname).unwrap()
//    };
//    let content = response.text().await?;
//    std::io::copy(&mut content.as_bytes(), &mut dest);
//    Ok(())
//}
async fn download(url: String) -> subprocess::Result<()> {
    let mut wget = Popen::create(
        &["wget", &url],
        PopenConfig {
            stdout: subprocess::Redirection::Pipe,
            ..Default::default()
        },
    )?;
    let (out, err) = wget.communicate(None)?;
    println!("{:?}, {:?}", out, err);
    //if let Some(exit_status) = wget.poll() {
    //    // the process has finished
    //} else {
    //    // it is still running, terminate it
    //    wget.terminate()?;
    //}
    Ok(())
}

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    let api_key = match env::var("WALLHAVEN_API_KEY") {
        Ok(key) => key,
        Err(_e) => "".to_string(), // return empty string if no api key is set
    };
    let mut url: String = if api_key == "" {
        "https://wallhaven.cc/api/v1/search?".to_string()
    } else {
        format!("https://wallhaven.cc/api/v1/search?apikey={api_key}")
    };
    let args = Args::parse();
    url.push_str(&(format!("&q={}", args.query)));
    if args.purity != None {
        url.push_str(&(format!("&purity={}", args.purity.clone().unwrap())))
    }
    if args.mresolution != None {
        url.push_str(&(format!("&atleast={}", args.mresolution.clone().unwrap())))
    }
    if args.ratios != None {
        url.push_str(&(format!("&ratios={}", args.ratios.clone().unwrap())))
    }
    if args.category != None {
        url.push_str(&(format!("&categories={}", args.category.clone().unwrap())))
    }
    if args.purity != None {
        url.push_str(&(format!("&purity={}", args.purity.clone().unwrap())))
    }
    if args.sorting != None {
        let seed = rand::random::<u16>();
        println!("{seed}");
        if args.sorting.clone().unwrap() == "random" {
            url.push_str(&(format!("&sort={}&seed={}", args.sorting.clone().unwrap(), seed)))
        } else {
            url.push_str(&(format!("&sort={}", args.sorting.clone().unwrap())))
        }
    }
    println!("{}", url); // for debugging url
    let body = reqwest::get(url)
        .await
        .expect("Request failed")
        .text()
        .await
        .unwrap();

    let parsed_json: Value = serde_json::from_str(&body).unwrap();
    //println!("{:#?}", parsed_json); // debug option to check if json is parsed correctly and check the response structure

    // get the array of data objects
    let data_array = parsed_json["data"].as_array().unwrap();
    // print out image url for each object in returned json
    for object in data_array {
        println!("{}", object.to_owned()["path"]);
        //download(object.to_owned()["path"].to_owned().to_string()).await;
    }
    //println!("{api_key}");
    //println!("{args:?}");
    Ok(())
}
