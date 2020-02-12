use core::marker::PhantomData;
use cortex_m_semihosting::hprintln;
// use embedded_hal::{
//     adc::{Channel, OneShot},
//     PwmPin,
// };
use embedded_hal::adc::Channel;
use stm32f1xx_hal::prelude::_embedded_hal_PwmPin as PwmPin;
use stm32f1xx_hal::prelude::_embedded_hal_adc_OneShot as OneShot;

use nb::Result;

pub struct Potentiometer<ADC, ADC_NUM, VALUE, ADC_CHANNEL> {
    pin: ADC_CHANNEL,
    max_value: VALUE,
    adc: PhantomData<ADC>,
    adc_num: PhantomData<ADC_NUM>
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
            adc_num: PhantomData
        }
    }

    pub fn read(&mut self, adc: &mut ADC) -> Result<f32, ADC::Error> {
        let intensity: u16 = adc.read(&mut self.pin)?.into();
        let max_value: u16 = self.max_value.into();
        Ok(intensity as f32 / max_value as f32)
    }
}

pub struct Prog<PWM_PIN, ADC, ADC_NUM, ADC_VALUE, ADC_CHANNEL> {
    pwm: PWM_PIN,
    poti: Potentiometer<ADC, ADC_NUM, ADC_VALUE, ADC_CHANNEL>,
    adc: ADC,
    _marker: PhantomData<ADC_NUM>
}

impl<PWM_PIN, ADC, ADC_NUM, ADC_VALUE, ADC_CHANNEL, DUTY>
    Prog<PWM_PIN, ADC, ADC_NUM, ADC_VALUE, ADC_CHANNEL>
where
    PWM_PIN: PwmPin<Duty = DUTY>,
    ADC_CHANNEL: Channel<ADC_NUM>,
    ADC: OneShot<ADC_NUM, ADC_VALUE, ADC_CHANNEL>,
    DUTY: core::ops::Mul + From<u16> + Into<u16>,
    ADC_VALUE: From<u16> + Into<u16> + Copy,
{
    pub fn init(
        pwm_pin: PWM_PIN,
        adc: ADC,
        adc_max_sample: ADC_VALUE,
        potentiometer_pin: ADC_CHANNEL,
    ) -> Self {
        let poti = Potentiometer::new(potentiometer_pin, adc_max_sample);
        Self {
            pwm: pwm_pin,
            poti,
            adc: adc,
            _marker: PhantomData
        }
    }

    fn set_pwm(&mut self, percentage: f32) {
        let max: u16 = self.pwm.get_max_duty().into();
        let duty: u16 = (max as f32 * percentage) as u16;
        self.pwm.set_duty(duty.into());
    }

    fn task(&mut self) -> Result<(), ADC::Error> {
        let intensity = self.poti.read(&mut self.adc)?;
        self.set_pwm(intensity);
        hprintln!("{}", intensity).unwrap();
        Ok(())
    }

    pub fn run(&mut self) -> ! {
        self.pwm.enable();
        loop {
            match self.task() {
                Ok(_) => {}
                Err(_) => hprintln!("Error").unwrap(),
            }
        }
    }
}
