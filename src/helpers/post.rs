use reqwest;

use crate::operations::interface;
use crate::RawDataset;
use std::env;
use std::os::raw::{c_short, c_uchar};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tokio;

#[derive(Serialize)]
struct JsonData {
    x: Vec<f32>,
    y: Vec<f32>,
    finished: bool,
}

/// CH1, CH2 にセットされているレンジの番号を取得する
///
/// # Argument
///
/// * id - 装置のユニット番号選択スイッチの数字
fn get_ranges(id: c_short) -> (u8, u8) {
    let mut ch1_range: u8 = 0;
    let mut ch2_range: u8 = 0;

    let ch1_range_ptr = &mut ch1_range as *mut c_uchar;
    let ch2_range_ptr = &mut ch2_range as *mut c_uchar;
    interface::input_check(id, ch1_range_ptr, ch2_range_ptr);

    (ch1_range, ch2_range)
}

/// レンジの番号からレンジ幅を計算
///
/// # Argument
///
/// * range - レンジの番号。例えば0なら +/- 10 V
fn calc_range_width(range: u8) -> f32 {
    match range {
        0 => 20.0,
        1 | 4 => 10.0,
        2 | 5 => 5.0,
        3 | 6 => 2.5,
        _ => 0.0,
    }
}

/// 装置から得られた１点のストレートバイナリを電圧に変換
///
/// # Arguments
///
/// * ch1_range - CH1のレンジ番号
/// * ch2_range - CH2のレンジ番号
/// * ch1_data - CH1から得られたストレートバイナリ形式のデータ1点
/// * ch2_data - CH2から得られたストレートバイナリ形式のデータ1点
///
/// # Returns
///
/// CH1, CH2の出力電圧値
pub fn convert_to_voltage(
    ch1_range: u8,
    ch2_range: u8,
    ch1_data: f32,
    ch2_data: f32,
) -> (f32, f32) {
    let ch1_width: f32 = calc_range_width(ch1_range);
    let ch2_width: f32 = calc_range_width(ch2_range);

    let ch1_result: f32;
    let ch2_result: f32;
    if ch1_range > 3 {
        ch1_result = ch1_data * ch1_width / 2f32.powf(16.0);
    } else {
        ch1_result = ch1_data * ch1_width / 2f32.powf(16.0) - ch1_width / 2.0;
    }

    if ch2_range > 3 {
        ch2_result = ch2_data * ch2_width / 2f32.powf(16.0);
    } else {
        ch2_result = ch2_data * ch2_width / 2f32.powf(16.0) - ch2_width / 2.0;
    }

    return (ch1_result, ch2_result);
}

pub fn post_data(id: c_short, flag: Arc<Mutex<i8>>, dataset: Arc<Mutex<Vec<RawDataset>>>) {
    let range: (c_uchar, c_uchar) = get_ranges(id);
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
        thread::sleep(time::Duration::from_millis(300));
        let dataset = dataset.lock().unwrap();
        let mut xx: Vec<f32> = Vec::new();
        let mut yy: Vec<f32> = Vec::new();

        for i in 0..dataset.len() {
            let voltage =
                convert_to_voltage(range.0, range.1, dataset[i].x as f32, dataset[i].y as f32);

            xx.push(voltage.0);
            yy.push(voltage.1);
        }

        if *flag.lock().unwrap() == 1 {
            rt.block_on(async {
                let data = JsonData {
                    x: xx,
                    y: yy,
                    finished: true,
                };
                let _response = client
                    .post(&url)
                    .json(&data)
                    .send()
                    .await
                    .expect("Failed to post json");
            });
            break;
        }

        rt.block_on(async {
            let data = JsonData {
                x: xx,
                y: yy,
                finished: false,
            };
            let _response = client
                .post(&url)
                .json(&data)
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
