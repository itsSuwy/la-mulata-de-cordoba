use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::SpeedMode;
use esp_idf_svc::sys::EspError;

use crate::system::{CanalServo, Servomotor};

const PAUSA_MS: u32 = 20;         // Pausa por grado durante el barrido
const PAUSA_REGRESO_MS: u32 = 15; // Pausa por grado al regresar a la posición original

// Recorrido del servo como tramos (inicio, fin): 90 -> 180 -> 90 -> 0 -> 90
const TRAMOS: [(u32, u32); 4] = [(90, 180), (180, 90), (90, 0), (0, 90)];
const ORIGEN: u32 = 90; // Posición original del servo de Idle

/// Idle controla el Servo1 durante todo el programa. No sabe de sensores ni de Active.
pub struct Barrido {
    tramo: usize,
    angulo: u32, // Siguiente ángulo a escribir
    actual: u32, // Último ángulo escrito (posición real del servo)
}

impl Barrido {
    pub fn new() -> Self {
        Self { tramo: 0, angulo: TRAMOS[0].0, actual: ORIGEN }
    }

    /// Avanza un grado y regresa. main decide cuándo volver a llamarlo.
    pub fn paso<S: SpeedMode>(&mut self, servos: &mut Servomotor<'_, S>) -> Result<(), EspError> {
        servos.fijar_angulo(CanalServo::Servo2, self.angulo)?;
        self.actual = self.angulo;
        FreeRtos::delay_ms(PAUSA_MS);

        let (inicio, fin) = TRAMOS[self.tramo];
        if self.angulo == fin {
            // Pasa al siguiente tramo, empezando un grado después del vértice
            self.tramo = (self.tramo + 1) % TRAMOS.len();
            let (i, f) = TRAMOS[self.tramo];
            self.angulo = if i < f { i + 1 } else { i - 1 };
        } else {
            self.angulo = if inicio < fin { self.angulo + 1 } else { self.angulo - 1 };
        }
        Ok(())
    }

    /// Regresa el servo suavemente a su posición original y reinicia el barrido
    pub fn volver_a_origen<S: SpeedMode>(
        &mut self,
        servos: &mut Servomotor<'_, S>,
    ) -> Result<(), EspError> {
        servos.mover_suave(CanalServo::Servo2, self.actual, ORIGEN, PAUSA_REGRESO_MS)?;
        *self = Self::new();
        Ok(())
    }
}