use anyhow::{Context, Error, Result};
use serde::Deserialize;
use std::{collections::HashMap, env, fs};
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub jenkins: Jenkins,
    pub folder: HashMap<String, HashMap<String, BranchJobMapping>>,
}

type Branch = String;
type Job = String;
type BranchJobMapping = HashMap<Branch, Vec<Job>>;

#[derive(Deserialize, Debug)]
pub struct Jenkins {
    url: String,
    port: u32,
    pub api: String,
    pub username: String,
}

impl Config {
    pub fn load_config() -> Result<Self, Error> {
        let dir = env::current_exe()
            .context("failed to get current executable path")?
            .parent()
            .context("failed to get directory of exe")?
            .to_path_buf();

        let config_path = dir.join("config.toml");

        let config_str = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file at: {:?}", &config_path))?;

        let config: Config =
            toml::from_str(&config_str).context("Failed to parse config contents into TOML")?;

        config.validate()?;
        Ok(config)
    }

    pub fn find_jobs(&self, folder: &str, project: &str, branch: &str) -> Option<&Vec<String>> {
        self.folder
            .get(folder)
            .and_then(|projects| projects.get(project))
            .and_then(|branches| branches.get(branch))
    }

    fn validate(&self) -> Result<()> {
        self.jenkins
            .validate()
            .context("Invalid Jenkins configuration")?;
        if self.folder.is_empty() {
            anyhow::bail!("The 'folder' field must not be empty");
        }
        for (folder_name, projects) in &self.folder {
            if projects.is_empty() {
                anyhow::bail!(
                    "The projects for folder '{}' must not be empty",
                    folder_name
                );
            }
            for (project_name, branches) in projects {
                if branches.is_empty() {
                    anyhow::bail!(
                        "The branch mapping for project '{}' in folder '{}' must not be empty",
                        project_name,
                        folder_name
                    );
                }
                for (branch_name, jobs) in branches {
                    if jobs.is_empty() {
                        anyhow::bail!(
                            "The jobs list for branch '{}' in project '{}' in folder '{}' must not be empty",
                            branch_name,
                            project_name,
                            folder_name
                        );
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_folders(&self) -> Vec<&str> {
        self.folder.keys().map(|k| k.as_str()).collect()
    }

    pub fn get_projects(&self, folder: &str) -> Option<Vec<&str>> {
        self.folder
            .get(folder)
            .map(|projects| projects.keys().map(|k| k.as_str()).collect())
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
