use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Channel, Pca9685};
use std::time::Duration;
use std::thread;

fn main() {
    let dev = match I2cdev::new("/dev/i2c-1") {
        Ok(dev) => dev,
        Err(_error) => {
            panic!("Could not create I2C device. Is I2C enabled in raspi-config?")
        },
    };

    let address = Address::default();
    let mut pwm = Pca9685::new(dev, address).unwrap();

    // This corresponds to a frequency of 60 Hz.
    pwm.set_prescale(127).unwrap();

    // It is necessary to enable the device.
    pwm.enable().unwrap();

    // Turn on channel 0 at 0.
    pwm.set_channel_on_off(Channel::C15,0, 307).unwrap();
    thread::sleep(Duration::from_millis(5000));
    let b4: f32 = 270.0;
    pwm.set_channel_on_off(Channel::C15, 0, b4.round() as u16).unwrap();

    thread::sleep(Duration::from_millis(100));

    println!("everything is initialized");

    // Turn off channel 0 at 2047, which is 50% in
    // the range `[0..4095]`.
    // pwm.set_channel_off(Channel::C15, 2047).unwrap();

    let _dev = pwm.destroy(); // Get the I2C device back
}