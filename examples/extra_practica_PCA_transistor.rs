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

    let canales = [Channel::C0]; // Canal a controlar del PCA

    loop {
        for canal in canales {
            set_percent(&mut pwm, canal, 100); // destello al máximo de golpe
            FreeRtos::delay_ms(100);           // se sostiene un instante

            // Decae rápido: 100, 95, 90 ... 0
            for pct in (0u8..=100).rev().step_by(5) {
                set_percent(&mut pwm, canal, pct);
                FreeRtos::delay_ms(20);        // 21 pasos × 20 ms ≈ 420 ms
            }
        }
        FreeRtos::delay_ms(2_000);             // pausa antes del siguiente
    }
}

fn set_percent(pwm: &mut Pca9685<I2cDriver<'_>>, canal: Channel, pct: u8) { // Función para asignar un porcentaje de encendido a un canal del PCA
    match pct { // Se asigna el porcentaje de encendido al canal del PCA
        0 => pwm.set_channel_full_off(canal).unwrap(),        // apagado total
        100.. => pwm.set_channel_full_on(canal, 0).unwrap(),  // encendido total
        p => pwm.set_channel_on_off(canal, 0, (p as u32 * 4096 / 100) as u16).unwrap(), // encendido parcial
    }
}