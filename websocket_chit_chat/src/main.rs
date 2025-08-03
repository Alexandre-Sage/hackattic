use futures::{SinkExt, StreamExt};
use helpers::HackatticApi;
use rustls::crypto::CryptoProvider;
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpSocket, TcpStream},
    time::Instant,
};
use tokio_tungstenite::{
    WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

const CHALLENGE_NAME: &str = "websocket_chit_chat";
const FIRST_MESSAGE:&[u8] = b"hello! start counting the time between the pings (start from the moment you opened this connection)";
const GOOD: &[u8] = b"good!";
const INTERVALS: &[i32] = &[700, 1500, 2000, 2500, 3000];

#[derive(Deserialize, Debug)]
struct ChallengeToken {
    token: String,
}

fn extract_quoted_string(s: &str) -> std::string::String {
    let start_index = s.find('"').unwrap();

    let end_index = s.rfind('"').unwrap();

    s[start_index + 1..end_index].to_string()
}

#[derive(Serialize, Debug)]
struct Answer {
    secret: String,
}

impl Answer {
    fn new(secret: &str) -> Self {
        Self {
            secret: extract_quoted_string(secret),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api = HackatticApi::new(CHALLENGE_NAME);
    let token = api.problem::<ChallengeToken>().await?.token;

    let chat_url = format!("wss://hackattic.com/_/ws/{token}").into_client_request()?;
    let (stream, _) = connect_async(chat_url).await?;
    let (mut sender, mut reader) = stream.split();

    let mut last_message_time = Instant::now();
    loop {
        if let Some(Ok(message)) = reader.next().await {
            println!("{}", message.clone().into_text().unwrap());
            let current_time = Instant::now();
            let data = message.into_data();
            if &data != FIRST_MESSAGE && data != GOOD {
                let elapsed = current_time.duration_since(last_message_time).as_millis();
                println!("Elapsed: {}", elapsed);

                let closest = INTERVALS
                    .iter()
                    .min_by_key(|i| (**i as i64 - elapsed as i64).abs())
                    .unwrap();

                sender.send(closest.to_string().into()).await?;
            }
            if data != GOOD {
                last_message_time = current_time;
            }
            if data.starts_with(b"congratulations") {
                let response = std::str::from_utf8(&data)?;
                let answer = Answer::new(response);
                let response = api.solve(answer).await?;
                println!("{}", response);
                break;
            }
        } else {
            break;
        }
    }

    Ok(())
}
