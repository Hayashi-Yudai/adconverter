use crate::operations::interface;
use crate::RawDataset;
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::os::raw::{c_int, c_short, c_uint};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread, time};
use synthrs::filter::{convolve, cutoff_from_frequency, lowpass_filter};

/// low pass filter
/// cutoff: 30 Hz
/// sampling rate: 100 kHz
/// band: 0.01
fn lowpass(sample: &Vec<c_int>) -> Vec<c_int> {
    let filter = lowpass_filter(cutoff_from_frequency(3000.0, 1000_000), 0.1);
    let sample: Vec<f64> = sample.into_iter().map(|x| *x as f64).collect();

    convolve(&filter, sample.as_slice())
        .into_iter()
        .map(|x| x.round() as c_int)
        .collect()
}

// ステージのポジション(tmp1)ごとにデータをまとめる
// +/-10Vとして位置測定をしていると仮定している
fn update_data(
    x: &Vec<c_int>,
    y: &Vec<c_int>,
    dataset: &mut MutexGuard<Vec<RawDataset>>,
    length: c_uint,
) {
    for i in 0..length as usize {
        let xx = x[i];
        let yy = y[i];

        match dataset.binary_search_by(|entry| entry.x.cmp(&xx)) {
            Ok(idx) => {
                let length = dataset[idx].len as i32;
                dataset[idx].y =
                    ((dataset[idx].y * length + yy) as f32 / (length + 1) as f32).round() as i32;
                dataset[idx].len += 1;
            }
            Err(_) => {
                dataset.push(RawDataset {
                    x: xx,
                    y: yy,
                    len: 1,
                });
            }
        }
    }

    dataset.sort_by(|a, b| a.x.cmp(&b.x));
}

/// 装置の連続データ取り込みの制御。指定の時間だけデータ取り込みを行う
/// このメソッドではデータの取り込み開始、終了を制御するだけで装置のバッファに
/// たまったデータの取り出しは行わない
///
/// # Arguments
///
/// * id - 装置のユニット番号選択スイッチの数字
/// * seconds - データ取り込みを行う秒数
/// * flag - データ取り込み中であるかを判別するフラグ
pub fn continuous_read(id: c_short, seconds: u64, flag: Arc<Mutex<i8>>) {
    let sleeping_time = time::Duration::from_secs(seconds);

    // CH1, 2ともに+/-10Vの入力を受け付ける
    // 入力が+/-10VなのはSR830の仕様
    interface::input_set(id, 0, 0);
    interface::set_clock(id, 500, 0);
    interface::start(id, 2, 0, 0, 0);
    interface::trigger(id);

    *flag.lock().unwrap() = 0; // 計測開始のフラグを立てる
    thread::sleep(sleeping_time);

    interface::stop(id);

    *flag.lock().unwrap() = 1; // 計測終了のフラグを立てる
    println!("Timer stopped");
}

/// データの取り込みが行われているフラグが立っている間
/// CH1, CH2 からのデータを取得する
///
/// # Arguments
///
/// * id - 装置のユニット番号選択スイッチの数字
/// * flag - データ取り込み中であるかを判別するフラグ
/// * position - CH1のデータを収納するベクトル
/// * intensity - CH2のデータを収納するベクトル
pub fn get_data(id: c_short, flag: Arc<Mutex<i8>>, dataset: Arc<Mutex<Vec<RawDataset>>>) {
    // let ranges: (u8, u8) = get_ranges(id);
    // const MAX_LENGTH: usize = 262142;
    const MAX_LENGTH: usize = 100000;
    let mut length: c_uint;

    println!("Data acquisition started");
    loop {
        if *flag.lock().unwrap() != -1 {
            break;
        }
        thread::sleep(time::Duration::from_millis(1));
    }

    // let mut data1: Vec<c_int> = Vec::with_capacity(MAX_LENGTH);
    // let mut data2: Vec<c_int> = Vec::with_capacity(MAX_LENGTH);
    let mut data1: Vec<c_int> = vec![0; MAX_LENGTH];
    let mut data2: Vec<c_int> = vec![0; MAX_LENGTH];

    loop {
        if *flag.lock().unwrap() == 1 {
            break;
        }
        let device_status = interface::status(false);

        if device_status.status == 3 {
            length = min(device_status.ch1_datalen, device_status.ch2_datalen);
            let l_ptr = &mut length as *mut u32;
            interface::takeout_data(id, 0, data1.as_mut_ptr(), l_ptr);
            interface::takeout_data(id, 1, data2.as_mut_ptr(), l_ptr);
        } else {
            continue;
        }

        let mut position_denoised: Vec<c_int> = lowpass(&data1);
        position_denoised.shrink_to_fit();
        data2.shrink_to_fit();

        let mut dataset = dataset.lock().unwrap();
        update_data(&position_denoised, &data2, &mut dataset, length);
    }
    println!("Data acquisition stopped");
}

pub fn write_to_csv(file_name: &str, x: &Vec<f32>, y: &Vec<f32>) {
    let mut file = File::create(file_name).unwrap();

    for i in 0..x.len() {
        write!(file, "{},{}\n", x[i], y[i]).unwrap();
    }

    file.flush().unwrap();
}

#[cfg(test)]
mod test {
    use super::*;
    use nearly_eq::*;
    use rand::Rng;
    use std::f64::consts::PI;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Instant;

    #[test]
    fn test_lowpass() {
        const DATA_NUM: usize = 10000; // 0.1 sec
        let mut x = vec![0];
        let mut y = vec![0; DATA_NUM];
        let mut rng = rand::thread_rng();

        for i in 1..DATA_NUM {
            x.push(i as i32);
        }

        // 1 data point = 10 micro-sec
        // 1 cycle = 0.05 sec = 5000 data points
        // y = sin(2e-4 * x)
        for i in 0..DATA_NUM {
            y[i] = (1000.0 * (x[i] as f32 * 2e-4 * 2.0 * PI as f32).sin()) as i32;
        }

        let mut y_noise = y.clone();
        for i in 0..DATA_NUM {
            let noise: f32 = rng.gen();
            y_noise[i] = y[i] + (2000.0 * 5e-3 * (noise - 0.5)) as i32;
        }

        let denoised = lowpass(&y_noise);
        println!("{}", denoised.len());

        for i in 5..(DATA_NUM - 5) {
            assert_nearly_eq!(y[i], denoised[i], 4);
        }
    }

    #[test]
    fn test_update_data() {
        let x = vec![0, 1, 2, 3, 4];
        let y = vec![0, 1, 4, 9, 16];
        let mut dataset = Mutex::new(vec![
            RawDataset { x: 0, y: 0, len: 1 },
            RawDataset { x: 2, y: 5, len: 2 },
            RawDataset {
                x: 4,
                y: 16,
                len: 1,
            },
        ]);

        update_data(&x, &y, &mut dataset.lock().unwrap(), 5);

        let dataset = dataset.lock().unwrap();
        let correct_ys = [0, 1, 5, 9, 16];
        let correct_lens = [2, 1, 3, 1, 2];
        for i in 0..5 {
            assert_eq!(dataset[i].x, i as i32);
            assert_eq!(dataset[i].y, correct_ys[i]);
            assert_eq!(dataset[i].len, correct_lens[i]);
        }
    }

    #[test]
    fn test_continuous_read() {
        let seconds = 1;
        let start = Instant::now();
        continuous_read(1, seconds, Arc::new(Mutex::new(0)));
        let end = start.elapsed();

        assert_nearly_eq!(end.as_millis() as f32, (seconds * 1000) as f32, 50.0);
    }
}
