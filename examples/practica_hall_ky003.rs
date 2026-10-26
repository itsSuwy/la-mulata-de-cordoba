use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Señal (S) del KY-003 en GPIO17. El módulo ya trae su propia pull-up.
    let sensor = PinDriver::input(pins.gpio17, Pull::Floating).unwrap();

    // La señal es activa en bajo: baja cuando el imán está cerca
    let mut detectado_antes = sensor.is_low();
    log::info!("Estado inicial: {}", if detectado_antes { "IMAN DETECTADO" } else { "sin iman" });

    loop {
        let detectado = sensor.is_low();
        if detectado != detectado_antes {
            log::info!("{}", if detectado { "IMAN DETECTADO" } else { "sin iman" });
            detectado_antes = detectado;
        }
        FreeRtos::delay_ms(10);
    }
}