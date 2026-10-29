use esp_idf_svc::hal::peripherals::{Peripherals};
// use std::sync::{mpsc, Arc, Mutex};
use esp_idf_svc::hal::delay::FreeRtos; 

mod system; // Abstraccion y comunicacion del hardware hacia la ESP32-S3
use system::{Relevadores, Mocs, Servomotor, Ultrasonico, ControladorPca, SensoresHall}; // Structs implementados
use system::{CanalMoc,CanalRele, HallSensor}; // Enunms para seleccionar el componente especifico


mod idle; // Control del animatronico mientras no haya nadie observando
use idle::Barrido; 

enum Estado{Idle, Active,}

const ESPERA_MS: u32 = 3_000;
const UMBRAL_CM: f32 = 80.0; 
const LECTURAS_SEGUIDAS: u8 = 3;

fn main() {
    esp_idf_svc::sys::link_patches();
    //esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Inicializacion de variables dentro de structs
    let mut relevadores = Relevadores::new(pins.gpio1, pins.gpio2, pins.gpio3, pins.gpio4, pins.gpio5).unwrap();
    let mut mocs = Mocs::new(pins.gpio6, pins.gpio7).unwrap();
    let mut ultrasonico = Ultrasonico::new(pins.gpio8, pins.gpio9).unwrap();
    let mut servomotores = Servomotor::new(
    peripherals.ledc.timer0,
    peripherals.ledc.channel0, pins.gpio10,
    peripherals.ledc.channel1, pins.gpio11,)
    .unwrap();    

    /*let pca = ControladorPca::new(
        peripherals.i2c0, 
        pins.gpio12, 
        pins.gpio13)
        .unwrap();
    let pca = Arc::new(Mutex::new(pca)); // El PCA ahora vive dentro del Arc
    let pca_independent = Arc::clone(&pca); // Nna copia del acceso para el hilo
    let pca_active = Arc::clone(&pca); // Otra para Active
    let (tx_rayo, rx_rayo) = mpsc::channel::<()>();
    */

    //let mut sensores_hall = SensoresHall::new(pins.gpio14, pins.gpio15).unwrap();
    let mut barrido = Barrido::new();
    let mut estado = Estado::Idle; // Por defecto empieza en idle la Mulata
    let mut seguidas: u8 = 0;

    // Activamos la parte ac
    relevadores.encender(CanalRele::Ac).unwrap(); // Establece la conexion general de corriente alterna
    mocs.encender(CanalMoc::Estado).unwrap(); // Se deberia encender un foquito verde indicando que el Relevador de AC general esta conduciendo energia

    relevadores.encender(CanalRele::Antorcha1).unwrap();
    relevadores.encender(CanalRele::Antorcha2).unwrap();

    loop{
        match estado {
        Estado::Idle => {
            barrido.paso(&mut servomotores).unwrap();

            match ultrasonico.medir_distancia() {
                Some(cm) if cm < UMBRAL_CM => seguidas += 1,
                _ => seguidas = 0, // Se filtra el ruido  
            }
            if seguidas >= LECTURAS_SEGUIDAS { // Hay una persona observando
                seguidas = 0; // Se reinicia el contador 
                barrido.volver_a_origen(&mut servomotores).unwrap(); // El servo de idle vuelve a su origen
                FreeRtos::delay_ms(ESPERA_MS); // Espera unos segundos para la transicion
                estado = Estado::Active; // Active toma el control
            }
        }
        Estado::Active => {
            // TODO: aquí va la secuencia, usando CanalServo::Servo2
            estado = Estado::Idle;
        }
    }
    }
}
