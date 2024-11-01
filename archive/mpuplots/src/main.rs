/*use linux_embedded_hal::{Delay, I2cdev};
use ahrs::{Ahrs,Madgwick};
use mpu6050::{Mpu6050, Mpu6050Error};
use nalgebra::{Quaternion, Vector3,vector};
use plotters::prelude::*;
use std::{fs, path::Path, thread, time::Duration, time::Instant};

fn quat_to_euler(q:Quaternion<f32>)->Vector3<f32>{
    let w =q.w;
    let x =q.i;
    let y =q.j;
    let z=q.k;

    let ysqr=y*y;

    let t0=2.0*(w*x+y*z);
    let t1=1.0-2.0*(x*x+ysqr);
    let roll=t0.atan2(t1);

    let t2=2.0*(w*y-z*x);
    let t2=t2.clamp(-1.0,1.0);
    let pitch= t2.asin();

    let t3=2.0*(w*z+x*y);
    let t4=1.0-2.0*(ysqr+z*z);
    let yaw=t3.atan2(t4);

    Vector3::new(yaw,pitch,roll)
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le capteur MPU6050
    let i2c_bus = I2cdev::new("/dev/i2c-1")?;
    let mut mpu = Mpu6050::new(i2c_bus);
    mpu.init(&mut Delay).expect("erreure d'initialisation");


    let mut ahrs = Madgwick::new(1.0 / 100.0, 0.1);


    // Vecteurs pour stocker les données
    let mut roll_data = Vec::new();
    let mut pitch_data = Vec::new();
    let mut yaw_data = Vec::new();
    let mut timestamps = Vec::new();

    //let mut q=Quaternion::new(1.0,0.0,0.0,0.0);


    // Enregistrer les données pendant 10 secondes
    let start = Instant::now();
    //let mut last_update=start;
    while start.elapsed() < Duration::new(10, 0) {

        let gyro=mpu.get_gyro().unwrap();
        let acc=mpu.get_acc().unwrap();

        let gx=gyro.x.to_radians() as f64;
        let gy=gyro.y.to_radians() as f64;
        let gz=gyro.z.to_radians() as f64;
        let ax=acc.x as f64;
        let ay=acc.y as f64;
        let az=acc.z as f64;



        //let now=Instant::now();
        //let delta_t= (now-last_update).as_secs_f32();
        //last_update=now;
        //let q_new=Quaternion::new(1.0,gx*0.5*delta_t,gy*0.5*delta_t, gz*0.5*delta_t);

        //q=q*q_new;
        //q=q.normalize();
        //let euler=quat_to_euler(q);

        let quat=ahrs.update(
            &vector![gx, gy, gz],
            &vector![ax, ay, az],
            &vector![0.0, 0.0, 0.0],
          //  1.0 / 100.0,
        ).unwrap();

        let euler_angles = quat.euler_angles();
        let (yaw, pitch, roll) = (euler_angles.0, euler_angles.1, euler_angles.2);


        roll_data.push(roll as f32);
        pitch_data.push(pitch as f32);
        yaw_data.push(yaw as f32);
        timestamps.push(start.elapsed().as_secs_f32());

        thread::sleep(Duration::from_millis(100));
    }

    // Créer les répertoires nécessaires pour enregistrer les graphiques
    let dir_path = "plotters-doc-data";
    if !Path::new(dir_path).exists() {
        fs::create_dir_all(dir_path)?;
    }

    // Afficher les graphiques
    plot_data(&timestamps, &roll_data, "Roll", "plotters-doc-data/roll.png")?;
    plot_data(&timestamps, &pitch_data, "Pitch", "plotters-doc-data/pitch.png")?;
    plot_data(&timestamps, &yaw_data, "Yaw", "plotters-doc-data/yaw.png")?;

    Ok(())
}




// Fonction pour tracer les données
fn plot_data(timestamps: &[f32], data: &[f32], label: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(label, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..20f32, data.iter().cloned().fold(f32::INFINITY, f32::min)..data.iter().cloned().fold(f32::NEG_INFINITY, f32::max))?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        timestamps.iter().zip(data.iter()).map(|(&x, &y)| (x, y)),
        &RED,
    ))?
    .label(label)
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
*/

use ahrs::{Ahrs, Madgwick};
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::{Mpu6050, Mpu6050Error};
use nalgebra::{Vector3, vector};
use plotters::prelude::*;
use std::{fs, path::Path, thread, time::Duration, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le capteur MPU6050
    let i2c_bus = I2cdev::new("/dev/i2c-1")?;
    let mut mpu = Mpu6050::new(i2c_bus);
    mpu.init(&mut Delay)?;

    // AHRS Madgwick filter
    let mut ahrs = Madgwick::new(1.0 / 100.0, 0.1);

    // Vecteurs pour stocker les données
    let mut roll_data = Vec::new();
    let mut pitch_data = Vec::new();
    let mut yaw_data = Vec::new();
    let mut timestamps = Vec::new();

    // Enregistrer les données pendant 10 secondes
    let start = Instant::now();
    while start.elapsed() < Duration::new(10, 0) {
        // Lire les données du gyroscope et de l'accéléromètre
        let gyro = mpu.get_gyro()?;
        let accel = mpu.get_acc()?;

        // Convertir les données en f64
        let gx = gyro.x.to_radians() as f64;
        let gy = gyro.y.to_radians() as f64;
        let gz = gyro.z.to_radians() as f64;
        let ax = accel.x as f64;
        let ay = accel.y as f64;
        let az = accel.z as f64;

        // Mettre à jour le filtre AHRS Madgwick
        let quat = ahrs.update(
            &vector![gx, gy, gz],
            &vector![ax, ay, az],
            &vector![0.0, 0.0, 0.0],
        ).unwrap();

        // Obtenir les angles d'Euler
        let euler_angles = quat.euler_angles();
        let (yaw, pitch, roll) = (euler_angles.0, euler_angles.1, euler_angles.2);

        roll_data.push(roll as f32);
        pitch_data.push(pitch as f32);
        yaw_data.push(yaw as f32);
        timestamps.push(start.elapsed().as_secs_f32());

        thread::sleep(Duration::from_millis(100));
    }

    // Créer les répertoires nécessaires pour enregistrer les graphiques
    let dir_path = "plotters-doc-data";
    if !Path::new(dir_path).exists() {
        fs::create_dir_all(dir_path)?;
    }

    // Afficher les graphiques
    plot_data(&timestamps, &roll_data, "Roll", "plotters-doc-data/roll.png")?;
    plot_data(&timestamps, &pitch_data, "Pitch", "plotters-doc-data/pitch.png")?;
    plot_data(&timestamps, &yaw_data, "Yaw", "plotters-doc-data/yaw.png")?;

    Ok(())
}

// Fonction pour tracer les données
fn plot_data(timestamps: &[f32], data: &[f32], label: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(label, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..10f32, data.iter().cloned().fold(f32::INFINITY, f32::min)..data.iter().cloned().fold(f32::NEG_INFINITY, f32::max))?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        timestamps.iter().zip(data.iter()).map(|(&x, &y)| (x, y)),
        &RED,
    ))?
    .label(label)
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
