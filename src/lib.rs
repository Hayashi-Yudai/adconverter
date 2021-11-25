mod helpers;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use dotenv::dotenv;
use helpers::{helper, operation, post};
use std::os::raw::{c_int, c_short, c_uint};
use std::sync::{Arc, Mutex};
use std::thread;

#[no_mangle]
pub extern "C" fn run(id: c_short, seconds: u64) {
    // sequence が走っているかを示すフラグ
    // -1: not-started, 0: running, 1: finished
    let flag = Arc::new(Mutex::new(0));
    dotenv().ok();

    // +/- 3.75μm駆動させたときに精度375nmで取るために必要な領域
    const DATA_SIZE: usize = 20000;

    let flg1 = Arc::clone(&flag);
    let time_keeper = thread::spawn(move || {
        helper::continuous_read(id, seconds, flg1);
    });

    let flg2 = Arc::clone(&flag);
    let x = Arc::new(Mutex::new(Vec::<f32>::with_capacity(DATA_SIZE)));
    let y = Arc::new(Mutex::new(Vec::<f32>::with_capacity(DATA_SIZE)));
    let counter = Arc::new(Mutex::new(Vec::<u32>::with_capacity(DATA_SIZE)));

    let x_cln = Arc::clone(&x);
    let y_cln = Arc::clone(&y);
    let counter_cln = Arc::clone(&counter);
    let job_runner = thread::spawn(move || {
        helper::get_data(id, flg2, x_cln, y_cln, counter_cln);
    });

    let x_cln2 = Arc::clone(&x);
    let y_cln2 = Arc::clone(&y);
    let flg3 = Arc::clone(&flag);
    let post_data = thread::spawn(move || {
        let _ = post::post_data(flg3, x_cln2, y_cln2);
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

    operation::open(0);
    operation::set_clock(0, 500, 0);
    operation::input_set(0, 0, 0);
    operation::start(0, 0, 0, 0, 0);
    operation::trigger(0);

    for _ in 0..100 {
        let mut data1 = [0 as c_int; MAX_LENGTH];
        let device_status = operation::status(true);
        length = device_status.ch1_datalen;

        if device_status.status == 3 {
            operation::takeout_data(0, 0, data1.as_mut_ptr(), &mut length as *mut u32);
        }

        for i in 0..length as usize {
            store.push(data1[i]);
        }
        println!("length: {}", length);
    }

    operation::stop(0);
    operation::close(0);

    let mut a: Vec<f32> = vec![];
    for i in 0..store.len() as usize {
        let result = helper::convert_to_voltage(0, 0, store[i] as f32, store[i] as f32);
        a.push(result.0);
    }
    helper::write_to_csv("C:/Users/yudai/Desktop/a.csv", &a, &a);
}
