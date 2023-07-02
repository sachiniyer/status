use axum::extract::ws::WebSocket;
use futures::future::join_all;
use reqwest;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use tokio::sync::mpsc;
use tokio::task;
use tokio::task::JoinHandle;

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
                    match test_site(site).await {
                        Ok(r) => {
                            let _ = tx.send(r.site.into());
                        }
                        Err(e) => {
                            let _ = tx.send(e.to_string().into());
                        }
                    }
                }));
            }
            while let Some(message) = rx.recv().await {
                let _ = stream.send(message).await;
            }

            let _ = join_all(tasks).await;
        }
        Err(e) => {
            let _ = stream.send(e.to_string().into()).await;
        }
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
    match res.len() {
        0 => Err("Nothing found".to_string()),
        _ => Ok(res.iter().map(|x| "https://".to_string() + x).collect()),
    }
}

async fn test_site(url: String) -> Result<SiteResponse, String> {
    let result = reqwest::get(url.clone()).await;
    match result {
        Ok(r) => match r.status() {
            reqwest::StatusCode::OK => Ok(SiteResponse {
                site: url.clone(),
                status: true,
            }),
            _ => Ok(SiteResponse {
                site: url.clone(),
                status: false,
            }),
        },
        Err(_) => Err("Request Failed".to_string()),
    }
}
