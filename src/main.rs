use reqwest;
use serde_json::{Result, Value};

#[tokio::main]
async fn main() -> Result<()> {
    let body = reqwest::get("https://wallhaven.cc/api/v1/search?q=id:119949&ratios=landscape")
        .await
        .expect("Request failed")
        .text()
        .await
        .unwrap();

    let parsed_json: Value = serde_json::from_str(&body)?;
    //println!("{:#?}", parsed_json); debug option to check if json is parsed correctly and check the response structure
    // get the array of data objects
    let data_array = parsed_json["data"].as_array().unwrap();
    // print out image url for each object in returned json
    for object in data_array {
        println!("{}", object.to_owned()["path"]);
    }
    Ok(())
}
