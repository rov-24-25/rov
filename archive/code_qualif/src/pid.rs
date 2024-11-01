//use tokio::time;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use linux_embedded_hal::I2cdev;
use linux_embedded_hal_mpu::{Delay as DelayMPU,I2cdev as I2cMPU};
use pwm_pca9685::{Channel, Pca9685, Address};
use mpu6050::*;
use std::error::Error;
use std::thread;
use std::time::{Duration,Instant};
use pid::Pid;



#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {
        let listener = TcpListener::bind("0.0.0.0:12345").await?;
        println!("connected");

        let i2c_mpu=I2cMPU::new("/dev/i2c-1").map_err(|e| format!("Failed to open I2C device: {:?}", e)).unwrap();
        let mut mpu=Mpu6050::new(i2c_mpu);
        let mut delay=DelayMPU;
        mpu.init(&mut delay).expect("erreure d'initialisation");
        let mut accel;
        let mut roll;
        let mut pitch;

        let mut mean_roll=0.0;
        let mut mean_pitch=0.0;

        let i2c_pca = I2cdev::new("/dev/i2c-1")
        .map_err(|e| format!("Failed to open I2C device: {:?}", e))?;

        let pca_address = Address::default(); // default I2C address for PCA9685
        let mut pca= Pca9685::new(i2c_pca, pca_address).unwrap();


        let mut pid_pitch=Pid::new(0.0,90.0);
        let mut pid_roll=Pid::new(0.0,90.0);
        pid_pitch.p(1.0,90.0).i(0.1,90.0).d(0.01,90.0);
        pid_roll.p(1.0,90.0).i(0.1,90.0).d(0.01,90.0);


        pca.set_prescale(127).unwrap(); // Set frequency of PWM outputs to 60Hz

        pca.enable().unwrap();
        pca.set_channel_on_off(Channel::C2,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C1,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C0,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C5,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C4,0, 307).unwrap();
        pca.set_channel_on_off(Channel::C3,0, 307).unwrap();
        thread::sleep(Duration::from_millis(5000));
        println!("motors enabled");

        let start = Instant::now();
        while start.elapsed() < Duration::new(10, 0){
            accel=mpu.get_acc_angles().unwrap();
            mean_roll+=accel[0].to_degrees();
            mean_pitch+=accel[1].to_degrees();
            thread::sleep(Duration::from_millis(50));
        }
        mean_pitch/=200.0 as f32;
        mean_roll/=200.0 as f32;

        println!("mpu calibrated");




        while let Ok((mut socket, addr))=listener.accept().await{
            println!("connection from {}",addr);
            let mut buf=[0u8;16];


            while let Ok(_)=socket.read_exact(&mut buf).await{
                let data_points=[
                f32::from_le_bytes(buf[0..4].try_into().unwrap()),
                f32::from_le_bytes(buf[4..8].try_into().unwrap()),
                f32::from_le_bytes(buf[8..12].try_into().unwrap()),
                f32::from_le_bytes(buf[12..16].try_into().unwrap())
                ];
                //println!("received data: {:?}", data_points);


                accel=mpu.get_acc_angles().unwrap();
                roll=accel[0];
                pitch=accel[1];

                roll=roll.to_degrees() as f32;
                roll-=mean_roll;
                if roll.abs()<3.0{
                    roll=0.0;
                }
                pitch=pitch.to_degrees() as f32;
                pitch-=mean_pitch;
                if pitch.abs()<3.0{
                    pitch=0.0;
                }

                let correction_pitch=pid_pitch.next_control_output(pitch).output;
                let correction_roll=pid_roll.next_control_output(roll).output;

                let new_correction_pitch=(correction_pitch+90.0)*2.0/180.0-1.0;
                let new_correction_roll=(correction_roll+90.0)*2.0/180.0-1.0;

                println!("correction pitch: {}, correction roll: {}",new_correction_pitch,new_correction_roll);

                let h2=307.0+data_points[1]*20.0;//307.0+(1.0+cons)*0.5*norm*20.0;//avant gauche
                let h1=307.0+data_points[1]*20.0;//307.0+(1.0-cons)*0.5*norm*20.0;//avant droit
                let h3=307.0+data_points[0]*20.0;//arrière

// !!!! changer les attribution de correction selon l'orientation de la mpu !!!!

                let h4=307.0+(data_points[2]-new_correction_pitch)*20.0;//vertical arrière
                let h5=307.0+(data_points[2]+new_correction_pitch/2.0+new_correction_roll)*20.0;//vertical avant gauche
                let h6=307.0+(data_points[2]+new_correction_pitch/2.0-new_correction_roll)*20.0;//vertical avant droit

                pca.set_channel_on_off(Channel::C2, 0,h1.round() as u16).unwrap();
                pca.set_channel_on_off(Channel::C3, 0,h2.round() as u16).unwrap();
                pca.set_channel_on_off(Channel::C1, 0,h3.round() as u16).unwrap();
                pca.set_channel_on_off(Channel::C0,0, h4.round() as u16).unwrap();
                pca.set_channel_on_off(Channel::C4,0, h5.round() as u16).unwrap();
                pca.set_channel_on_off(Channel::C5,0, h6.round() as u16).unwrap();

                thread::sleep(Duration::from_millis(100));

            }
        }
        Ok(())
    }