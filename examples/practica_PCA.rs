use esp_idf_svc::hal::delay::{Ets, FreeRtos};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType; // Permite escribir 100u32.kHz()
use pwm_pca9685::{Address, Channel, Pca9685};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Asignar al bus I2C (SDA en GPIO8, SCL en GPIO9, a 100 kHz)
    let config = I2cConfig::new().baudrate(100u32.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, pins.gpio8, pins.gpio9, &config).unwrap();

    let mut pwm = Pca9685::new(i2c, Address::default()).unwrap(); // Configurar el chip con la dirección por defecto (0x40)

    pwm.set_prescale(5).unwrap(); // Asigna el prescaler a 5, lo que da una frecuencia de 1000 Hz

    pwm.enable().unwrap();
    Ets::delay_us(1_000); // Despertar al PCA y esperar a que el oscilador se estabilice

    let canales = [Channel::C0, Channel::C1, Channel::C2]; // Canales a controlar del PCA

    loop {
        // Cada LED subira de 0 a 100 % en secuencia y se queda encendido
        for canal in canales {
            for pct in 0u8..=100 {
                set_percent(&mut pwm, canal, pct);
                //log::info!("Canal {} al porcentaje {pct}", canal as u8); // Ligero log para ver el porcentaje de cada canal
                FreeRtos::delay_ms(15);
            }
        }
        FreeRtos::delay_ms(600);

        // Se apagan los tres para repetir
        for canal in canales {
            set_percent(&mut pwm, canal, 0);
        }
        FreeRtos::delay_ms(500);
    }
}

fn set_percent(pwm: &mut Pca9685<I2cDriver<'_>>, canal: Channel, pct: u8) { // Función para asignar un porcentaje de encendido a un canal del PCA
    match pct { // Se asigna el porcentaje de encendido al canal del PCA
        0 => pwm.set_channel_full_off(canal).unwrap(),        // apagado total
        100.. => pwm.set_channel_full_on(canal, 0).unwrap(),  // encendido total
        p => pwm.set_channel_on_off(canal, 0, (p as u32 * 4096 / 100) as u16).unwrap(), // encendido parcial
    }
}