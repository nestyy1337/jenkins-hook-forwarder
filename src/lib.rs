mod config;
pub mod logging;
use axum::http::HeaderMap;
use axum::{Json, Router, routing::post};
use config::Config;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

lazy_static! {
    static ref CONFIG: Config = Config::load_config().expect("Failed to initialize config");
}

pub struct App {
    listener: TcpListener,
    app: Router,
}

impl App {
    pub async fn create_app() -> App {
        let app = Router::new().route("/", post(handle_hook));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .expect("Port 3000 is already taken");

        info!("Application is running on 127.0.0.1:3000");

        App { listener, app }
    }

    pub async fn run(self) {
        if let Err(e) = axum::serve(self.listener, self.app).await {
            error!("Application encountered an error: {:?}", e);
        }
    }
}

#[derive(Deserialize, Debug)]
struct HookPayload {
    #[serde(rename = "ref")]
    ref_field: String,
    repository: Repository,
}

#[derive(Deserialize, Debug)]
struct Repository {
    name: String,
}

async fn handle_hook(event_type: HeaderMap, Json(payload): Json<serde_json::Value>) {
    if let Some(event) = event_type.get("X-Github-Event") {
        if event != "push" {
            warn!("Github event is not a push: {:?}", event);
            return;
        }
    } else {
        warn!("Missing X-Github-Event header");
        return;
    }

    let hook: HookPayload = match serde_json::from_value(payload) {
        Ok(hook) => hook,
        Err(e) => {
            error!("Failed to parse JSON payload: {:?}", e);
            return;
        }
    };

    let branch = hook.ref_field.trim_start_matches("refs/heads/");
    let repo_name = hook.repository.name;

    let jenkins_url = CONFIG.jenkins.get_url();
    let username = CONFIG.jenkins.username.clone();
    let api_key = CONFIG.jenkins.api.clone();

    for (folder_name, projects) in &CONFIG.folder {
        for (project_name, _) in projects {
            if project_name == &repo_name {
                info!(
                    "Received push to {}/{} branch: {}",
                    folder_name, project_name, branch
                );
                if let Some(jobs) = CONFIG.find_jobs(folder_name, project_name, branch) {
                    for job in jobs {
                        let client = reqwest::Client::builder()
                            .danger_accept_invalid_certs(true)
                            .build()
                            .expect("Failed to build reqwest client");

                        let mut params = HashMap::new();
                        params.insert("token", api_key.clone());

                        info!(
                            "Triggering build at {}/{}/{}/buildWithParameters",
                            &jenkins_url, &folder_name, &job
                        );

                        let res = client
                            .get(format!(
                                "{}/{}/{}/buildWithParameters",
                                &jenkins_url, &folder_name, &job
                            ))
                            .basic_auth(&username, Some(&api_key))
                            .query(&params)
                            .send()
                            .await;

                        match res {
                            Ok(response) => {
                                info!(
                                    "Build '{}' has been triggered by push to {}/{}",
                                    job, repo_name, branch
                                );
                                info!("Response: {:?}", response);
                            }
                            Err(e) => {
                                error!("Failed to trigger build for {}: {:?}", job, e);
                            }
                        }
                    }
                } else {
                    warn!(
                        "No jobs found for repo '{}' and branch '{}'",
                        repo_name, branch
                    );
                }
                return;
            }
        }
    }

    // we couldnt find a matched project
    info!("No project found for repo '{}'", repo_name);
}
