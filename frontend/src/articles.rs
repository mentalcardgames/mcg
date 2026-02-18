use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "userId")]
    pub user_id: u32,
    pub id: u32,
    pub title: String,
    pub body: String,
}

pub async fn fetch_posts() -> Result<Vec<Post>, String> {
    let url = "https://jsonplaceholder.typicode.com/posts/";

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch posts: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let posts: Vec<Post> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(posts)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_posts_blocking() -> Result<Vec<Post>, String> {
    let url = "https://jsonplaceholder.typicode.com/posts/";
    let response = reqwest::blocking::get(url)
        .map_err(|e| format!("Failed to fetch posts (blocking): {}", e))?;
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    let posts: Vec<Post> = response
        .json()
        .map_err(|e| format!("Failed to parse JSON (blocking): {}", e))?;
    Ok(posts)
}
