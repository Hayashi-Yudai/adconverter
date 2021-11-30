use super::utils;
use super::*;
use std::os::raw::{c_int, c_short, c_uchar, c_uint};

#[cfg(not(feature = "release"))]
use rand::Rng;
#[cfg(not(feature = "release"))]
use std::f64::consts::PI;

/// Open device with specified ID
#[no_mangle]
pub extern "C" fn open(id: c_short) {
    #[cfg(feature = "release")]
    {
        let error: c_short;
        unsafe {
            error = TUSB0216AD_Device_Open(id);
        }
        utils::parse_error(error, "TUSB0216AD_Device_Open");
    }

    #[cfg(not(feature = "release"))]
    {
        let error: c_short;
        match id {
            0 => error = 0,
            _ => error = 5,
        }
        utils::parse_error(error, "TUSB0216AD_Device_Open");
    }
}

/// Close the connection with the device
#[no_mangle]
pub extern "C" fn close(_id: c_short) {
    #[cfg(feature = "release")]
    {
        unsafe {
            TUSB0216AD_Device_Close(_id);
        }
    }
}

#[allow(dead_code)]
pub fn single_data(_id: c_short, _data: *mut c_int) {
    #[cfg(feature = "release")]
    {
        let error: c_short;
        unsafe {
            error = TUSB0216AD_Ad_Single(_id, _data);
        }

        utils::parse_error(error, "TUSB0216AD_Ad_Single");
    }
}

pub fn start(id: c_short, ch: c_uchar, prelen: c_int, trig_type: c_uchar, trig_ch: c_uchar) {
    let error: c_short;

    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Start(id, ch, prelen, trig_type, trig_ch);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 0 || ch > 2 || prelen < 0 || trig_type > 3 || trig_ch > 1 {
            error = 5;
        } else {
            error = 0;
        }
    }
    utils::parse_error(error, "TUSB0216AD_Start");
}

pub fn stop(id: c_short) {
    let error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Stop(id);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 0 {
            error = 5;
        } else {
            error = 0;
        }
    }
    utils::parse_error(error, "TUSB0216AD_Stop");
}

/// Show the device status
///
/// * verbose: bool
///     if true, it prints the status on the screen
pub fn status(verbose: bool) -> DeviceStatus {
    let mut status: u8 = 1;
    let mut overflow: [u8; 2] = [0, 0];
    let mut datalen: [u32; 2] = [0, 0];
    #[cfg(feature = "release")]
    {
        unsafe {
            TUSB0216AD_Ad_Status(
                0,
                &mut status as *mut u8,
                overflow.as_mut_ptr(),
                datalen.as_mut_ptr(),
            );
        }
    }

    #[cfg(not(feature = "release"))]
    {
        status = 3;
        overflow = [0, 0];
        datalen = [10000, 10000];
    }

    match verbose {
        true => {
            println!("============");
            println!("Status: {}", status);
            println!("Overflow: {:?}", overflow);
            println!("DataLen: {:?}", datalen);
            println!("============");
        }
        false => {}
    }

    DeviceStatus::new(status, datalen[0], datalen[1])
}

pub fn takeout_data(id: c_short, ch: c_uchar, data: *mut c_int, length: *mut c_uint) {
    let mut error: c_short;

    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Ad_Data(id, ch, data, length);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        let mut rng = rand::thread_rng();

        if id != 0 {
            error = 5;
            utils::parse_error(error, "TUSB0216AD_Ad_Data");
        }
        unsafe {
            if ch != 0 && ch != 1 {
                error = 8;
            } else {
                *length = 100000;
                let height = 2f32.powf(15.0);

                for i in 0..100000 {
                    let phase = 2e-4 * 2.0 * PI as f32 * i as f32;
                    let random: f32 = rng.gen();
                    let noise = 1e3 * random;

                    *data.offset(i) = (height * (phase.sin() + 1.0) + noise) as i32;
                }
                error = 0;
            }
        }
    }
    utils::parse_error(error, "TUSB0216AD_Ad_Data");
}

#[no_mangle]
pub extern "C" fn set_clock(id: c_short, clock_time: c_int, sel: c_uchar) {
    let mut error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_AdClk_Set(id, clock_time, sel);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        error = 0;
        if id != 0 {
            error = 5;
        }

        if clock_time < 500 {
            error = 8;
        }

        if sel != 0 && sel != 1 {
            error = 8;
        }
    }
    utils::parse_error(error, "TUSB0216AD_AdClk_Set");
}

/// Change input range of each channel.
/// Specify the input ranges with a number.
/// 0: +/-10 V, 1: +/-5V, 2: +/-2.5 V, 3: +/-1.25V
/// 4: 10 V, 5: 5 V, 6: 2.5 V
///
/// * `id` - Device number
/// * `type1` - input range of CH1
/// * `type2` - input range of CH2
pub fn input_set(id: c_short, type1: c_uchar, type2: c_uchar) {
    let error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Input_Set(id, type1, type2);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 0 || type1 > 6 || type2 > 6 {
            error = 5;
        } else {
            error = 0;
        }
    }
    utils::parse_error(error, "TUSB0216AD_Input_Set");
}

pub fn input_check(id: c_short, type1: *mut c_uchar, type2: *mut c_uchar) {
    let error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Input_Check(id, type1, type2);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        unsafe {
            if id != 0 {
                error = 5;
            } else {
                error = 0;
            }
            *type1 = 0;
            *type2 = 0;
        }
    }
    utils::parse_error(error, "TUSB0216AD_Input_Check");
}

pub fn trigger(id: c_short) {
    let error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Trigger(id);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 0 {
            error = 5;
        } else {
            error = 0;
        }
    }
    utils::parse_error(error, "TUSB0216AD_Trigger");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ad_data_mock() {
        const MAX_LENGTH: usize = 100000;
        let mut length = MAX_LENGTH as u32;
        let mut data1 = [0; MAX_LENGTH];
        let mut data2 = [0; MAX_LENGTH];
        let l_ptr = &mut length as *mut u32;
        takeout_data(0, 0, data1.as_mut_ptr(), l_ptr);
        takeout_data(0, 1, data2.as_mut_ptr(), l_ptr);

        assert_eq!(length, 100000);
    }
}
