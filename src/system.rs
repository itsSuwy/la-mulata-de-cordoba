use esp_idf_svc::hal::gpio::{Input, InputPin, Output, OutputPin, PinDriver, Pull};
use esp_idf_svc::sys::EspError;
use std::time::Instant;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution, LedcChannel, LedcTimer, SpeedMode};
use esp_idf_svc::hal::units::FromValueType;
use esp_idf_svc::hal::delay::FreeRtos;

const PERIODO_US: u32 = 20_000; // 50 Hz = un pulso cada 20 ms
const MIN_US: u32 = 600;        // ancho de pulso para 0°
const MAX_US: u32 = 2_400;      // ancho de pulso para 180°

pub enum CanalRele { // Enum para identificar cada relevador
    Ac,
    Motor,
    Antorcha1,
    Antorcha2,
    FocoVerde,
}

pub enum CanalMoc { // Enum para identificar cada MOC
    Relampago,
    Estado,
}

#[derive(Clone, Copy)]
pub enum CanalServo {
    Servo1,
    Servo2,
}

pub struct Relevadores<'a> { // Arreglo de relevadores, cada uno con un pin asnignable
    rele_ac: PinDriver<'a, Output>,
    rele_motor: PinDriver<'a, Output>,
    rele_antorcha_1: PinDriver<'a, Output>,
    rele_antorcha_2: PinDriver<'a, Output>,
    rele_foco_verde: PinDriver<'a, Output>,
}

impl<'a> Relevadores<'a> { // Bloque logico de control sobre relevadores
    pub fn new( // Cada pin llega por separado, desde main se asigna el pin a un relevador especifico
        ac: impl OutputPin + 'a,
        motor: impl OutputPin + 'a,
        antorcha_1: impl OutputPin + 'a,
        antorcha_2: impl OutputPin + 'a,
        foco_verde: impl OutputPin + 'a,
    ) -> Result<Self, EspError> {
        Ok(Self {
            rele_ac: PinDriver::output(ac)?,
            rele_motor: PinDriver::output(motor)?,
            rele_antorcha_1: PinDriver::output(antorcha_1)?,
            rele_antorcha_2: PinDriver::output(antorcha_2)?,
            rele_foco_verde: PinDriver::output(foco_verde)?,
        })
    }

    pub fn encender(&mut self, canal: CanalRele) -> Result<(), EspError> { // Enciende el relevador especifico
        match canal {
            CanalRele::Ac => self.rele_ac.set_high()?,
            CanalRele::Motor => self.rele_motor.set_high()?,
            CanalRele::Antorcha1 => self.rele_antorcha_1.set_high()?,
            CanalRele::Antorcha2 => self.rele_antorcha_2.set_high()?,
            CanalRele::FocoVerde => self.rele_foco_verde.set_high()?,
        }
        Ok(())
    }
    pub fn apagar(&mut self, canal: CanalRele) -> Result<(), EspError> { // Apaga el relevador especifico
        match canal {
            CanalRele::Ac => self.rele_ac.set_low()?, // Por seguridad, cualquier fallo siempre activara este
            CanalRele::Motor => self.rele_motor.set_low()?,
            CanalRele::Antorcha1 => self.rele_antorcha_1.set_low()?,
            CanalRele::Antorcha2 => self.rele_antorcha_2.set_low()?,
            CanalRele::FocoVerde => self.rele_foco_verde.set_low()?,
        }
        Ok(())
    }
}

pub struct MOCs <'a>{
    moc_relampago: PinDriver<'a, Output>,
    moc_estado: PinDriver<'a, Output>, // Este indicara exito al establecerse la conexion del primer relevador a NA
}
impl <'a> MOCs<'a>{
    pub fn new( // Misma logica que relevadores, en Main se asigna el pin especifico
        relampago: impl OutputPin + 'a,
        estado: impl OutputPin + 'a,
    ) -> Result<Self, EspError> {
        Ok(Self{
            moc_relampago: PinDriver::output(relampago)?,
            moc_estado: PinDriver::output(estado)?,
        })
    }
    pub fn encender(&mut self, canal: CanalMoc) -> Result<(), EspError> { // Enciende el MOC especifico
        match canal {
            CanalMoc::Relampago => self.moc_relampago.set_high()?,
            CanalMoc::Estado => self.moc_estado.set_high()?,
        }
        Ok(())
    }
    pub fn apagar(&mut self, canal: CanalMoc) -> Result<(), EspError> { // Apaga el MOC especifico
        match canal {
            CanalMoc::Relampago => self.moc_relampago.set_low()?, 
            CanalMoc::Estado => self.moc_estado.set_low()?,
        }
        Ok(())
    }
}

pub struct Ultrasonico<'a> {
    echo: PinDriver<'a, Input>,
    trig: PinDriver<'a, Output>,
}
impl <'a> Ultrasonico<'a> {
    pub fn new(
        echo: impl InputPin + 'a,
        trig: impl OutputPin + 'a,
    ) -> Result<Self, EspError> {
        let mut echo_driver = PinDriver::input(echo, Pull::Down)?; // Se puede variar a Up o Floating, por gusto personal lo mantengo en Down
        let trig_driver = PinDriver::output(trig)?;
        Ok(Self{
            echo: echo_driver, 
            trig: trig_driver,
        })
    }
    pub fn medir_distancia(&mut self) -> Option<f32> { // Devuelve la distancia medida en centimetros
        self.trig.set_low().unwrap();
        Ets::delay_us(2);
        self.trig.set_high().unwrap();
        Ets::delay_us(10);
        self.trig.set_low().unwrap();

        let distancia_cm: Option<f32> = 'medicion: {
            let espera = Instant::now();
            while self.echo.is_low() {
                if espera.elapsed().as_micros() > 5_000 {
                    break 'medicion None;
                }
            }
            let inicio = Instant::now();
            while self.echo.is_high() {
                if inicio.elapsed().as_micros() > 30_000 {
                    break 'medicion None;
                }
            }
            Some(inicio.elapsed().as_micros() as f32 / 58.0)
        };
        distancia_cm
    }
}

pub struct Servomotor<'a, S: SpeedMode> {
    _timer: LedcTimerDriver<'a, S>,
    servo_1: LedcDriver<'a>,
    servo_2: LedcDriver<'a>,
}

impl<'a, S: SpeedMode> Servomotor<'a, S> {
    pub fn new<T, C1, P1, C2, P2>(
        timer_peripheral: T,
        channel_1_peripheral: C1,
        pin_1: P1,
        channel_2_peripheral: C2,
        pin_2: P2,
    ) -> Result<Self, EspError>
    where
        T: LedcTimer<SpeedMode = S> + 'a,
        C1: LedcChannel<SpeedMode = S> + 'a,
        C2: LedcChannel<SpeedMode = S> + 'a,
        P1: OutputPin + 'a,
        P2: OutputPin + 'a,
    {
        let timer = LedcTimerDriver::new(
            timer_peripheral,
            &TimerConfig::new()
                .frequency(50u32.Hz().into())
                .resolution(Resolution::Bits14),
        )?;

        let servo_1 =
            LedcDriver::new(channel_1_peripheral, &timer, pin_1)?;

        let servo_2 =
            LedcDriver::new(channel_2_peripheral, &timer, pin_2)?;

        let mut servos = Self {
            _timer: timer,
            servo_1,
            servo_2,
        };

        servos.fijar_angulo(CanalServo::Servo1, 10)?;
        servos.fijar_angulo(CanalServo::Servo2, 10)?;

        Ok(servos)
    }


    /// Fija de manera directa el ángulo (0 a 180) de un servo específico
    pub fn fijar_angulo(&mut self, canal: CanalServo, angulo: u32) -> Result<(), EspError> {
        let angulo_clamped = angulo.min(180); // Limitar a rango máximo de 180°
        let pulso_us = MIN_US + angulo_clamped * (MAX_US - MIN_US) / 180;
        let driver = match canal {
            CanalServo::Servo1 => &mut self.servo_1,
            CanalServo::Servo2 => &mut self.servo_2,
        };
        let duty = pulso_us * driver.get_max_duty() / PERIODO_US;
        driver.set_duty(duty)?;
        Ok(())
    }

    /// Mueve el servo grado por grado desde una posición inicial a una final con un delay por paso
    pub fn mover_suave(
        &mut self,
        canal: CanalServo,
        inicio: u32,
        fin: u32,
        pausa_ms: u32,
    ) -> Result<(), EspError> {
        if inicio < fin {
            for a in inicio..=fin {
                self.fijar_angulo(canal, a)?;
                FreeRtos::delay_ms(pausa_ms);
            }
        } else {
            for a in (fin..=inicio).rev() {
                self.fijar_angulo(canal, a)?;
                FreeRtos::delay_ms(pausa_ms);
            }
        }
        Ok(())
    }
}
