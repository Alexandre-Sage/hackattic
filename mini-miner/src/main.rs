use anyhow::Context;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use sha256::digest;

const PROBLEM_URL: &str = "https://hackattic.com/challenges/mini_miner/problem?access_token=";
const SOLVE_URL: &str = "https://hackattic.com/challenges/mini_miner/solve?access_token=";

fn parse_nonce<'de, D>(d: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    #[serde(deserialize_with = "parse_nonce")]
    nonce: usize,
    data: Vec<(String, i32)>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Challenge {
    difficulty: usize,
    block: Block,
}

impl Block {
    fn increment_nonce(&mut self) {
        self.nonce += 1;
    }
}

impl ToString for Block {
    fn to_string(&self) -> String {
        json!(self).to_string()
    }
}

async fn challenge() -> anyhow::Result<Challenge> {
    let token = dotenvy::var("AUTH_TOKEN").unwrap();
    let url = format!("{}{}", PROBLEM_URL, token);

    let res = reqwest::get(url).await?;
    res.json().await.with_context(|| "Could not parse json")
}

async fn submit(response: usize) {
    let token = dotenvy::var("AUTH_TOKEN").unwrap();
    let url = format!("{}{}", SOLVE_URL, token);

    let res = reqwest::Client::new()
        .post(url)
        .json(&json!({"nonce":response}))
        .send()
        .await
        .unwrap();
    dbg!(res.json::<serde_json::Value>().await.unwrap());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let challenge = challenge().await?;
    let bytes_needed = challenge.difficulty / 4;
    let mut block = challenge.block;
    let start = (0..bytes_needed).map(|_| 0.to_string()).collect::<String>();
    loop {
        let result = digest(block.to_string());
        if result.starts_with(&start) {
            submit(block.nonce).await;
            break;
        }
        block.increment_nonce();
    }
    Ok(())
}
