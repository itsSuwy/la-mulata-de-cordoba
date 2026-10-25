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
        rele.set_high().unwrap(); // Se enciende el rele
        log::info!("Bobina encendida");
        FreeRtos::delay_ms(500);

        rele.set_low().unwrap(); // Se apaga el rele
        log::info!("Bobina apagada");
        FreeRtos::delay_ms(500); // Delay de 500 ms
        //panic!("Panic controlado para ver el funcionamiento de esta funcion"); 
    }
}