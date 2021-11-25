use super::utils;
use super::*;
use std::os::raw::{c_int, c_short, c_uchar, c_uint};

#[cfg(not(feature = "release"))]
use std::f64::consts::PI;

/// Open device with specified ID
pub fn open(id: c_short) {
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
            1 => error = 0,
            _ => error = 5,
        }
        utils::parse_error(error, "TUSB0216AD_Device_Open");
    }
}

/// Close the connection with the device
pub fn close(id: c_short) {
    #[cfg(feature = "release")]
    {
        unsafe {
            TUSB0216AD_Device_Close(id);
        }
    }
}

#[allow(dead_code)]
pub fn single_data(id: c_short, data: *mut c_int) {
    #[cfg(feature = "release")]
    {
        let error: c_short;
        unsafe {
            error = TUSB0216AD_Ad_Single(id, data);
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
        if id != 1 || ch > 2 || prelen < 0 || trig_type > 3 || trig_ch > 1 {
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
        if id != 1 {
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
    let mut status = 1 as u8;
    let mut overflow = [0, 0];
    let mut datalen = [0, 0];
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
        if id != 1 {
            error = 5;
            utils::parse_error(error, "TUSB0216AD_Ad_Data");
        }
        unsafe {
            if ch != 0 && ch != 1 {
                error = 8;
            } else {
                *length = 10000;
                for i in 0..10000 {
                    *data.offset(i) = (2f32.powf(15.0)
                        * ((2e-4 * 2.0 * PI as f32 * i as f32).sin() + 1.0))
                        as i32;
                }
                error = 0;
            }
        }
    }
    utils::parse_error(error, "TUSB0216AD_Ad_Data");
}

pub fn set_clock(id: c_short, clock_time: c_int, sel: c_uchar) {
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
        if id != 1 {
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
        if id != 1 || type1 > 6 || type2 > 6 {
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
            if id != 1 {
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
        if id != 1 {
            error = 5;
        } else {
            error = 0;
        }
    }
    utils::parse_error(error, "TUSB0216AD_Trigger");
}