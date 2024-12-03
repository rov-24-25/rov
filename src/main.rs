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
    let mut pwm = Pca9685::new(i2c, address)
        .map_err(|e| format!("Erreur lors de l'initialisation du PCA9685: {:?}", e))?;
    pwm.enable()
        .map_err(|e| format!("Erreur lors de l'activation du PCA9685: {:?}", e))?;
    pwm.set_prescale(25)
        .map_err(|e| format!("Erreur lors de la configuration de la prescale: {:?}", e))?; // Fréquence PWM d'environ 100 Hz pour l'ESC

    // Étape d'initialisation : Envoyer le signal "neutre" à l'ESC pour chaque moteur
    for &channel in &[Channel::C0, Channel::C1, Channel::C2, Channel::C3] {
        pwm.set_channel_on_off(channel, 0, 3072)
            .map_err(|e| format!("Erreur lors de l'envoi du signal neutre au canal {:?}: {:?}", channel, e))?;
        println!("Signal neutre envoyé à l'ESC sur le canal {:?}", channel);
    }

    println!("Attente de reconnaissance des ESCs.");
    thread::sleep(Duration::from_secs(8)); // Attente pour l'initialisation de l'ESC

    // Augmenter progressivement le signal pour l'initialisation de chaque ESC
    for pwm_val in (1100..=1900).step_by(200) {
        for &channel in &[Channel::C0, Channel::C1, Channel::C2, Channel::C3] {
            pwm.set_channel_on_off(channel, 0, pwm_val)
                .map_err(|e| format!("Erreur lors de l'envoi du signal PWM au canal {:?}: {:?}", channel, e))?;
            println!(
                "Valeur PWM envoyée pour l'initialisation du canal {:?}: {}",
                channel, pwm_val
            );
        }
        thread::sleep(Duration::from_secs(2));
    }

    // Contrôler chaque moteur avec une valeur fixe de PWM
    let pwm_val = 1600; // Valeur fixe pour faire tourner les moteurs
    for &channel in &[Channel::C0, Channel::C1, Channel::C2, Channel::C3] {
        pwm.set_channel_on_off(channel, 0, pwm_val)
            .map_err(|e| format!("Erreur lors de l'envoi du signal PWM au canal {:?}: {:?}", channel, e))?;
        println!(
            "Valeur PWM envoyée pour faire tourner le moteur sur le canal {:?}: {}",
            channel, pwm_val
        );
    }

    // Laisser tourner les moteurs pendant 10 secondes
    thread::sleep(Duration::from_secs(10));

    // Arrêter tous les moteurs
    for &channel in &[Channel::C0, Channel::C1, Channel::C2, Channel::C3] {
        pwm.set_channel_on_off(channel, 0, 0)
            .map_err(|e| format!("Erreur lors de l'arrêt du moteur sur le canal {:?}: {:?}", channel, e))?;
        println!("Moteur arrêté sur le canal {:?}", channel);
    }

    Ok(())
}
