use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::Notify;
use async_stream::stream;
use tokio::task;
use tokio::time::interval;
use tokio::process::Command;
use std::time::Duration;
use tokio::net::TcpListener;
use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Channel, Pca9685, Address};
use std::error::Error;
use linux_embedded_hal_mpu::Delay as DelayMPU;
use linux_embedded_hal_mpu::I2cdev as I2cMPU;
use mpu6050::*;

struct SharedState {
    pitch: Mutex<f32>,
    roll: Mutex<f32>,
    frame: Mutex<Vec<u8>>,
    notify: Notify,
}


async fn index_handler(_: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let content = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>MPU6050 Data and Video Feed</title>
        <style>
            #graph {
                border: 1px solid black;
                width: 400px;
                height: 400px;
                position: relative;
                display: inline-block;
            }
            #graph::before, #graph::after {
                content: '';
                position: absolute;
                background: black;
            }
            #graph::before {
                width: 1px;
                height: 100%;
                top: 0;
                left: 50%;
            }
            #graph::after {
                width: 100%;
                height: 1px;
                top: 50%;
                left: 0;
            }
            #point {
                position: absolute;
                width: 10px;
                height: 10px;
                background: red;
                border-radius: 50%;
                transform: translate(-50%, -50%);
            }
            #video {
                display: inline-block;
                vertical-align: top;
            }
        </style>
        <script>
            let videoInterval;
            let capturing = false;

            async function fetchData() {
                const response = await fetch('/data');
                const data = await response.json();
                document.getElementById('roll').innerText = `Roll: ${data.roll.toFixed(2)}`;
                document.getElementById('pitch').innerText = `Pitch: ${data.pitch.toFixed(2)}`;

                const graph = document.getElementById('graph');
                const point = document.getElementById('point');

                const centerX = graph.clientWidth / 2;
                const centerY = graph.clientHeight / 2;
                const scale = centerX / 90; // scale factor for -90 to 90 degrees

                const x = centerX + data.roll * scale;
                const y = centerY - data.pitch * scale;

                point.style.left = `${x}px`;
                point.style.top = `${y}px`;
            }

            function captureImage() {
                const videoElement = document.querySelector('#video img');
                const canvas = document.createElement('canvas');
                canvas.width = videoElement.width;
                canvas.height = videoElement.height;
                const context = canvas.getContext('2d');
                context.drawImage(videoElement, 0, 0, canvas.width, canvas.height);
                const dataURL = canvas.toDataURL('image/jpeg');

                const link = document.createElement('a');
                link.href = dataURL;
                link.download = 'capture.jpg';
                link.click();
            }

            function captureVideo() {
                const button = document.getElementById('videoButton');
                if (!capturing) {
                    button.innerText = 'Stop Video';
                    capturing = true;
                    videoInterval = setInterval(() => {
                        const videoElement = document.querySelector('#video img');
                        const canvas = document.createElement('canvas');
                        canvas.width = videoElement.width;
                        canvas.height = videoElement.height;
                        const context = canvas.getContext('2d');
                        context.drawImage(videoElement, 0, 0, canvas.width, canvas.height);
                        const dataURL = canvas.toDataURL('image/jpeg');

                        const link = document.createElement('a');
                        link.href = dataURL;
                        link.download = `capture_${Date.now()}.jpg`;
                        link.click();
                    }, 500); // Capture image every 500 milliseconds
                } else {
                    button.innerText = 'Start Video';
                    capturing = false;
                    clearInterval(videoInterval);
                }
            }

            setInterval(fetchData, 500); // Fetch data every 500 milliseconds
        </script>
    </head>
    <body>
        <h1>MPU6050 Data and Video Feed</h1>
        <div id="video">
            <img src="/video_feed" width="640" height="480" />
        </div>
        <div id="graph">
            <div id="point"></div>
        </div>
        <pre id="roll">Roll: </pre>
        <pre id="pitch">Pitch: </pre>
        <button onclick="captureImage()">Capture Image</button>
        <button id="videoButton" onclick="captureVideo()">Start Video</button>
    </body>
    </html>
    "#;

    Ok(Response::builder()
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .unwrap())
}



async fn video_feed_handler(state: Arc<SharedState>) -> Result<Response<Body>, hyper::Error> {
    let boundary = "frame";
    let res = Response::builder()
        .header("Content-Type", format!("multipart/x-mixed-replace; boundary={}", boundary))
        .body(Body::wrap_stream(stream! {
            loop {
                state.notify.notified().await;
                let frame = state.frame.lock().unwrap().clone();
                let mut data = Vec::new();
                data.extend_from_slice(b"--frame\r\n");
                data.extend_from_slice(b"Content-Type: image/jpeg\r\n\r\n");
                data.extend_from_slice(&frame);
                data.extend_from_slice(b"\r\n");
                yield Ok::<_, hyper::Error>(data);
            }
        }))
        .unwrap();
    Ok(res)
}


async fn handle_request(_req: Request<Body>, state: Arc<SharedState>) -> Result<Response<Body>, hyper::Error> {
    let roll = *state.roll.lock().unwrap();
    let pitch = *state.pitch.lock().unwrap();
    let body = format!("{{\"roll\": {}, \"pitch\": {}}}", roll, pitch);
    Ok(Response::new(Body::from(body)))
}


async fn read_mpu6050(state: Arc<SharedState>) -> Result<(), Box<dyn Error>> {
    let i2c = I2cMPU::new("/dev/i2c-3").map_err(|e| format!("Failed to open I2C device: {:?}", e))?;
    let mut mpu = Mpu6050::new(i2c);
    let mut delay = DelayMPU;
    mpu.init(&mut delay).expect("Failed to initialize MPU6050");

    let mut mean_roll = 0.0;
    let mut mean_pitch = 0.0;
    let mut samples = 0;
    let mut prec_rol=0.0;
    let mut prec_pitch=0.0;

    let mut interval = interval(Duration::from_millis(50));
    loop {
        interval.tick().await;
        let accel = mpu.get_acc_angles().unwrap();
        let roll = accel[0].to_degrees() as f32;
        let pitch = accel[1].to_degrees() as f32;

        {
            let mut roll_lock = state.roll.lock().unwrap();
            let mut pitch_lock = state.pitch.lock().unwrap();
            if samples < 200 {
                mean_roll += roll;
                mean_pitch += pitch;
                samples += 1;
            } else if samples == 200 {
                mean_roll /= 200.0;
                mean_pitch /= 200.0;
                samples += 1;
            } else {
                if abs(prec_roll-roll)>4{
                    *roll_lock = roll - mean_roll;
                    prec_roll=*roll_lock}
                if abs(prec_pitch-pitch)>4{
                *pitch_lock = pitch - mean_pitch;
                prec_pitch=*pitch_lock}
            }
        }
    }
}


async fn captur_vid(state_clone: Arc<SharedState>){
    let (reader, writer) = UnixStream::pair().expect("Failed to create UnixStream pair");
    let writer_fd = writer.as_raw_fd();
    let stdio_writer = unsafe { Stdio::from_raw_fd(writer_fd) };
    let mut child = Command::new("libcamera-vid")
            .arg("--width")
            .arg("1640")
            .arg("--height")
            .arg("1232")
            .arg("--codec")
            .arg("mjpeg")
            .arg("--inline")
            .arg("--timeout")
            .arg("0")
            .arg("-o")
            .arg("-")
            .stdout(stdio_writer)
            .spawn()
            .expect("Failed to start libcamera-vid");

        let mut reader = BufReader::new(reader);
        let mut temp_buffer = Vec::new();

        loop {
            let mut buffer = vec![0; 65536];
            match reader.read(&mut buffer).await {
                Ok(size) if size > 0 => {
                    buffer.truncate(size);
                    temp_buffer.extend_from_slice(&buffer);
                    while let Some(pos) = temp_buffer.windows(2).position(|w| w == b"\xff\xd9") {
                        let mut frame = temp_buffer.split_off(pos + 2);
                        std::mem::swap(&mut temp_buffer, &mut frame);
                        frame.truncate(pos + 2);
                        {
                            let mut state_frame = state_clone.frame.lock().unwrap();
                            *state_frame = frame;
                        }
                        state_clone.notify.notify_one();
                    }
                }
                Ok(_) => break,
                Err(e) => {
                    eprintln!("Error reading from libcamera-vid: {}", e);
                    break;
                }
            }
        }

        // Ensure the child process is awaited to ensure it exits
        let _ = child.wait().await.expect("Failed to wait on child process");
    }



async fn pca_task()->Result<(),Box<dyn Error>>{
        let listener = TcpListener::bind("0.0.0.0:12345").await?;
        println!("connected");

        let i2c_pca = I2cdev::new("/dev/i2c-1")
        .map_err(|e| format!("Failed to open I2C device: {:?}", e))?;
        //let mut delay = Delay;

    // Attempt to initialize PCA9685
    let pca_address = Address::default(); // default I2C address for PCA9685
    let mut pca= Pca9685::new(i2c_pca, pca_address).unwrap();
        //pca.init(&mut delay);
    pca.set_prescale(127).unwrap();

    pca.enable().unwrap();
    pca.set_channel_on_off(Channel::C2,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C1,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C0,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C5,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C4,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C3,0, 307).unwrap();
    tokio::time::sleep(Duration::from_millis(5000)).await;

    pca.set_channel_on_off(Channel::C8,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C9,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C10,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C11,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C12,0, 307).unwrap();
    pca.set_channel_on_off(Channel::C13,0, 307).unwrap();

    let mut cam=156.0;
    println!("everything is initialized");

    while let Ok((mut socket, addr)) = listener.accept().await {
        println!("connection from {}", addr);

        let mut buf = [0u8; 48];
        while let Ok(_) = socket.read_exact(&mut buf).await {
            let data_points = [
                f32::from_le_bytes(buf[0..4].try_into().unwrap()), // x
                f32::from_le_bytes(buf[4..8].try_into().unwrap()), // y
                f32::from_le_bytes(buf[8..12].try_into().unwrap()), // z
                f32::from_le_bytes(buf[12..16].try_into().unwrap()), // rot
                f32::from_le_bytes(buf[16..20].try_into().unwrap()),
                f32::from_le_bytes(buf[20..24].try_into().unwrap()),
                f32::from_le_bytes(buf[24..28].try_into().unwrap()),
                f32::from_le_bytes(buf[28..32].try_into().unwrap()),
                f32::from_le_bytes(buf[32..36].try_into().unwrap()),
                f32::from_le_bytes(buf[36..40].try_into().unwrap()),
                f32::from_le_bytes(buf[40..44].try_into().unwrap()),
                f32::from_le_bytes(buf[44..48].try_into().unwrap())
            ];
            println!("received data: {:?}", data_points);

            let b0 = data_points[6];
            let b1 = data_points[7];
            let b2 = data_points[8];
            let b3 = data_points[9];
            let b4 = data_points[10];

            cam += data_points[11];


            let h2 = 307.0 - data_points[5] * 50.0 + data_points[4] * 25.0;
            let h1 = 307.0 - data_points[5] * 50.0 - data_points[4] * 25.0;
            let h3 = 307.0 + data_points[3] * 50.0 - data_points[4] * 30.0;
            let h4 = 307.0 - data_points[2] * 15.0 + data_points[0] * 30.0 + data_points[1] * 15.0;
            let h5 = 307.0 - data_points[2] * 15.0 - data_points[0] * 30.0 + data_points[1] * 15.0;
            let h6 = 307.0 - data_points[2] * 30.0 - data_points[1] * 30.0;

            pca.set_channel_on_off(Channel::C2, 0, h1.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C3, 0, h2.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C1, 0, h3.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C0, 0, h5.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C4, 0, h4.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C5, 0, h6.round() as u16).unwrap();

            pca.set_channel_on_off(Channel::C8, 0, b0.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C9, 0, b1.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C10, 0, b2.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C11, 0, b3.round() as u16).unwrap();
            pca.set_channel_on_off(Channel::C12, 0, b4.round() as u16).unwrap();


            if cam>=306.0 {
                pca.set_channel_on_off(Channel::C13, 0, 306.0_f32.round() as u16).unwrap();
            }
            else if cam<=0.0 {
                pca.set_channel_on_off(Channel::C13, 0, 0.0_f32.round() as u16).unwrap();
            }
            else{
                pca.set_channel_on_off(Channel::C13, 0, cam.round() as u16).unwrap();
            }

            



            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    Ok(())
}






#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let state = Arc::new(SharedState {
        pitch: Mutex::new(0.0),
        roll: Mutex::new(0.0),
        frame: Mutex::new(Vec::new()),
        notify: Notify::new(),
    });

    let state_clone_mpu = Arc::clone(&state);
    task::spawn(async move {
        read_mpu6050(state_clone_mpu).await.unwrap();
    });


    let make_svc_state = Arc::clone(&state);

    let make_svc = make_service_fn(move |_| {
    let state = Arc::clone(&make_svc_state);
    async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let state = Arc::clone(&state);
                async move {
                    match req.uri().path() {
                        "/video_feed" => video_feed_handler(state).await,
                        "/data" => handle_request(req, state).await,
                        "/" => index_handler(req).await,
                        _ => Ok(Response::builder()
                            .status(404)
                            .body(Body::from("Not Found"))
                            .unwrap()),
                    }
                }
            }))
        }
    });
    let addr = ([0, 0, 0, 0], 8000).into();
    let server = Server::bind(&addr).serve(make_svc);

    let server_handle = task::spawn(async {
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });
    let state_clone = Arc::clone(&state);
    let video_capture_handle = task::spawn(captur_vid(state_clone));

    let _pca_handle = task::spawn(async {
        if let Err(e) = pca_task().await {
            eprintln!("PCA9685 task error: {}", e);
        }
    });
    tokio::try_join!(server_handle, video_capture_handle)?;

    Ok(())
}