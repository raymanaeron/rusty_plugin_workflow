use crate::execution_plan::{ExecutionPlanLoader, PluginExecutionPlan};
use std::{fs, io::Write, path::PathBuf, error::Error};

/// Responsible for resolving and downloading updated execution plans.
pub struct ExecutionPlanUpdater;

impl ExecutionPlanUpdater {
    /// Downloads the latest execution plan if available.
    /// Returns the path to the file that should be used (either updated or original).
    pub fn fetch_and_prepare_latest(plan_path: &str) -> Result<PathBuf, Box<dyn Error>> {
        // Step 1: Load the base plan first (always required)
        let base_plan = ExecutionPlanLoader::load_from_file(plan_path)?;

        let general = &base_plan.general;
        let remote_path = Self::build_remote_path(
            &general.update_path_root,
            &general.product_family,
            &general.execution_plan_version,
        );

        match general.update_from.as_str() {
            "s3" => {
                println!("Checking S3 for updated execution plan: {}", remote_path);
                match Self::download_from_https_url(&remote_path) {
                    Ok(updated_path) => {
                        println!("Using updated execution plan from: {}", updated_path.display());
                        Ok(updated_path)
                    }
                    Err(err) => {
                        eprintln!("S3 update failed, using local plan. Error: {}", err);
                        Ok(PathBuf::from(plan_path))
                    }
                }
            }
            "local" | "unc" => {
                println!("Using override path: {}", remote_path);
                if PathBuf::from(&remote_path).exists() {
                    Ok(PathBuf::from(remote_path))
                } else {
                    eprintln!("Override path not found: {}, using local plan", remote_path);
                    Ok(PathBuf::from(plan_path))
                }
            }
            other => Err(format!("Unsupported update_from value: {}", other).into()),
        }
    }

    /// Constructs the full remote path to the new execution_plan.toml
    fn build_remote_path(root: &str, product: &str, version: &str) -> String {
        let mut fixed = root.trim_end_matches('/').to_string();
        fixed.push('/');
        fixed.push_str(product);
        fixed.push('/');
        fixed.push_str(version);
        fixed.push_str("/execution_plan.toml");
        fixed
    }

    /// Downloads the file from an HTTPS S3 URL and stores it locally in temp dir.
    fn download_from_https_url(url: &str) -> Result<PathBuf, Box<dyn Error>> {
        let response = ureq::get(url).call()?;
        if response.status() != 200 {
            return Err(format!("HTTP GET failed with status {}", response.status()).into());
        }

        let content = response.into_string()?;
        let tmp_path = std::env::temp_dir().join("execution_plan.override.toml");
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(content.as_bytes())?;
        Ok(tmp_path)
    }
}
