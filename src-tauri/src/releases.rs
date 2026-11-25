use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
}

#[derive(Serialize)]
pub struct Available {
    version: String,
}

fn version(tag: &str) -> Option<semver::Version> {
    semver::Version::parse(
        tag.strip_prefix("preview-")
            .or_else(|| tag.strip_prefix('v'))
            .unwrap_or(tag),
    )
    .ok()
}

#[tauri::command]
pub async fn github_update(app: tauri::AppHandle) -> Result<Option<Available>, String> {
    let releases: Vec<Release> = reqwest::Client::builder()
        .user_agent("PDFViewer-Windows-Update-Check")
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?
        .get("https://api.github.com/repos/Dracylfrr/pdfviewer/releases?per_page=30")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let current = &app.package_info().version;
    Ok(releases
        .into_iter()
        .filter(|release| !release.draft)
        .filter_map(|release| version(&release.tag_name))
        .filter(|candidate| candidate > current)
        .max()
        .map(|version| Available {
            version: version.to_string(),
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_project_tags_and_compares_components_numerically() {
        assert_eq!(version("preview-0.2.0"), version("v0.2.0"));
        assert!(version("preview-0.10.0").unwrap() > version("preview-0.2.0").unwrap());
        assert!(version("nightly").is_none());
    }
}
