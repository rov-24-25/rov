use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Channel, Pca9685, Address};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Ouvrir la connexion I2C
    let i2c = I2cdev::new("/dev/i2c-1")?; //Il faut qu'on installe I2C et qu'on vérifie le chemin du fichier
    let address = Address::default(); // C'est l'adresse par défaut du PCA9685 (0x40)
    
    // Initialiser le module PCA9685
    let mut pwm = Pca9685::new(i2c, address)?;
    pwm.enable()?;
    pwm.set_prescale(100)?; // Fréquence PWM ~50 Hz adaptée aux moteurs

    // Démarrer les moteurs sur certains canaux
    pwm.set_channel_on_off(Channel::C0, 0, 2048)?; // Canal 0 à 50% duty cycle

    println!("Moteur activé. Le canal envoie des signaux PWM.");

    std::thread::sleep(std::time::Duration::from_secs(10));

    // Arrêter les moteurs
    pwm.set_channel_on_off(Channel::C0, 0, 0)?; 
    println!("Moteurr arrêté.");

    Ok(())
}
