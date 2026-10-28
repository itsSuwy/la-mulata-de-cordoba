use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::delay::FreeRtos;
fn main() {
    // Casi identico al ejemplo de practica_rele.rs, pero en este ejemplo se utiliza un MOC y la practica esta mas enfocada a simular un relampago

    esp_idf_svc::sys::link_patches(); 
    esp_idf_svc::log::EspLogger::initialize_default(); 
    let peripherals = Peripherals::take().unwrap(); 
    let pins = peripherals.pins;
    let mut moc = PinDriver::output(pins.gpio42).unwrap(); // Se asigna el pint 42 al MOC

    loop {
        for _ in 0..5 { // Parpadeo inicial del MOC para simular un relampago
            moc .set_high().unwrap(); 
            log::info!("MOC encendido");
            FreeRtos::delay_ms(50);

            moc.set_low().unwrap(); 
            log::info!("MOC apagado");
            FreeRtos::delay_ms(50); 
        }
        FreeRtos::delay_ms(100); // Espera para el relampago final

        moc.set_high().unwrap(); 
        log::info!("MOC encendido");
        FreeRtos::delay_ms(500);

        moc.set_low().unwrap(); 
        log::info!("MOC apagado");
        FreeRtos::delay_ms(500); 
    }
}