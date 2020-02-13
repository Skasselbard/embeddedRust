use core::marker::PhantomData;
use cortex_m_semihosting::hprintln;
use embedded_hal::{
    adc::{Channel, OneShot},
    PwmPin,
};

use nb::Result;

pub struct Potentiometer<ADC, ADC_NUM, VALUE, ADC_CHANNEL> {
    pin: ADC_CHANNEL,
    max_value: VALUE,
    adc: PhantomData<ADC>,
    adc_num: PhantomData<ADC_NUM>,
}

impl<ADC, ADC_NUM, VALUE, ADC_CHANNEL> Potentiometer<ADC, ADC_NUM, VALUE, ADC_CHANNEL>
where
    ADC_CHANNEL: Channel<ADC_NUM>,
    ADC: OneShot<ADC_NUM, VALUE, ADC_CHANNEL>,
    VALUE: From<u16> + Into<u16> + Copy,
{
    pub fn new(pin: ADC_CHANNEL, max_value: VALUE) -> Self {
        Self {
            pin,
            max_value,
            adc: PhantomData,
            adc_num: PhantomData,
        }
    }

    pub fn read_percentage(&mut self, adc: &mut ADC) -> Result<f32, ADC::Error> {
        let intensity: u16 = adc.read(&mut self.pin)?.into();
        let max_value: u16 = self.max_value.into();
        Ok(intensity as f32 / max_value as f32)
    }
}
