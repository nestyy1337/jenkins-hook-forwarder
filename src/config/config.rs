use anyhow::{Context, Error, Result};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{collections::HashMap, env, fs};
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub jenkins: Jenkins,
    pub repos: HashMap<String, Repo>,
}

#[derive(Deserialize, Debug)]
pub struct Jenkins {
    url: String,
    port: u32,
    pub api: String,
    pub username: String,
}

type Branch = String;
type Job = String;

#[derive(Deserialize, Debug)]
struct Repo {
    branch_job_mapping: HashMap<Branch, Job>,
}

impl Config {
    pub fn load_config() -> Result<Self, Error> {
        let exe_path = env::current_exe().context("Failed to get current executable path")?;
        let dir = exe_path
            .parent()
            .context("Failed to get directory of executable")?;
        let config_path = dir.join("config.toml");

        let config_str = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file at: {:?}", &config_path))?;

        let config: Config =
            toml::from_str(&config_str).context("Failed to parse config contents into TOML")?;

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        self.jenkins
            .validate()
            .context("Invalid Jenkins configuration")?;
        if self.repos.is_empty() {
            anyhow::bail!("The 'repos' field must not be empty");
        }
        for (repo_name, repo) in &self.repos {
            if repo.branch_job_mapping.is_empty() {
                anyhow::bail!(
                    "The 'branch_job_mapping' for repo '{}' must not be empty",
                    repo_name
                );
            }
        }
        Ok(())
    }

    pub fn get_repos(&self) -> Vec<&str> {
        self.repos.keys().map(|k| k.as_str()).collect()
    }

    pub fn find_job(&self, repo: &str, branch: &str) -> Option<&String> {
        self.repos
            .get(repo)
            .and_then(|branches| branches.branch_job_mapping.get(branch))
    }
}

impl Jenkins {
    pub fn validate(&self) -> Result<()> {
        if self.url.trim().is_empty() {
            anyhow::bail!("The 'url' field in Jenkins must not be empty");
        }
        if self.api.trim().is_empty() {
            anyhow::bail!("The 'api' field in Jenkins must not be empty");
        }
        if self.username.trim().is_empty() {
            anyhow::bail!("The 'username' field in Jenkins must not be empty");
        }
        Ok(())
    }

    pub fn get_url(&self) -> String {
        format!("{}:{}/job", self.url, self.port)
    }
}

impl Repo {
    pub fn get_branches(&self) -> Vec<&str> {
        self.branch_job_mapping.keys().map(|k| k.as_str()).collect()
    }

    pub fn get_jobs(&self) -> Vec<&str> {
        self.branch_job_mapping
            .values()
            .map(|v| v.as_str())
            .collect()
    }
}
