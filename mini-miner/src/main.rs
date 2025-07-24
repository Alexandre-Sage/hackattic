use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use sha256::digest;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap();
    let api = helpers::HackatticApi::new("mini_miner");

    let challenge = api.problem::<Challenge>().await?;

    let bytes_needed = challenge.difficulty / 4;
    let mut block = challenge.block;
    let start = (0..bytes_needed).map(|_| 0.to_string()).collect::<String>();

    loop {
        let result = digest(block.to_string());
        if result.starts_with(&start) {
            let res = api.solve(&json!({"nonce":block.nonce})).await?;
            dbg!(res);
            break;
        }
        block.increment_nonce();
    }
    Ok(())
}
