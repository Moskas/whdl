use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub enum DownloadStatus {
  Downloaded,
  Skipped,
}

pub async fn download(
  client: &Client,
  url: String,
  id: String,
  file_type: &str,
  directory: &Path,
  pb: ProgressBar,
) -> Result<DownloadStatus, Box<dyn std::error::Error + Send + Sync>> {
  let ext = if file_type == "image/png" {
    "png"
  } else {
    "jpg"
  };
  let filename = format!("wallhaven-{}.{}", id, ext);
  let path = directory.join(&filename);

  if path.exists() {
    pb.set_style(ProgressStyle::with_template("  {msg}")?);
    pb.finish_with_message(format!("⏭  {} — already exists", filename));
    return Ok(DownloadStatus::Skipped);
  }

  let resp = client.get(&url).send().await?;

  if !resp.status().is_success() {
    let status = resp.status();
    pb.finish_with_message(format!("✗  {} — HTTP {}", filename, status));
    return Err(format!("{}: HTTP {}", filename, status).into());
  }

  let total_size = resp.content_length().unwrap_or(0);

  // Use a determinate bar if we know the size, otherwise a spinner
  if total_size > 0 {
    pb.set_length(total_size);
    pb.set_style(
            ProgressStyle::with_template(
                "  {msg}\n  {spinner:.cyan} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, eta {eta})",
            )?
            .progress_chars("█▓░")
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
  } else {
    pb.set_style(
      ProgressStyle::with_template("  {msg}\n  {spinner:.cyan} {bytes} ({bytes_per_sec})")?
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
  }
  pb.set_message(filename.clone());

  let mut file = File::create(&path).await?;
  let mut stream = resp.bytes_stream();

  while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    file.write_all(&chunk).await?;
    pb.inc(chunk.len() as u64);
  }

  pb.finish_with_message(format!("✓  {}", filename));
  Ok(DownloadStatus::Downloaded)
}
