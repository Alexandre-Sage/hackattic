use anyhow::Context;
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};

pub struct HackatticApi {
    client: reqwest::Client,
    challenge: String,
}

impl HackatticApi {
    pub fn new(challenge: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            challenge: challenge.to_owned(),
        }
    }

    fn format_url(&self, part: &str) -> String {
        let token = dotenvy::var("AUTH_TOKEN").unwrap();
        let challenge = &self.challenge;
        format!("https://hackattic.com/challenges/{challenge}/{part}?access_token={token}")
    }
    pub async fn problem<T>(&self) -> anyhow::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.format_url("problem");
        self.client
            .get(url)
            .send()
            .and_then(|res| res.json())
            .await
            .with_context(|| "Unable to fetch challenge problem")
    }

    pub async fn solve<T>(&self, response: T) -> anyhow::Result<serde_json::Value>
    where
        T: Serialize,
    {
        let url = self.format_url("solve");
        self.client
            .post(url)
            .json(&response)
            .send()
            .and_then(|res| res.json())
            .await
            .with_context(|| "Unable to send challenge solution")
    }
}
