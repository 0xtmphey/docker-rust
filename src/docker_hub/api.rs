use flate2::read::GzDecoder;
use std::path::PathBuf;
use tar::Archive;

use super::models::{Manifest, TokenResponse};

const AUTH_BASE_URL: &str =
    "https://auth.docker.io/token?service=registry.docker.io&scope=repository:library";
const REGISTRY_BASE_URL: &str = "https://registry.hub.docker.com/v2/library";

pub async fn download_image(to_path: &PathBuf, image: &str) -> anyhow::Result<()> {
    let mut parts = image.split(':');
    let image_name = parts.next().unwrap();
    let image_tag = parts.next().unwrap_or("latest");

    let image_url = format!("{AUTH_BASE_URL}/{image_name}:pull");
    let image_manifest_url = format!("{REGISTRY_BASE_URL}/{image_name}/manifests/{image_tag}");

    let http_client = reqwest::Client::new();

    let token: String = http_client
        .get(image_url)
        .send()
        .await?
        .json::<TokenResponse>()
        .await?
        .token;

    let manifest = http_client
        .get(image_manifest_url)
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?
        .json::<Manifest>()
        .await?;

    for layer in manifest.layers.iter() {
        let layer_url = format!("{REGISTRY_BASE_URL}/{image_name}/blobs/{}", layer.digest);

        let layer_bytes = http_client
            .get(layer_url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await?
            .bytes()
            .await?;

        let decompress_stream = GzDecoder::new(&layer_bytes[..]);
        let mut archive = Archive::new(decompress_stream);

        archive.unpack(to_path)?;
    }

    Ok(())
}
