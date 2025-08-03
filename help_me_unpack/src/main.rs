use std::io::{Cursor, Seek, SeekFrom};

use base64::prelude::*;
use byteorder::{LittleEndian, NetworkEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

const CHALLENGE_NAME: &str = "help_me_unpack";

#[derive(Debug, Deserialize)]
struct Challenge {
    bytes: String,
}

#[derive(Debug, Serialize)]
struct Answer {
    int: i32,
    uint: u32,
    short: i16,
    float: f32,
    double: f64,
    big_endian_double: f64,
}

impl Answer {
    async fn new(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);
        let int = cursor
            .read_i32::<LittleEndian>()
            .expect("Cannot read 'int' as i32");
        let uint = cursor
            .read_u32::<LittleEndian>()
            .expect("Cannot read 'uint' as u32");
        let short = cursor
            .read_i16::<LittleEndian>()
            .expect("Cannot read 'short' as i16");
        cursor
            .seek(SeekFrom::Current(2))
            .expect("Failed to seek past padding bytes");
        // After int (4) + uint (4) + short (2) = 10 bytes.
        // To align the next 4 byte float to a 4byte bound, 2 bytes of padding are needed.
        // as mentioned in the problem In case you're wondering, we're using 4 byte ints, so everything is in the context of a 32-bit platform.
        let float = cursor
            .read_f32::<LittleEndian>()
            .expect("Cannot read 'float' as f32");
        let double = cursor
            .read_f64::<LittleEndian>()
            .expect("Cannot read 'double' as f64");
        let big_endian_double = cursor
            .read_f64::<NetworkEndian>()
            .expect("Cannot read 'double' as f64");
        Self {
            int,
            uint,
            short,
            float,
            double,
            big_endian_double,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    let api = helpers::HackatticApi::new(CHALLENGE_NAME);

    let challenge = api.problem::<Challenge>().await?;
    println!("{:#?}", challenge);
    let decoded = BASE64_STANDARD.decode(challenge.bytes)?;
    let answer = Answer::new(&decoded).await;
    let answer = api.solve(answer).await?;
    println!("{:#?}", answer);
    Ok(())
}
