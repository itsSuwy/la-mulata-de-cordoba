use std::thread;
use esp_idf_svc::hal::delay::{Ets, FreeRtos};
use esp_idf_svc::hal::gpio::{Output, PinDriver};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;
use pwm_pca9685::{Address, Channel, Pca9685};
// Hola este codigo fue una tortura pero ya funciona

const CANALES: [Channel; 16] = [ // Los 16 canales a utilizar del PCA
    Channel::C0, Channel::C1, Channel::C2, Channel::C3,
    Channel::C4, Channel::C5, Channel::C6, Channel::C7,
    Channel::C8, Channel::C9, Channel::C10, Channel::C11,
    Channel::C12, Channel::C13, Channel::C14, Channel::C15,
];

const PAUSA_RAMPA_MS: u32 = 15; // Velocidad del incremento de brillo (rampa PWM)
const PAUSA_CANAL_MS: u32 = 50; // Tiempo de espera con el canal al 100%
const PAUSA_ESP_MS: u32 = 20; // Intervalo de parpadeo para los LEDs directos del ESP32
const LEDS_ESP: usize = 3; // Los leds asignados directamente al segundo hilo

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // PCA9685: permanece en el hilo principal.
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

    // El hilo secundario controla exclusivamente los GPIO del ESP32.
    let leds = [
        PinDriver::output(pins.gpio4).unwrap(),
        PinDriver::output(pins.gpio5).unwrap(),
        PinDriver::output(pins.gpio6).unwrap(),
    ];

    // Creacion del hilo secundario dedicado exclusivamente a los leds que se asignaron arriba
    thread::Builder::new()
        .name("leds_esp".into())
        .stack_size(4 * 1024) // Se le asigna 4 KB de Stack dentro de FreeRTOS para no saturar memoria
        .spawn(move || hilo_leds(leds))
        .unwrap();

    // Este sera el hilo principal
    loop {
        for (n, canal) in CANALES.iter().enumerate() {
            for pct in 0u8..=100 { // En este for vivira la rampa ascendente de encendido
                set_percent(&mut pwm, *canal, pct);
                FreeRtos::delay_ms(PAUSA_RAMPA_MS);
            }
            log::info!("Canal {n} al 100 %");
            FreeRtos::delay_ms(PAUSA_CANAL_MS);
            set_percent(&mut pwm, *canal, 0); // Apagado total del canal antes de pasar al siguiente <- Tendre que usar OE por un problema detectado entre flasheos de codigos entre el PCA y la ESP, lease docs/PCA para mas informacion
        }
        FreeRtos::delay_ms(50);
    }
}

// Estas seran las tareas del hilo secundario a realizar
fn hilo_leds(mut leds: [PinDriver<'static, Output>; LEDS_ESP],) { // No resulta bloqueante en el trabajo del hilo principal y este trabajara por su propia cuenta
    loop {
        // Enciende cada LED, uno por uno.
        for activo in 0..LEDS_ESP { // Aqui solo enciendo un led si se encuentra activo <- Lo refactorizare para despues para simplificarlo mas (TODO)
            for (i, led) in leds.iter_mut().enumerate() {
                if i == activo {
                    led.set_high().unwrap();
                } else {
                    led.set_low().unwrap();
                }
            }
            FreeRtos::delay_ms(PAUSA_ESP_MS);
        }
        // Apaga los tres LED mientras el PCA sigue trabajando.
        for led in leds.iter_mut() {
            led.set_low().unwrap();
        }
        FreeRtos::delay_ms(PAUSA_ESP_MS);
    }
}

fn set_percent(pwm: &mut Pca9685<I2cDriver<'_>>, canal: Channel, pct: u8) { // Función para asignar un porcentaje de encendido a un canal del PCA
    match pct { // Se asigna el porcentaje de encendido al canal del PCA
        0 => pwm.set_channel_full_off(canal).unwrap(),        // apagado total
        100.. => pwm.set_channel_full_on(canal, 0).unwrap(),  // encendido total
        p => pwm.set_channel_on_off(canal, 0, (p as u32 * 4096 / 100) as u16).unwrap(), // encendido parcial
    }
}