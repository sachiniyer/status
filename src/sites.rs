use axum::{extract::ws::WebSocket, response::Json};
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::{env, fmt};
use tokio::sync::mpsc;
use tokio::task;
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

#[derive(Debug)]
struct NginxError {
    message: String,
}

impl Error for NginxError {}

impl fmt::Display for NginxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SiteResponse {
    pub site: String,
    pub status: bool,
}

pub async fn handle_ws(mut stream: WebSocket) {
    let var_map: HashMap<String, String> = env::vars().collect();
    let sites = get_nginx(var_map.get("NGINX").unwrap().to_string()).await;
    match sites {
        Ok(sites) => {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let mut tasks: Vec<JoinHandle<()>> = vec![];
            for site in sites {
                let tx = tx.clone();
                tasks.push(task::spawn(async move {
                    let res;
                    match test_site(site).await {
                        Ok(r) => {
                            res = r.site.into();
                        }
                        Err(e) => {
                            res = format!("Request Failed {}", e.to_string()).into();
                        }
                    }

                    let _ = tx.send(res);
                }));
            }
            let tasks_len = tasks.len();
            let all_tasks = join_all(tasks);
            let timeout_duration = Duration::from_secs(3);
            let _ = timeout(timeout_duration, all_tasks);
            let mut i = 0;
            while let Some(message) = rx.recv().await {
                let _ = stream.send(message).await;
                i += 1;
                if i >= tasks_len {
                    break;
                }
            }
        }
        Err(e) => {
            let _ = stream.send(e.to_string().into()).await;
        }
    }
    let _ = stream.close().await;
}

pub async fn handle_http() -> Json<Value> {
    let var_map: HashMap<String, String> = env::vars().collect();
    let sites = get_nginx(var_map.get("NGINX").unwrap().to_string()).await;
    match sites {
        Ok(sites) => {
            let mut res = vec![];
            let (tx, mut rx) = mpsc::unbounded_channel();
            let mut tasks: Vec<JoinHandle<()>> = vec![];
            for site in sites {
                let tx = tx.clone();
                tasks.push(task::spawn(async move {
                    let res;
                    match test_site(site.clone()).await {
                        Ok(r) => res = r,
                        Err(_) => {
                            res = SiteResponse {
                                site,
                                status: false,
                            }
                        }
                    }

                    let _ = tx.send(res);
                }));
            }
            let tasks_len = tasks.len();
            let all_tasks = join_all(tasks);
            let timeout_duration = Duration::from_secs(3);
            let _ = timeout(timeout_duration, all_tasks);
            while let Some(message) = rx.recv().await {
                res.push(message);
                if res.len() >= tasks_len {
                    break;
                }
            }
            Json(json!(res))
        }
        Err(e) => Json(json!(SiteResponse {
            site: format!("Could not get sites: {}", e),
            status: false,
        })),
    }
}

async fn get_nginx(url: String) -> Result<Vec<String>, NginxError> {
    let result = reqwest::get(url.to_owned()).await;
    match result {
        Ok(r) => match r.status() {
            reqwest::StatusCode::OK => match r.text().await {
                Ok(v) => match parse_nginx(v).await {
                    Ok(v) => Ok(v),
                    Err(e) => Err(NginxError {
                        message: format!("Could not parse nginx content with: {}", e),
                    }),
                },
                Err(_) => Err(NginxError {
                    message: String::from("Could not get text from request"),
                }),
            },
            _ => Err(NginxError {
                message: String::from("Did not get right status code"),
            }),
        },
        Err(_) => Err(NginxError {
            message: String::from("Could not send request for nginx"),
        }),
    }
}

async fn parse_nginx(text: String) -> Result<Vec<String>, String> {
    let lines: Vec<String> = text
        .split('\n')
        .into_iter()
        .map(|x| x.to_string())
        .collect();

    let exclude: HashSet<String> = match lines.get(0) {
        Some(v) => v
            .split_whitespace()
            .into_iter()
            .filter(|s| s.ends_with("sachiniyer.com"))
            .map(|s| s.to_string())
            .collect(),
        None => return Err("Nothing found".to_string()),
    };
    let mut res: Vec<String> = vec![];

    for (i, l) in lines.iter().enumerate() {
        let words: Vec<String> = l
            .split_whitespace()
            .into_iter()
            .map(|x| x.to_string())
            .collect();
        if words.len() > 1 && i < lines.len() - 1 && words[0] == "listen" && words[1] == "80;" {
            let mut words: Vec<String> = lines[i + 1]
                .clone()
                .split_whitespace()
                .into_iter()
                .map(|x| x.to_string())
                .collect();
            if words.len() > 1 && words[0] == "server_name" {
                words.remove(0);
                words.last_mut().unwrap().pop();
                res.append(&mut words.clone())
            }
        }
    }
    res.retain(|x| !exclude.contains(x));
    match res.len() {
        0 => Err("Nothing found".to_string()),
        _ => Ok(res.iter().map(|x| "https://".to_string() + x).collect()),
    }
}

async fn test_site(url: String) -> Result<SiteResponse, String> {
    let retry_num = 3;
    let timeout_duration = Duration::from_secs(3);
    let client = Client::builder()
        .timeout(timeout_duration)
        .build()
        .or_else(|e| {
            Err(format!(
                "error with site {}, and error {}",
                url,
                e.to_string()
            ))
        })?;

    for _ in 0..retry_num {
        match call_site(&url, &client).await {
            Ok(_) => {
                return Ok(SiteResponse {
                    site: url.clone(),
                    status: true
                })
            },
            _ => {}
        };
    }
    Err(url)
}

async fn call_site(url: &String, client: &Client) -> Result<String, String> {
    let result = client.get(url.clone()).send().await;
    match result {
        Ok(r) => match r.status() {
            reqwest::StatusCode::OK => Ok(url.clone()),
            _ => Err(url.clone()),
        },
        Err(_) => Err(url.clone()),
    }
}
