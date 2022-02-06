mod helpers;
mod operations;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use dotenv::dotenv;
use helpers::{helper, post};
use operations::interface;
use std::cmp::Ordering;
use std::os::raw::{c_int, c_short, c_uint};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RawDataset {
    x: i32,
    y: i32,
    len: u32,
}

impl Ord for RawDataset {
    fn cmp(&self, other: &Self) -> Ordering {
        other.x.cmp(&self.x)
    }
}

impl PartialOrd for RawDataset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[no_mangle]
pub extern "C" fn run(id: c_short, clk_time: c_int, seconds: u64) {
    // sequence が走っているかを示すフラグ
    // -1: not-started, 0: running, 1: finished
    let flag = Arc::new(Mutex::new(0));
    dotenv().ok();

    // +/- 3.75μm駆動させたときに精度375nmで取るために必要な領域
    const DATA_SIZE: usize = 20000;

    let flg1 = Arc::clone(&flag);
    let time_keeper = thread::spawn(move || {
        helper::continuous_read(id, clk_time, seconds, flg1);
    });

    let flg2 = Arc::clone(&flag);
    let data = Arc::new(Mutex::new(Vec::<RawDataset>::with_capacity(DATA_SIZE)));

    let data_cln = Arc::clone(&data);
    let job_runner = thread::spawn(move || {
        helper::get_data(id, flg2, data_cln);
    });

    let data_cln2 = Arc::clone(&data);
    let flg3 = Arc::clone(&flag);
    let post_data = thread::spawn(move || {
        let _ = post::post_data(id, flg3, data_cln2);
    });

    time_keeper.join().expect("Paniced at time_keeper");
    job_runner.join().expect("Paniced at job_runner");
    post_data.join().expect("Paniced at post_data thread");
}

#[no_mangle]
pub extern "C" fn test_run() {
    const MAX_LENGTH: usize = 100000;
    let mut length: c_uint;

    let mut store: Vec<c_int> = vec![];
    let mut store2: Vec<c_int> = vec![];

    interface::open(0);
    interface::set_clock(0, 500, 0);
    interface::input_set(0, 0, 0);
    interface::start(0, 0, 0, 0, 0);
    interface::trigger(0);

    for _ in 0..20 {
        let mut data1 = [0 as c_int; MAX_LENGTH];
        let mut data2 = [0 as c_int; MAX_LENGTH];
        let device_status = interface::status(true);
        length = device_status.ch1_datalen;

        if device_status.status == 3 {
            interface::takeout_data(0, 0, data1.as_mut_ptr(), &mut length as *mut u32);
            interface::takeout_data(0, 1, data2.as_mut_ptr(), &mut length as *mut u32);
        } else {
            continue;
        }

        for i in 0..length as usize {
            store.push(data1[i]);
            store2.push(data2[i]);
        }
        println!("length: {}", length);
    }

    interface::stop(0);
    interface::close(0);

    let mut a: Vec<f32> = vec![];
    let mut b: Vec<f32> = vec![];
    for i in 0..store.len() as usize {
        let result = post::convert_to_voltage(0, 0, store[i] as f32, store2[i] as f32);
        a.push(result.0);
        b.push(result.1);
    }
    helper::write_to_csv("C:/Users/yudai/Desktop/a.csv", &a, &b);
}
