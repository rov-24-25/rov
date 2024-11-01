use linux_embedded_hal::{Delay, I2cdev};
//use embedded_hal::blocking::delay::DelayMs;
use mpu6050::*;
use std::error::Error;
//use std::f32::consts::PI;
//use piston_window::{PistonWindow, WindowSettings};
use plotters::prelude::*;
//use plotters_piston::PistonBackend;
//use tokio::runtime::Runtime;



fn main()->Result<(),Box<dyn Error>>{
    let i2c_mpu=I2cdev::new("/dev/i2c-1")?;//.map_err(|e| format!("Failed to open I2C device: {:?}", e))?;
    let mut mpu=Mpu6050::new(i2c_mpu);
    let mut delay=Delay;
    mpu.init(&mut delay);
    
    loop{
        let accel=mpu.get_acc_angles()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}


/* 
    let mut window: PistonWindow = WindowSettings::new("Real Time MPU6050 Data", [700, 700])
        .exit_on_esc(true)
        .build()
        .unwrap();
    let rt = Runtime::new()?;

    rt.block_on(async move {
        while let Some(e)=window.next() {
            window.draw_2d(&e, |c, g, _device| {
                let root = PistonBackend::new((700, 700),1.0, c,g).into_drawing_area();
                root.fill(&WHITE).unwrap();
                let mut chart = ChartBuilder::on(&root)
                    .caption("MPU6050 Data", ("sans-serif", 50).into_font())
                    .margin(10)
                    .x_label_area_size(30)
                    .y_label_area_size(30)
                    .build_cartesian_2d(-180f32..180f32, -180f32..180f32)
                    .unwrap();
                chart.configure_mesh().draw().unwrap();
                let angles = mpu.get_acc_angles().unwrap(); // Assume get_acc_angles returns Result<Vector2<f32>, Error>
                chart.draw_series(PointSeries::of_element(
                    vec![(angles.x * 180. / PI, angles.y * 180. / PI)],
                    5,
                    &RED,
                    &|coord, size, style| {
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
                    },
                )).unwrap();
            });
            // Introduce delay or wait for next frame event
            dlay.delay_ms(100u16);
        }
    });

    Ok(())
}
*/






