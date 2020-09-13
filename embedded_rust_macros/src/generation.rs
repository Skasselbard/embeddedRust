use crate::{Frequency, Gpio, PWMInterface};
use syn::{Stmt, Type};

/// The Generator trait is used to determine the proper generation functions
/// It is just a meta trait that combines all special generation traits.
pub trait Generator: DeviceGeneration + GpioGeneration + SysGeneration {}

pub trait DeviceGeneration {
    /// Everything that should be used in the device init function with
    /// a ``use crate::pa::th`` statement.
    fn generate_imports(&self) -> Vec<Stmt>;
    /// Here you can add functions to prepare the general device
    /// and introduce variable names for later use
    /// For example the stm32f1xx boards need acces to a peripheral
    /// singleton and initialized flash.
    fn generate_device_init(&self) -> Vec<Stmt>;
    /// In the stm32f1 hal, each pin channel ('A' to 'E' in the pin types PAX to PEX)
    /// has to be initialized to initialize the actual pins
    /// this is done with these statements.
    /// A function to get the channel name is included in the Pin trait.
    /// A function to get the pin is included in the Gpio trait.
    fn generate_channels(&self, gpios: &Vec<&dyn Gpio>) -> Vec<Stmt>;
}
pub trait GpioGeneration {
    /// In this function all pins should be introduced with a let binding.
    /// The identifiers for the pins should be generatet with the identifier
    /// function of the Gpio trait (or rather its Component trait bound).
    /// The identifiers will later be used to populate the global data statics.
    ///
    /// All other gpio dependent initializations (like gpio interrupts) should go
    /// here as well.
    fn generate_gpios(&self, gpios: &Vec<&dyn Gpio>) -> Vec<Stmt>;
    /// This function should return all gpio interrupts that should be enabled.
    /// For the Stm32f1 boards this would be the apropriate Exti_X (External
    /// Interrupt) lines
    fn interrupts(&self, gpios: &Vec<&dyn Gpio>) -> Vec<Stmt>;
}
pub trait PWMGeneration {
    fn generate_pwm_pins(&self, pwms: &Vec<&dyn PWMInterface>) -> Vec<Stmt>{
        let mut stmts = vec![];
        for pwm in pwms{
            stmts.append(&mut pwm.generate());
        }
        stmts
    }
}
pub trait SysGeneration {
    /// With this function statements for board speed are generated
    /// These statements go right after the device init statements
    fn generate_clock(&self, sys_frequency: &Option<Frequency>) -> Vec<Stmt>;
}
