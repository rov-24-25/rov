use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;

use linux_embedded_hal::{Delay,I2cdev};
use pwm_pca9685::{Channel, Pca9685, Address};

use std::error::Error;
use std::thread;
use std::time::Duration;


#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {
        let listener = TcpListener::bind("0.0.0.0:12345").await?;
        println!("connected");

        let i2c_pca = I2cdev::new("/dev/i2c-1")
        .map_err(|e| format!("Failed to open I2C device: {:?}", e))?;
        //let mut delay = Delay;

    // Attempt to initialize PCA9685
        let pca_address = Address::default(); // default I2C address for PCA9685
        let mut pca= Pca9685::new(i2c_pca, pca_address).unwrap();
        //pca.init(&mut delay);

        pca.set_prescale(127).unwrap(); // Set frequency of PWM outputs to 50Hz

        pca.enable().unwrap();
        pca.set_channel_on_off(Channel::C2,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C1,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C0,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C5,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C4,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C3,0, 307).unwrap();
        thread::sleep(Duration::from_millis(5000));

        pca.set_channel_on_off(Channel::C8,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C9,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C10,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C11,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C12,0, 307).unwrap();
        println!("everything is initialized");


        while let Ok((mut socket, addr))=listener.accept().await{
            println!("connection from {}",addr);

            let mut buf=[0u8;44];
            while let Ok(_)=socket.read_exact(&mut buf).await{
                let data_points=[
                f32::from_le_bytes(buf[0..4].try_into().unwrap()),//x
                f32::from_le_bytes(buf[4..8].try_into().unwrap()),//y
                f32::from_le_bytes(buf[8..12].try_into().unwrap()),//z
                f32::from_le_bytes(buf[12..16].try_into().unwrap()),//rot
                f32::from_le_bytes(buf[16..20].try_into().unwrap()),
                f32::from_le_bytes(buf[20..24].try_into().unwrap()),
                f32::from_le_bytes(buf[24..28].try_into().unwrap()),
                f32::from_le_bytes(buf[28..32].try_into().unwrap()),
                f32::from_le_bytes(buf[32..36].try_into().unwrap()),
                f32::from_le_bytes(buf[36..40].try_into().unwrap()),
                f32::from_le_bytes(buf[40..44].try_into().unwrap())
                ];
                println!("received data: {:?}", data_points);


        let b0=data_points[6];
        let b1=data_points[7];
        let b2=data_points[8];
        let b3=data_points[9];
        let b4=data_points[10];

        let h2=307.0-data_points[1]*60.0;//307.0+(1.0+cons)*0.5*norm*20.0;//avant gauche
        let h1=307.0-data_points[1]*60.0;//307.0+(1.0-cons)*0.5*norm*20.0;//avant droit
                //let h1=307.0-data_points[1]*82.0;
                //let h2=307.0+data_points[0]*82.0;
        let h3=307.0+data_points[0]*60.0;//arrière
        let h4=307.0-data_points[2]*25.0+data_points[4]*20.0+data_points[5]*20.0;//vertical av g
        let h5=307.0-data_points[2]*25.0-data_points[4]*20.0+data_points[5]*20.0;// vertical av d
        let h6=307.0-data_points[2]*50.0-data_points[5]*30.0; // vertical ar


        pca.set_channel_on_off(Channel::C2, 0,h1.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C3, 0,h2.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C1, 0,h3.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C0,0, h5.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C4,0, h4.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C5,0, h6.round() as u16).unwrap();


        pca.set_channel_on_off(Channel::C8, 0,b0.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C9, 0,b1.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C10, 0,b2.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C11, 0,b3.round() as u16).unwrap();
        pca.set_channel_on_off(Channel::C12, 0,b4.round() as u16).unwrap();

        thread::sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}