use reqwest;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tokio;

pub fn post_data(
    flag: Arc<Mutex<i8>>,
    position: Arc<Mutex<Vec<f32>>>,
    intensity: Arc<Mutex<Vec<f32>>>,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    println!("Start posting!");
    loop {
        if *flag.lock().unwrap() != -1 {
            break;
        }
        thread::sleep(time::Duration::from_millis(1));
    }
    println!("Send json!");

    let client = reqwest::Client::new();
    let url = env::var("DATA_POST_URL").expect("DATA_POST_URL is not set");
    loop {
        if *flag.lock().unwrap() == 1 {
            break;
        }

        thread::sleep(time::Duration::from_millis(100));

        let mut map = HashMap::new();
        let x = position.lock().unwrap();
        let y = intensity.lock().unwrap();

        let mut xx: Vec<f32> = Vec::new();
        let mut yy: Vec<f32> = Vec::new();

        for i in 0..x.len() {
            xx.push(x[i]);
            yy.push(y[i]);
        }

        rt.block_on(async {
            map.insert("x", &xx);
            map.insert("y", &yy);
            let _response = client
                .post(&url)
                .json(&map)
                .send()
                .await
                .expect("Failed to post json");
        });
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    use dotenv::dotenv;
    use std::env;

    #[test]
    fn test_post() {
        dotenv().ok();

        // let client = reqwest::Client::new();
        let url = env::var("DATA_POST_URL").expect("DATA_POST_URL is not set");

        assert_eq!(&url, "http://localhost:8000/core/rapid-scan-data/");
    }
}
