use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType; // Permite escribir 50u32.Hz()

const PERIODO_US: u32 = 20_000; // 50 Hz = un pulso cada 20 ms
const MIN_US: u32 = 600;        // ancho de pulso para 0°
const MAX_US: u32 = 2_400;      // ancho de pulso para 180°

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Un solo temporizador a 50 Hz que comparten los dos servos
    let timer = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::new()
            .frequency(50u32.Hz().into())
            .resolution(Resolution::Bits14),
    ).unwrap();

    // Un canal por servo (señal en GPIO21 y GPIO47)
    let mut servo1 = LedcDriver::new(peripherals.ledc.channel0, &timer, pins.gpio21).unwrap();
    //let mut servo2 = LedcDriver::new(peripherals.ledc.channel1, &timer, pins.gpio47).unwrap();

    // Posición inicial segura, uno a la vez
    set_angulo(&mut servo1, 10);
    FreeRtos::delay_ms(1_000);
    //set_angulo(&mut servo2, 10);
    //FreeRtos::delay_ms(1_000);

    loop {
        log::info!("Servo 1");
        mover(&mut servo1, 30, 90, 20);
        FreeRtos::delay_ms(500);
        mover(&mut servo1, 90, 30, 20);
        FreeRtos::delay_ms(500);

        //log::info!("Servo 2");
        //mover(&mut servo2, 10, 170, 20);
        //FreeRtos::delay_ms(500);
        //mover(&mut servo2, 170, 10, 20);
        //FreeRtos::delay_ms(500);
    }
}

// Convierte un ángulo (0 a 180) en el valor de duty que espera el LEDC
fn set_angulo(servo: &mut LedcDriver<'_>, angulo: u32) {
    let pulso_us = MIN_US + angulo * (MAX_US - MIN_US) / 180;
    let duty = pulso_us * servo.get_max_duty() / PERIODO_US;
    servo.set_duty(duty).unwrap();
    log::info!("Angulo ordenado: {angulo}°");
}

// Mueve el servo grado por grado, con una pausa entre cada paso
fn mover(servo: &mut LedcDriver<'_>, desde: u32, hasta: u32, pausa_ms: u32) {
    if desde < hasta {
        for a in desde..=hasta {
            set_angulo(servo, a);
            FreeRtos::delay_ms(pausa_ms);
        }
    } else {
        for a in (hasta..=desde).rev() {
            set_angulo(servo, a);
            FreeRtos::delay_ms(pausa_ms);
        }
    }
}