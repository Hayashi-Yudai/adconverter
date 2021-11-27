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
                dataset[idx].y = (dataset[idx].y * length + yy) / (length + 1);
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
        println!("Retreiving");

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
    use crate::helpers::post;
    use dotenv::dotenv;
    use nearly_eq::*;
    use rand::Rng;
    use std::f64::consts::PI;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn test_get_range() {
        let range = get_ranges(2);

        assert_eq!(range.0, 0);
        assert_eq!(range.1, 0);
    }

    /// 0: +/-10V, 1: +/-5V, 2: +/-2.5V, 3: +/-1.25V, 4: 10V, 5: 5V, 6: 2.5V
    #[test]
    fn test_calc_width() {
        let width = calc_range_width(0); // +/-10 V
        assert_eq!(width, 20.0);

        let width = calc_range_width(1); // +/-5 V
        assert_eq!(width, 10.0);

        let width = calc_range_width(2); // +/-2.5 V
        assert_eq!(width, 5.0);

        let width = calc_range_width(3); // +/-1.25 V
        assert_eq!(width, 2.5);

        let width = calc_range_width(4); // 10 V
        assert_eq!(width, 10.0);

        let width = calc_range_width(5); // 5 V
        assert_eq!(width, 5.0);

        let width = calc_range_width(6); // 2.5 V
        assert_eq!(width, 2.5);
    }

    #[test]
    fn test_converting_voltage() {
        let ch1_range = 0;
        let ch2_range = 0;
        let ch1_data = 1000.0;
        let ch2_data = 500.0;

        let result = convert_to_voltage(ch1_range, ch2_range, ch1_data, ch2_data);

        assert_eq!(result.0, ch1_data * 20.0 / 2f32.powf(16.0) - 10.0);
        assert_eq!(result.1, ch2_data * 20.0 / 2f32.powf(16.0) - 10.0);
    }

    #[test]
    fn test_lowpass() {
        const DATA_NUM: usize = 10000; // 0.1 sec
        let mut x = vec![0.0];
        let mut y = vec![0.0; DATA_NUM];
        let mut rng = rand::thread_rng();

        for i in 1..DATA_NUM {
            x.push(i as f32);
        }

        // 1 data point = 10 micro-sec
        // 1 cycle = 0.05 sec = 5000 data points
        // y = sin(2e-4 * x)
        for i in 0..DATA_NUM {
            y[i] = (x[i] * 2e-4 * 2.0 * PI as f32).sin();
        }

        let mut y_noise = y.clone();
        for i in 0..DATA_NUM {
            let noise: f32 = rng.gen();
            y_noise[i] = y[i] + 2.0 * 5e-3 * (noise - 0.5);
        }

        let denoised = lowpass(y_noise);
        println!("{}", denoised.len());

        for i in 5..(DATA_NUM - 5) {
            assert_nearly_eq!(y[i], denoised[i], 0.003);
        }
    }

    #[test]
    fn test_update_data() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 1.0, 4.0, 9.0, 16.0];
        let position = Mutex::new(vec![0.0, 2.0, 4.0]);
        let intensity = Mutex::new(vec![0.01, 4.5, 15.8]);
        let counter = Mutex::new(vec![1, 2, 1]);

        update_data(
            &x,
            &y,
            &mut position.lock().unwrap(),
            &mut intensity.lock().unwrap(),
            &mut counter.lock().unwrap(),
        );

        let position = position.lock().unwrap();
        let intensity = intensity.lock().unwrap();
        let counter = counter.lock().unwrap();
        let xx = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let yy = vec![0.005, 1.0, 13.0 / 3.0, 9.0, 15.9];
        let cc = vec![2, 1, 3, 1, 2];
        for i in 0..5 {
            assert_eq!(position[i], xx[i]);
            assert_eq!(intensity[i], yy[i]);
            assert_eq!(counter[i], cc[i]);
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

    #[test]
    #[ignore]
    fn test_get_data() {
        dotenv().ok();

        let position: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let intensity: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let counter: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));
        let flag = Arc::new(Mutex::new(-1));

        let flg1 = Arc::clone(&flag);
        let time_keeper = thread::spawn(move || {
            continuous_read(0, 1, flg1);
        });

        let flg2 = Arc::clone(&flag);
        let posi_cln = Arc::clone(&position);
        let intensity_cln = Arc::clone(&intensity);
        let counter_cln = Arc::clone(&counter);
        let job_runner = thread::spawn(move || {
            get_data(0, flg2, posi_cln, intensity_cln, counter_cln);
        });

        let x_cln2 = Arc::clone(&position);
        let y_cln2 = Arc::clone(&intensity);
        let flg3 = Arc::clone(&flag);
        let post_data = thread::spawn(move || {
            let _ = post::post_data(flg3, x_cln2, y_cln2);
        });

        time_keeper.join().unwrap();
        job_runner.join().unwrap();
        post_data.join().unwrap();

        assert_eq!(*Arc::clone(&flag).lock().unwrap(), 1);
    }
}
