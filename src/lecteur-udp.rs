use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let address = "0.0.0.0:8080";
    let socket = UdpSocket::bind(address)?;

    println!("En écoute sur {}", address);

    let mut buf = [0u8; 1024]; // Tampon pour stocker les données reçues.

    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                let received_data = &buf[..size];
                println!(
                    "Reçu {} octets de {} : {:?}",
                    size,
                    src,
                    received_data
                );

                // Si les données sont des floats (par exemple, envoyées avec `struct.pack('f')` en Python)
                if let Ok(value) = bincode::deserialize::<f32>(received_data) {
                    println!("Donnée reçue (float): {}", value);
                } else {
                    println!("Impossible de convertir les données en float.");
                }
            }
            Err(e) => eprintln!("Erreur lors de la réception: {}", e),
        }
    }
}
