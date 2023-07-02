use reqwest;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use tide::Request;

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

pub async fn get_sites(mut _req: Request<()>) -> tide::Result {
    let var_map: HashMap<String, String> = env::vars().collect();
    let sites: Vec<String> = get_nginx(var_map.get("NGINX").unwrap().to_string()).unwrap();
    let sites: String = sites.into_iter().collect();
    Ok(format!("{}", sites).into())
}

#[tokio::main]
async fn get_nginx(url: String) -> Result<Vec<String>, NginxError> {
    let result = reqwest::get(url.to_owned()).await;
    match result {
        Ok(r) => match r.status() {
            reqwest::StatusCode::OK => match r.text().await {
                Ok(v) => match parse_nginx(v).await {
                    Ok(v) => Ok(v),
                    Err(_) => Err(NginxError {
                        message: String::from("Could not parse nginx content"),
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
    for l in lines {
        let mut words: Vec<String> = l
            .split_whitespace()
            .into_iter()
            .map(|x| x.to_string())
            .collect();
        if words.len() > 2 && words[0] == "server_name" && words[1] == "sachiniyer.com" {
            words.remove(0);
            words.last_mut().unwrap().pop();
            return Ok(words);
        }
    }
    Err("Could not parse".to_string())
}
