// Asset Store CLI Tool
// 资源商店命令行工具

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json;
use dialoguer::{theme::ColorfulTheme, Select, Input};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;

#[derive(Parser)]
#[command(name = "asset-store")]
#[command(about = "Game Engine Asset Store CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 搜索资源
    Search {
        /// 搜索关键词
        query: String,
        /// 资源类型
        #[arg(short, long)]
        asset_type: Option<String>,
        /// 类别
        #[arg(short, long)]
        category: Option<String>,
        /// 标签
        #[arg(short = 't', long)]
        tags: Vec<String>,
        /// 每页结果数
        #[arg(short = 'n', long, default_value_t = 20)]
        per_page: usize,
    },
    /// 下载资源
    Download {
        /// 资源ID
        asset_id: String,
        /// 输出目录
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 获取资源详情
    Info {
        /// 资源ID
        asset_id: String,
    },
    /// 上传资源
    Upload {
        /// 资源清单文件(JSON)
        manifest: PathBuf,
        /// 资源文件
        files: Vec<PathBuf>,
    },
    /// 列出收藏
    Favorites {
        /// 用户ID
        #[arg(short, long)]
        user_id: Option<String>,
    },
    /// 下载历史
    History {
        /// 用户ID
        #[arg(short, long)]
        user_id: Option<String>,
    },
}

#[derive(Debug, serde::Deserialize)]
struct AssetMetadata {
    id: String,
    name: String,
    description: String,
    asset_type: String,
    category: String,
    version: String,
    author: String,
    tags: Vec<String>,
    license: String,
    #[serde(rename = "fileSizeBytes")]
    file_size_bytes: u64,
    rating: f32,
    #[serde(rename = "downloadCount")]
    download_count: u32,
    #[serde(rename = "previewUrls")]
    preview_urls: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SearchResult {
    assets: Vec<AssetMetadata>,
    #[serde(rename = "totalCount")]
    total_count: u32,
    page: u32,
    #[serde(rename = "perPage")]
    per_page: u32,
    #[serde(rename = "totalPages")]
    total_pages: u32,
}

#[derive(Debug, serde::Deserialize)]
struct AssetFile {
    filename: String,
    #[serde(rename = "fileType")]
    file_type: String,
    #[serde(rename = "sizeBytes")]
    size_bytes: u64,
    url: String,
    hash: String,
}

#[derive(Debug, serde::Deserialize)]
struct AssetData {
    metadata: AssetMetadata,
    files: Vec<AssetFile>,
}

struct AssetStoreClient {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl AssetStoreClient {
    fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn search(
        &self,
        query: &str,
        asset_type: Option<&str>,
        category: Option<&str>,
        tags: &[String],
        page: u32,
        per_page: u32,
    ) -> Result<SearchResult> {
        let mut url = format!("{}/api/v1/assets/search", self.base_url);
        let mut params = vec![
            ("query", query),
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ];

        if let Some(t) = asset_type {
            params.push(("asset_type", t));
        }
        if let Some(c) = category {
            params.push(("category", c));
        }
        for tag in tags {
            params.push(("tags", tag));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to send search request")?;

        if !response.status().is_success() {
            anyhow::bail!("Search request failed with status: {}", response.status());
        }

        let result: SearchResult = response
            .json()
            .await
            .context("Failed to parse search response")?;

        Ok(result)
    }

    async fn get_asset(&self, asset_id: &str) -> Result<AssetData> {
        let url = format!("{}/api/v1/assets/{}", self.base_url, asset_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch asset")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get asset with status: {}", response.status());
        }

        let asset: AssetData = response.json().await.context("Failed to parse asset data")?;

        Ok(asset)
    }

    async fn download_file(&self, url: &str, progress_bar: &ProgressBar) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to download file")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut data = Vec::new();
        let mut downloaded = 0;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            data.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                progress_bar.set_position(progress as u64);
            }
        }

        Ok(data)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let base_url = std::env::var("ASSET_STORE_URL")
        .unwrap_or_else(|_| "https://api.example.com".to_string());
    let api_key = std::env::var("ASSET_STORE_API_KEY").ok();

    let client = AssetStoreClient::new(base_url, api_key);

    match cli.command {
        Commands::Search {
            query,
            asset_type,
            category,
            tags,
            per_page,
        } => {
            println!("🔍 Searching for: {}", query);
            println!();

            let result = client
                .search(&query, asset_type.as_deref(), category.as_deref(), &tags, 1, per_page as u32)
                .await?;

            println!("Found {} assets:", result.total_count);
            println!();

            for (i, asset) in result.assets.iter().enumerate() {
                println!("{}. {}", i + 1, asset.name);
                println!("   ID: {}", asset.id);
                println!("   Type: {} | Category: {}", asset.asset_type, asset.category);
                println!("   Author: {}", asset.author);
                println!("   Rating: {:.1} | Downloads: {}", asset.rating, asset.download_count);
                if asset.pricing.type == "free" {
                    println!("   Price: Free");
                } else {
                    println!("   Price: ${}", asset.pricing.price_usd.unwrap_or(0.0));
                }
                println!("   Tags: {}", asset.tags.join(", "));
                println!();
            }
        }

        Commands::Download { asset_id, output } => {
            println!("⬇️  Downloading asset: {}", asset_id);

            let asset = client.get_asset(&asset_id).await?;
            let output_dir = output.unwrap_or_else(|| PathBuf::from(asset.metadata.name.clone()));

            fs::create_dir_all(&output_dir).await?;

            let pb = ProgressBar::new(100);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% {msg}")
                .progress_chars("##-"));

            for file in &asset.files {
                println!("Downloading: {}", file.filename);
                let file_path = output_dir.join(&file.filename);
                let data = client.download_file(&file.url, &pb).await?;

                let mut f = fs::File::create(&file_path).await?;
                f.write_all(&data).await?;

                pb.set_message(format!("Downloaded: {}", file.filename));
            }

            pb.finish_with_message("Download complete!");
            println!("✅ Asset downloaded to: {:?}", output_dir);
        }

        Commands::Info { asset_id } => {
            let asset = client.get_asset(&asset_id).await?;

            println!("📦 Asset Information");
            println!();
            println!("Name: {}", asset.metadata.name);
            println!("ID: {}", asset.metadata.id);
            println!("Version: {}", asset.metadata.version);
            println!("Author: {}", asset.metadata.author);
            println!();
            println!("Description:");
            println!("  {}", asset.metadata.description);
            println!();
            println!("Type: {}", asset.metadata.asset_type);
            println!("Category: {}", asset.metadata.category);
            println!("License: {}", asset.metadata.license);
            println!("Size: {} MB", asset.metadata.file_size_bytes / 1024 / 1024);
            println!("Rating: {:.1}", asset.metadata.rating);
            println!("Downloads: {}", asset.metadata.download_count);
            println!();
            println!("Tags:");
            for tag in &asset.metadata.tags {
                println!("  - {}", tag);
            }
            println!();
            println!("Files:");
            for file in &asset.files {
                println!("  - {} ({} KB)", file.filename, file.size_bytes / 1024);
            }
        }

        Commands::Upload { manifest, files } => {
            println!("📤 Uploading asset...");

            // 读取清单文件
            let manifest_content = fs::read_to_string(&manifest).await
                .context("Failed to read manifest file")?;
            let asset_data: AssetData = serde_json::from_str(&manifest_content)
                .context("Failed to parse manifest file")?;

            // 验证文件存在
            for file_path in &files {
                if !file_path.exists() {
                    anyhow::bail!("File not found: {:?}", file_path);
                }
            }

            // 构建上传请求
            let url = format!("{}/api/v1/assets/upload", client.base_url);

            // 创建multipart表单
            let mut form = reqwest::multipart::Form::new();

            // 添加清单
            form = form.part("manifest", reqwest::multipart::Part::text(manifest_content));

            // 添加文件
            for file_path in &files {
                let file_name = file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let file_content = fs::read(file_path).await
                    .context("Failed to read file")?;
                let part = reqwest::multipart::Part::bytes(file_content)
                    .file_name(file_name.to_string());
                form = form.part("files", part);
            }

            // 发送上传请求
            let mut request = client.client.post(&url);
            if let Some(api_key) = &client.api_key {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }
            request = request.multipart(form);

            let response = request.send().await
                .context("Failed to upload asset")?;

            if !response.status().is_success() {
                anyhow::bail!("Upload failed with status: {}", response.status());
            }

            let result: serde_json::Value = response.json().await
                .context("Failed to parse upload response")?;

            println!("✅ Upload complete!");
            if let Some(asset_id) = result.get("asset_id").and_then(|v| v.as_str()) {
                println!("Asset ID: {}", asset_id);
            }
        }

        Commands::Favorites { user_id } => {
            let uid = user_id.unwrap_or_else(|| {
                std::env::var("USER_ID").unwrap_or_else(|_| "default".to_string())
            });

            println!("❤️  Favorites for user: {}", uid);

            // 获取收藏列表
            let url = format!("{}/api/v1/users/{}/favorites", client.base_url, uid);

            let response = client.client.get(&url)
                .send()
                .await
                .context("Failed to fetch favorites")?;

            if !response.status().is_success() {
                if response.status() == 404 {
                    println!("No favorites found.");
                } else {
                    anyhow::bail!("Failed to fetch favorites: {}", response.status());
                }
                return Ok(());
            }

            let result: SearchResult = response.json().await
                .context("Failed to parse favorites response")?;

            if result.assets.is_empty() {
                println!("No favorites found.");
            } else {
                println!("Found {} favorite(s):", result.assets.len());
                println!();
                for (i, asset) in result.assets.iter().enumerate() {
                    println!("{}. {}", i + 1, asset.name);
                    println!("   ID: {}", asset.id);
                    println!("   Type: {}", asset.asset_type);
                    println!("   Rating: {:.1}", asset.rating);
                    println!();
                }
            }
        }

        Commands::History { user_id } => {
            let uid = user_id.unwrap_or_else(|| {
                std::env::var("USER_ID").unwrap_or_else(|_| "default".to_string())
            });

            println!("📜 Download history for user: {}", uid);

            // 获取下载历史
            let url = format!("{}/api/v1/users/{}/downloads", client.base_url, uid);

            let response = client.client.get(&url)
                .send()
                .await
                .context("Failed to fetch download history")?;

            if !response.status().is_success() {
                if response.status() == 404 {
                    println!("No download history found.");
                } else {
                    anyhow::bail!("Failed to fetch history: {}", response.status());
                }
                return Ok(());
            }

            #[derive(Debug, serde::Deserialize)]
            struct DownloadRecord {
                asset_id: String,
                asset_name: String,
                download_date: String,
                version: String,
            }

            let records: Vec<DownloadRecord> = response.json().await
                .context("Failed to parse history response")?;

            if records.is_empty() {
                println!("No download history found.");
            } else {
                println!("Found {} download(s):", records.len());
                println!();
                for (i, record) in records.iter().enumerate() {
                    println!("{}. {} (v{})", i + 1, record.asset_name, record.version);
                    println!("   ID: {}", record.asset_id);
                    println!("   Date: {}", record.download_date);
                    println!();
                }
            }
        }
    }

    Ok(())
}
