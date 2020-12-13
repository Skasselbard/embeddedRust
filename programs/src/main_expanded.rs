#![feature(prelude_import)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::v1::*;
#[macro_use]
extern crate core;
extern crate alloc;
use cortex_m_rt::entry;
use embedded_rust::io::{AsyncReadExt, AsyncWriteExt};
use embedded_rust::Task;
use embedded_rust_macros::*;
struct BluePill;
impl BluePill {
    #[inline]
    fn init() {
        use stm32f1xx_hal::prelude::*;
        use stm32f1xx_hal::gpio::{self, Edge, ExtiPin};
        use stm32f1xx_hal::timer::{self, Timer};
        use stm32f1xx_hal::pwm::{self, PwmChannel};
        use stm32f1xx_hal::pac;
        use stm32f1xx_hal::serial::{self, Config};
        use embedded_rust::resources::{Resource, Pin, InputPin, OutputPin, PWMPin, Serial};
        use embedded_rust::device::{Port, Channel, SerialID};
        use embedded_rust::Runtime;
        let peripherals = stm32f1xx_hal::pac::Peripherals::take().unwrap();
        let mut flash = peripherals.FLASH.constrain();
        let mut rcc = peripherals.RCC.constrain();
        let cfgr = rcc.cfgr;
        let cfgr = cfgr.sysclk(36000000u32.hz());
        let clocks = cfgr.freeze(&mut flash.acr);
        let mut afio = peripherals.AFIO.constrain(&mut rcc.apb2);
        let mut gpioa = peripherals.GPIOA.split(&mut rcc.apb2);
        let mut gpioc = peripherals.GPIOC.split(&mut rcc.apb2);
        let mut gpiob = peripherals.GPIOB.split(&mut rcc.apb2);
        let mut pa0 = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);
        pa0.make_interrupt_source(&mut afio);
        pa0.trigger_on_edge(&peripherals.EXTI, Edge::FALLING);
        pa0.enable_interrupt(&peripherals.EXTI);
        let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        let mut pa1 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
        let mut pb7 = gpiob.pb7.into_floating_input(&mut gpiob.crl);
        let mut pb6 = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let timer = Timer::tim2(peripherals.TIM2, &clocks, &mut rcc.apb1);
        let (pa1) = timer
            .pwm::<timer::Tim2NoRemap, _, _, _>((pa1), &mut afio.mapr, 10000u32.hz())
            .split();
        let mut usart1 = serial::Serial::usart1(
            peripherals.USART1,
            (pb6, pb7),
            &mut afio.mapr,
            Config::default().baudrate(9600u32.bps()),
            clocks,
            &mut rcc.apb2,
        );
        let (mut usart1_tx, mut usart1_rx) = usart1.split();
        usart1_rx.listen();
        static mut SYS: Option<()> = None;
        static mut SYS_ARRAY: Option<[&'static mut dyn Resource; 0usize]> = None;
        static mut INPUT_PINS: Option<(InputPin<gpio::gpioa::PA0<gpio::Input<gpio::PullUp>>>,)> =
            None;
        static mut INPUT_PINS_ARRAY: Option<[&'static mut dyn Resource; 1usize]> = None;
        static mut OUTPUT_PINS: Option<(
            OutputPin<gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>>,
        )> = None;
        static mut OUTPUT_PINS_ARRAY: Option<[&'static mut dyn Resource; 1usize]> = None;
        static mut PWM_PINS: Option<(PWMPin<PwmChannel<pac::TIM2, pwm::C2>>,)> = None;
        static mut PWM_PINS_ARRAY: Option<[&'static mut dyn Resource; 1usize]> = None;
        static mut CHANNELS: Option<()> = None;
        static mut CHANNELS_ARRAY: Option<[&'static mut dyn Resource; 0usize]> = None;
        static mut SERIALS: Option<(Serial<serial::Tx<pac::USART1>, serial::Rx<pac::USART1>>,)> =
            None;
        static mut SERIALS_ARRAY: Option<[&'static mut dyn Resource; 1usize]> = None;
        static mut TIMERS: Option<()> = None;
        static mut TIMERS_ARRAY: Option<[&'static mut dyn Resource; 0usize]> = None;
        unsafe {
            SYS = Some(());
            let sys = SYS.as_mut().unwrap();
            SYS_ARRAY = Some([]);
            INPUT_PINS = Some((InputPin::new(Pin::new(Channel::A, Port::P00), pa0),));
            let input_pins = INPUT_PINS.as_mut().unwrap();
            INPUT_PINS_ARRAY = Some([&mut input_pins.0]);
            OUTPUT_PINS = Some((OutputPin::new(Pin::new(Channel::C, Port::P13), pc13),));
            let output_pins = OUTPUT_PINS.as_mut().unwrap();
            OUTPUT_PINS_ARRAY = Some([&mut output_pins.0]);
            PWM_PINS = Some((PWMPin::new(Pin::new(Channel::A, Port::P01), pa1),));
            let pwm_pins = PWM_PINS.as_mut().unwrap();
            PWM_PINS_ARRAY = Some([&mut pwm_pins.0]);
            CHANNELS = Some(());
            let channels = CHANNELS.as_mut().unwrap();
            CHANNELS_ARRAY = Some([]);
            SERIALS = Some((Serial::new(SerialID::Usart1, usart1_tx, usart1_rx),));
            let serials = SERIALS.as_mut().unwrap();
            SERIALS_ARRAY = Some([&mut serials.0]);
            TIMERS = Some(());
            let timers = TIMERS.as_mut().unwrap();
            TIMERS_ARRAY = Some([]);
            SERIALS.as_mut().unwrap().0.init();
        }
        unsafe {
            Runtime::init(
                10240usize,
                SYS_ARRAY.as_mut().unwrap(),
                INPUT_PINS_ARRAY.as_mut().unwrap(),
                OUTPUT_PINS_ARRAY.as_mut().unwrap(),
                PWM_PINS_ARRAY.as_mut().unwrap(),
                CHANNELS_ARRAY.as_mut().unwrap(),
                SERIALS_ARRAY.as_mut().unwrap(),
                TIMERS_ARRAY.as_mut().unwrap(),
            )
            .expect("Runtime initialization failed");
        }
    }
    #[inline]
    fn get_resource(
        uri: &str,
    ) -> Result<embedded_rust::resources::ResourceID, embedded_rust::resources::ResourceError> {
        embedded_rust::Runtime::get().get_resource(uri)
    }
    #[inline]
    fn run() -> ! {
        unsafe {
            stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::EXTI0);
            stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::USART1);
        }
        embedded_rust::Runtime::get().run()
    }
}
#[doc(hidden)]
#[export_name = "main"]
pub unsafe extern "C" fn __cortex_m_rt_main_trampoline() {
    __cortex_m_rt_main()
}
fn __cortex_m_rt_main() -> ! {
    BluePill::init();
    Task::new(test_task()).spawn();
    BluePill::run();
}
enum Level {
    Full,
    High,
    Half,
    Low,
    Off,
}
struct Brightness {
    level: Level,
}
impl Brightness {
    fn next(&mut self) -> f32 {
        match self.level {
            Level::Full => {
                self.level = Level::High;
                0.75f32
            }
            Level::High => {
                self.level = Level::Half;
                0.5f32
            }
            Level::Half => {
                self.level = Level::Low;
                0.25f32
            }
            Level::Low => {
                self.level = Level::Off;
                0.0f32
            }
            Level::Off => {
                self.level = Level::Full;
                1.0f32
            }
        }
    }
}
pub async fn test_task() {
    let mut button_events = BluePill::get_resource("event:gpio/pa0").unwrap();
    let mut led = BluePill::get_resource("digital:gpio/pc13").unwrap();
    let mut brightness = Brightness { level: Level::Off };
    let mut pwm = BluePill::get_resource("percent:pwm/pa1").unwrap();
    let mut usart1 = BluePill::get_resource("bus:serial/usart1").unwrap();
    pwm.write(&if false {
        brightness.next().to_be_bytes()
    } else {
        brightness.next().to_le_bytes()
    })
    .await
    .unwrap();
    let mut led_state = false;
    let mut buf = [0; 10];
    loop {
        usart1.write("ABCDEFGHIJ".as_bytes()).await.unwrap();
        usart1.read(&mut buf).await.unwrap();
    }
}
