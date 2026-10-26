use esp_idf_svc::hal::adc::attenuation::DB_12;
use esp_idf_svc::hal::adc::oneshot::{config::AdcChannelConfig, AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // ADC1 con la salida analógica del KY-024 (A0) en GPIO4
    let adc = AdcDriver::new(peripherals.adc1).unwrap();
    let adc_config = AdcChannelConfig {
        attenuation: DB_12, // rango de entrada de ~0 a 3.1 V
        ..Default::default()
    };
    let mut analogico = AdcChannelDriver::new(&adc, pins.gpio4, &adc_config).unwrap();

    // Salida digital del comparador (D0), solo para observarla
    let digital = PinDriver::input(pins.gpio5, Pull::Floating).unwrap();

    // Lectura en reposo: promedio de 50 muestras, con el imán lejos
    log::info!("Midiendo reposo, no acerques el iman...");
    let mut suma: u32 = 0;
    for _ in 0..50 {
        suma += adc.read(&mut analogico).unwrap() as u32;
        FreeRtos::delay_ms(10);
    }
    let reposo = (suma / 50) as i32;
    log::info!("Reposo = {reposo}");

    loop {
        let lectura = adc.read(&mut analogico).unwrap() as i32;
        let delta = lectura - reposo;
        let d0 = if digital.is_high() { "alto" } else { "bajo" };
        log::info!("lectura: {lectura:4} | delta: {delta:+5} | D0: {d0}");
        FreeRtos::delay_ms(100);
    }
}