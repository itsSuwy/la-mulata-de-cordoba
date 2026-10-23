use esp_idf_svc::hal::delay::{Ets, FreeRtos}; // Para el control del tiempo
use esp_idf_svc::hal::gpio::{PinDriver, Pull}; // Para las entradas y salidas digitales
use esp_idf_svc::hal::peripherals::Peripherals; 
use std::time::Instant;

fn main() {
    // Configuraciones iniciales
    esp_idf_svc::sys::link_patches(); // Se aplica el parche
    esp_idf_svc::log::EspLogger::initialize_default(); // Para imprimir en terminal
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Bloque logico
    let echo = PinDriver::input(pins.gpio10, Pull::Down).unwrap(); // Recibe la entrada del sensor para saber si detecta algo 
    let mut trig = PinDriver::output(pins.gpio11).unwrap(); // Envia el pulso para que el sensor dispare la onda
    trig.set_low().unwrap(); // El sensor empieza en bajo

    // Ciclo principal
    loop {
        trig.set_low().unwrap(); 
        Ets::delay_us(2); 
        trig.set_high().unwrap(); // El sensor dispara
        Ets::delay_us(10); // Durante 10 microsegundos el pin de disparo se mantiene en alto
        trig.set_low().unwrap(); // Se reinicia

        let distancia_cm: Option<f32> = 'medicion: {
            let espera = Instant::now();
            while echo.is_low() { // Mientras el echo esta bajo
                if espera.elapsed().as_micros() > 5_000 { // Si tarda mas de 5 milisegundos en recibir el eco, se asume que no hay obstaculo
                    break 'medicion None; 
                }
            }
            let inicio = Instant::now(); // Si el pin se pone en alto, se inicia el conteo del tiempo que tarda en volver a ponerse en bajo
            while echo.is_high() {
                if inicio.elapsed().as_micros() > 30_000 { // Si tarda mas de 30 milisegundos en recibir el eco, se asume que no hay obstaculo
                    break 'medicion None;
                }
            }
            Some(inicio.elapsed().as_micros() as f32 / 58.0) // Se calcula la distancia en centimetros (tiempo en microsegundos / 58)
        };

        match distancia_cm {  // Se imprime en terminal la distancia medida
            Some(d) if d < 30.0 => log::info!("Obstaculo detectado a {:.1} cm", d),
            Some(d) => log::info!("Libre, objeto mas cercano a {:.1} cm", d),
            None => log::info!("Sin eco (no hay obstaculo o fuera de rango)"),
        }

        FreeRtos::delay_ms(100); // Pausa entre mediciones
    }
}