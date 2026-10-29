use std::thread;
use esp_idf_svc::hal::delay::{Ets, FreeRtos};
use esp_idf_svc::hal::gpio::{Output, PinDriver};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;
use pwm_pca9685::{Address, Channel, Pca9685};

// Hola, soy masoquista y ahora inverti la responsabilidad de ambos hilos

const CANALES: [Channel; 16] = [ // Los 16 canales a utilizar del PCA
    Channel::C0, Channel::C1, Channel::C2, Channel::C3,
    Channel::C4, Channel::C5, Channel::C6, Channel::C7,
    Channel::C8, Channel::C9, Channel::C10, Channel::C11,
    Channel::C12, Channel::C13, Channel::C14, Channel::C15,
];

const PAUSA_RAMPA_MS: u32 = 15; // Velocidad del incremento de brillo
const PAUSA_CANAL_MS: u32 = 50; // Espera con el canal al 100 %
const PAUSA_ESP_MS: u32 = 20; // Intervalo de los LEDs del ESP32

const LEDS_ESP: usize = 3; // LEDs asignados al hilo principal

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Inicialización del PCA9685
    let config = I2cConfig::new().baudrate(100u32.kHz().into());

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        pins.gpio8,
        pins.gpio9,
        &config,
    ).unwrap();

    let mut pwm = Pca9685::new(i2c, Address::default()).unwrap();
    pwm.set_prescale(5).unwrap();
    pwm.enable().unwrap();
    Ets::delay_us(1_000);

    // LEDs directos del ESP32
    let leds = [
        PinDriver::output(pins.gpio4).unwrap(),
        PinDriver::output(pins.gpio5).unwrap(),
        PinDriver::output(pins.gpio6).unwrap(),
    ];

    // Creación del hilo secundario dedicado al PCA9685
    thread::Builder::new()
        .name("hilo_pca".into())
        .stack_size(8 * 1024) // Por seguridad le dejo 8 Kb
        .spawn(move || hilo_pca(pwm)) // Recibe el bloque del PCA
        .unwrap();

    // El hilo principal controla exclusivamente los LEDs del ESP32
    hilo_leds(leds);
}

// Tareas del hilo secundario: control independiente del PCA9685
fn hilo_pca(mut pwm: Pca9685<I2cDriver<'static>>) {
    loop {
        for (n, canal) in CANALES.iter().enumerate() {
            // Rampa ascendente de encendido
            for pct in 0u8..=100 {
                set_percent(&mut pwm, *canal, pct);
                FreeRtos::delay_ms(PAUSA_RAMPA_MS);
            }
            log::info!("Canal {n} al 100 %");
            FreeRtos::delay_ms(PAUSA_CANAL_MS);
            // Apagado total antes de pasar al siguiente canal
            set_percent(&mut pwm, *canal, 0);
        }
        FreeRtos::delay_ms(50);
    }
}

// Tareas del hilo principal: control de los LEDs del ESP32
fn hilo_leds(mut leds: [PinDriver<'static, Output>; LEDS_ESP]) {
    loop {
        // Enciende cada LED, uno por uno
        for activo in 0..LEDS_ESP {
            for (i, led) in leds.iter_mut().enumerate() {
                if i == activo {
                    led.set_high().unwrap();
                } else {
                    led.set_low().unwrap();
                }
            }
            FreeRtos::delay_ms(PAUSA_ESP_MS);
        }
        // Apaga los tres LEDs mientras el PCA sigue trabajando
        for led in leds.iter_mut() {
            led.set_low().unwrap();
        }
        FreeRtos::delay_ms(PAUSA_ESP_MS);
    }
}

// Función para asignar un porcentaje de encendido a un canal del PCA
fn set_percent(pwm: &mut Pca9685<I2cDriver<'_>>, canal: Channel, pct: u8,) {
    match pct {
        0 => pwm.set_channel_full_off(canal).unwrap(),
        100.. => pwm.set_channel_full_on(canal, 0).unwrap(),
        p => pwm.set_channel_on_off(canal,0,(p as u32 * 4096 / 100) as u16,).unwrap(),
    }
}