use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Channel, Pca9685, Address};
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialiser la connexion I2C
    let i2c = I2cdev::new("/dev/i2c-1")?; // Bus I2C sur lequel le PCA9685 est détecté
    let address = Address::default(); // Adresse par défaut du PCA9685 (0x40)

    // Initialiser le module PCA9685
    let mut pwm = Pca9685::new(i2c, address).map_err(|e| format!("Erreur lors de l'initialisation du PCA9685: {:?}", e))?;
    pwm.enable().map_err(|e| format!("Erreur lors de l'activation du PCA9685: {:?}", e))?;
    pwm.set_prescale(50).map_err(|e| format!("Erreur lors de la configuration de la prescale: {:?}", e))?; // Fréquence PWM d'environ 100 Hz pour l'ESC

    // Étape d'initialisation : Envoyer le signal "neutre" à l'ESC (1500 microsecondes)
    pwm.set_channel_on_off(Channel::C0, 0, 3072).map_err(|e| format!("Erreur lors de l'envoi du signal neutre à l'ESC: {:?}", e))?;
    println!("Signal neutre envoyé à l'ESC. Attente de reconnaissance.");

    // Attendre que l'ESC soit prêt (8 secondes comme dans le code Arduino)
    thread::sleep(Duration::from_secs(8));

    // Augmenter progressivement le signal pour l'initialisation de l'ESC
    for pwm_val in (1100..=1900).step_by(200) {
        pwm.set_channel_on_off(Channel::C0, 0, pwm_val).map_err(|e| format!("Erreur lors de l'envoi du signal PWM: {:?}", e))?;
        println!("Valeur PWM envoyée pour l'initialisation: {}", pwm_val);
        thread::sleep(Duration::from_secs(2));
    }

    // Contrôler le moteur avec une valeur fixe de PWM
    let pwm_val = 1600; // Valeur fixe pour faire tourner le moteur
    pwm.set_channel_on_off(Channel::C0, 0, pwm_val).map_err(|e| format!("Erreur lors de l'envoi du signal PWM pour le moteur: {:?}", e))?;
    println!("Valeur PWM envoyée pour faire tourner le moteur: {}", pwm_val);

    // Laisser tourner le moteur pendant 10 secondes
    thread::sleep(Duration::from_secs(10));

    // Arrêter le moteur
    pwm.set_channel_on_off(Channel::C0, 0, 0).map_err(|e| format!("Erreur lors de l'arrêt du moteur: {:?}", e))?;
    println!("Moteur arrêté.");

    Ok(())
}
