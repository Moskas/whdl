use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde_json::Value;
use std::{env, fmt::Debug, path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;

mod download;
use download::{download, DownloadStatus};

const PAGE_SIZE: usize = 24;

#[derive(Parser, Debug)]
#[command(name = "whdl")]
#[command(version = "2.0")]
#[command(author = "Moskas minemoskas@gmail.com")]
#[command(
  about = "Wallhaven.cc wallpaper downloader",
  long_about = "Wallhaven.cc wallpaper downloader. For exact values and query format checkout official api docs https://wallhaven.cc/help/api"
)]
struct Args {
  /// Query to search for
  #[arg(short, long)]
  query: String,
  /// List of aspect ratios, can be a list comma separated
  #[arg(short = 'R', long)]
  ratios: Option<String>,
  /// Exact resolution(s), can be a list comma separated
  #[arg(short = 'r', long)]
  resolution: Option<String>,
  /// Minimal resolution to search for
  #[arg(short, long)]
  atleast: Option<String>,
  /// Purity filter in xxx format: 100/110/111 (sfw/sketchy/nsfw)
  #[arg(short, long, default_value = "100")]
  purity: Option<String>,
  /// Categories in xxx format: 100/010/001 (general/anime/people)
  #[arg(short, long, default_value = "111")]
  category: Option<String>,
  /// Method of sorting results: date_added, relevance, random, views, favorites, toplist
  #[arg(short, long, default_value = "date_added")]
  sorting: Option<String>,
  /// Order of sorting results: desc, asc
  #[arg(short, long, default_value = "desc")]
  order: Option<String>,
  /// Colors to search for
  #[arg(short = 'C', long)]
  colors: Option<String>,
  /// Download a specific single page of results (conflicts with --count)
  #[arg(short = 'P', long, conflicts_with = "count")]
  page: Option<usize>,
  /// Total wallpapers to download, fetching multiple pages as needed.
  /// Wallhaven returns 24 per page, so --count 50 fetches 3 pages and keeps
  /// the first 50. Conflicts with --page.
  #[arg(short = 'n', long, conflicts_with = "page")]
  count: Option<usize>,
  /// Directory to save wallpapers into
  #[arg(short, long, default_value = ".")]
  directory: PathBuf,
  /// Show what would be downloaded without saving anything
  #[arg(long)]
  dry_run: bool,
  /// Maximum number of concurrent downloads
  #[arg(short = 'j', long, default_value = "4")]
  jobs: usize,
}

/// Build the base search URL without a page number so callers can append
/// `&page=N` themselves. The random seed is chosen once here so all pages
/// in a single run share the same shuffle.
fn build_base_url(args: &Args, api_key: &str) -> String {
  let prefix = if api_key.is_empty() {
    "https://wallhaven.cc/api/v1/search?".to_string()
  } else {
    format!("https://wallhaven.cc/api/v1/search?apikey={}&", api_key)
  };

  let mut url = format!("{}q={}", prefix, args.query.replace('"', ""));

  macro_rules! push_param {
    ($opt:expr, $key:expr) => {
      if let Some(val) = &$opt {
        url.push_str(&format!("&{}={}", $key, val));
      }
    };
  }

  push_param!(args.purity, "purity");
  push_param!(args.resolution, "resolutions");
  push_param!(args.atleast, "atleast");
  push_param!(args.ratios, "ratios");
  push_param!(args.category, "categories");
  push_param!(args.order, "order");
  push_param!(args.colors, "colors");

  if let Some(sorting) = &args.sorting {
    if sorting == "random" {
      let seed = rand::random_range(100_000..1_000_000);
      url.push_str(&format!("&seed={}", seed));
    }
    url.push_str(&format!("&sorting={}", sorting));
  }

  url
}

async fn fetch_page(
  client: &reqwest::Client,
  base_url: &str,
  page: usize,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}&page={}", base_url, page);
  let body = client.get(&url).send().await?.text().await?;
  let parsed: Value = serde_json::from_str(&body)
    .map_err(|_| "Failed to parse API response — is your API key valid?")?;
  let data = parsed["data"]
    .as_array()
    .ok_or("Unexpected API response format")?
    .clone();
  Ok(data)
}

async fn fetch_wallpapers(args: &Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let api_key = env::var("WALLHAVEN_API_KEY")
    .unwrap_or_default()
    .replace('"', "");
  let client = reqwest::Client::new();
  let base_url = build_base_url(args, &api_key);

  let spinner = ProgressBar::new_spinner();
  spinner
    .set_style(ProgressStyle::with_template("{spinner:.cyan} {msg}")?.tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "));
  spinner.enable_steady_tick(std::time::Duration::from_millis(80));

  let wallpapers: Vec<Value> = if let Some(count) = args.count {
    // Calculate how many pages we need, fetch them all concurrently
    // (they're cheap metadata requests), then truncate to exactly count.
    let pages_needed = count.div_ceil(PAGE_SIZE);
    spinner.set_message(format!(
      "Querying Wallhaven API ({} page(s) for {} wallpapers)...",
      pages_needed, count
    ));

    let mut handles = Vec::with_capacity(pages_needed);
    for page in 1..=pages_needed {
      let client = client.clone();
      let base_url = base_url.clone();
      handles.push(tokio::spawn(async move {
        fetch_page(&client, &base_url, page).await
      }));
    }

    let mut all = Vec::with_capacity(pages_needed * PAGE_SIZE);
    for handle in handles {
      match handle.await? {
        Ok(mut page_data) => all.append(&mut page_data),
        Err(e) => eprintln!("  warning: failed to fetch a page: {}", e),
      }
    }
    all.truncate(count);
    all
  } else {
    // Single explicit page, or page 1 as default.
    let page = args.page.unwrap_or(1);
    spinner.set_message(format!("Querying Wallhaven API (page {})...", page));
    fetch_page(&client, &base_url, page).await?
  };

  spinner.finish_and_clear();
  println!("Found {} wallpaper(s)", wallpapers.len());

  if args.dry_run {
    println!("\nDry run — no files will be saved:\n");
    for w in &wallpapers {
      let id = w["id"].as_str().unwrap_or("?");
      let res = w["resolution"].as_str().unwrap_or("?");
      let url = w["path"].as_str().unwrap_or("?");
      println!("  [{id}] {res:<12} {url}");
    }
    return Ok(());
  }

  tokio::fs::create_dir_all(&args.directory).await?;
  download_wallpapers(&client, &wallpapers, args).await
}

async fn download_wallpapers(
  client: &reqwest::Client,
  wallpapers: &[Value],
  args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let multi = Arc::new(MultiProgress::new());
  let semaphore = Arc::new(Semaphore::new(args.jobs));
  let mut handles = Vec::with_capacity(wallpapers.len());

  for w in wallpapers {
    let url = w["path"].as_str().unwrap().to_string();
    let id = w["id"].as_str().unwrap().to_string();
    let file_type = w["file_type"].as_str().unwrap().to_string();

    let client = client.clone();
    let directory = args.directory.clone();
    let multi = Arc::clone(&multi);
    let sem = Arc::clone(&semaphore);
    let pb = multi.add(ProgressBar::new(0));

    handles.push(tokio::spawn(async move {
      // Permit is released automatically when _permit is dropped at end
      // of this block, allowing the next queued task to proceed.
      let _permit = sem.acquire().await.unwrap();
      download(&client, url, id, &file_type, &directory, pb).await
    }));
  }

  let (mut downloaded, mut skipped, mut failed) = (0usize, 0usize, 0usize);
  let mut failures: Vec<String> = Vec::new();

  for handle in handles {
    match handle.await? {
      Ok(DownloadStatus::Downloaded) => downloaded += 1,
      Ok(DownloadStatus::Skipped) => skipped += 1,
      Err(e) => {
        failed += 1;
        failures.push(e.to_string());
      }
    }
  }

  println!(
    "\n  ✓ {} downloaded   ⏭ {} skipped   ✗ {} failed",
    downloaded, skipped, failed
  );
  for f in &failures {
    eprintln!("  error: {}", f);
  }

  Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let args = Args::parse();
  fetch_wallpapers(&args).await
}
