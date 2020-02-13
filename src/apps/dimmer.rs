use super::Potentiometer;
use core::marker::PhantomData;
use embedded_hal::{
    adc::{Channel, OneShot},
    PwmPin,
};
use nb::Result;

pub struct App<PWM_PIN, ADC, ADC_NUM, ADC_VALUE, ADC_CHANNEL> {
    pwm: PWM_PIN,
    poti: Potentiometer<ADC, ADC_NUM, ADC_VALUE, ADC_CHANNEL>,
    adc: ADC,
    _marker: PhantomData<ADC_NUM>,
}

impl<PWM_PIN, ADC, ADC_NUM, ADC_VALUE, ADC_CHANNEL, DUTY>
    App<PWM_PIN, ADC, ADC_NUM, ADC_VALUE, ADC_CHANNEL>
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
            _marker: PhantomData,
        }
    }

    fn set_pwm(&mut self, percentage: f32) {
        let max: u16 = self.pwm.get_max_duty().into();
        let duty: u16 = (max as f32 * percentage) as u16;
        self.pwm.set_duty(duty.into());
    }

    fn task(&mut self) -> Result<(), ADC::Error> {
        let intensity = self.poti.read_percentage(&mut self.adc)?;
        self.set_pwm(intensity);
        Ok(())
    }

    pub fn run(&mut self) -> ! {
        self.pwm.enable();
        loop {
            match self.task() {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
}
