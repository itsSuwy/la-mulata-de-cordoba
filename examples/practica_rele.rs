use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::delay::FreeRtos;
fn main() {
    esp_idf_svc::sys::link_patches(); 
    esp_idf_svc::log::EspLogger::initialize_default(); 
    let peripherals = Peripherals::take().unwrap(); 
    let pins = peripherals.pins;
    let mut rele = PinDriver::output(pins.gpio42).unwrap(); // Se asigna el pint 5 a una variable de caracter mutable

    loop {
        rele.set_high().unwrap(); 
        log::info!("Rele encendido");
        FreeRtos::delay_ms(500); 

        rele.set_low().unwrap(); 
        log::info!("Rele apagado");
        FreeRtos::delay_ms(500);
    }
}