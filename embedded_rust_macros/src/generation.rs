use crate::{config::Config, types::Serial, Frequency, Gpio, PWMInterface};
use syn::{parse_quote, parse_str, Expr, ExprUnsafe, Ident, Stmt, Type};

/// The Generator trait is used to determine the proper generation functions
/// It is just a meta trait that combines all special generation traits.
pub trait Generator:
    DeviceGeneration + GpioGeneration + SysGeneration + PWMGeneration + SerialGeneration
{
}
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
    fn generate_channels(&self, gpios: &Vec<Box<dyn Gpio>>) -> Vec<Stmt>;
}
pub trait GpioGeneration {
    /// In this function all pins should be introduced with a let binding.
    /// The identifiers for the pins should be generatet with the identifier
    /// function of the Gpio trait (or rather its Component trait bound).
    /// The identifiers will later be used to populate the global data statics.
    ///
    /// All other gpio dependent initializations (like gpio interrupts) should go
    /// here as well.
    fn generate_gpios(&self, gpios: &Vec<Box<dyn Gpio>>) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        for gpio in gpios {
            stmts.append(&mut gpio.generate());
        }
        stmts
    }
    /// This function should return all gpio interrupts that should be enabled.
    /// For the Stm32f1 boards this would be the apropriate Exti_X (External
    /// Interrupt) lines
    fn interrupts(&self, gpios: &Vec<Box<dyn Gpio>>) -> Vec<Stmt>;
}
pub trait PWMGeneration {
    fn generate_pwm_pins(&self, pwms: &Vec<&dyn PWMInterface>) -> Vec<Stmt> {
        let mut stmts = vec![];
        for pwm in pwms {
            stmts.append(&mut pwm.generate());
        }
        stmts
    }
}
pub trait SerialGeneration {
    fn generate_serials(&self, serials: &Vec<&dyn Serial>) -> Vec<Stmt> {
        let mut stmts = vec![];
        for serial in serials {
            stmts.append(&mut serial.generate())
        }
        stmts
    }
}
pub trait SysGeneration {
    /// With this function statements for board speed are generated
    /// These statements go right after the device init statements
    fn generate_clock(&self, sys_frequency: &Option<Frequency>) -> Vec<Stmt>;
}

//TODO: shorten
macro_rules! define_static {
    ($name:expr, (), $types:expr) => {{
        let tys: &Vec<Type> = $types;
        let len: usize = tys.len();
        let ident = quote::format_ident!("{}", $name);
        let array_ident = quote::format_ident!("{}_ARRAY", $name);
        let src: Vec<Stmt> = syn::parse_quote!(
            static mut #ident: Option<(#(#tys,)*)> = None;
            static mut #array_ident: Option<[&'static mut dyn Resource; #len]> = None;
        );
        src
    }};
    ($name:expr, $resourceType:expr, $types:expr) => {{
        let tys: &Vec<Type> = $types;
        let len: usize = tys.len();
        let resource_type = quote::format_ident!("{}", $resourceType);
        let ident = quote::format_ident!("{}", $name);
        let array_ident = quote::format_ident!("{}_ARRAY", $name);
        let src: Vec<Stmt> = syn::parse_quote!(
            static mut #ident: Option<(#(#resource_type<#tys>,)*)> = None;
            static mut #array_ident: Option<[&'static mut dyn Resource; #len]> = None;
        );
        src
    }};
    ($name:expr, $serialType:expr, $word_tys:expr, $types:expr) => {{
        let tys: &Vec<Type> = $types;
        let len: usize = tys.len();
        let resource_type = quote::format_ident!("{}", $serialType);
        let word_tys: &Vec<Type> = $word_tys;
        let ident = quote::format_ident!("{}", $name);
        let array_ident = quote::format_ident!("{}_ARRAY", $name);
        let src: Vec<Stmt> = syn::parse_quote!(
            static mut #ident: Option<(#(#resource_type<#tys, #word_tys>,)*)> = None;
            static mut #array_ident: Option<[&'static mut dyn Resource; #len]> = None;
        );
        src
    }};
}

pub(crate) fn component_statics(config: &Config) -> Vec<Stmt> {
    let mut stmts = vec![];
    stmts.append(&mut define_static!("SYS", (), &vec![]));
    stmts.append(&mut define_static!(
        "INPUT_PINS",
        "InputPin",
        &config.input_tys()
    ));
    stmts.append(&mut define_static!(
        "OUTPUT_PINS",
        "OutputPin",
        &config.output_tys()
    ));
    stmts.append(&mut define_static!("PWM_PINS", "PWMPin", &config.pwm_tys()));
    stmts.append(&mut define_static!("CHANNELS", (), &vec![]));
    stmts.append(&mut define_static!(
        "SERIALS",
        "Serial",
        &config.serial_word_tys(),
        &config.serial_tys()
    ));
    stmts.append(&mut define_static!("TIMERS", (), &vec![]));
    stmts.into()
}

macro_rules! init_static {
    ($name:expr, $idents:expr, $constructors:expr) => {{
        let name: &str = $name;
        let idents: &Vec<Ident> = $idents;
        let constructors: &Vec<Expr> = $constructors;
        let index = (0..idents.len()).map(syn::Index::from);
        let ident_upper = quote::format_ident!("{}", name.to_uppercase());
        let ident_lower = quote::format_ident!("{}", name.to_lowercase());
        let array_ident = quote::format_ident!("{}_ARRAY", name.to_uppercase());
        let src: Vec<Stmt> = syn::parse_quote!(
            #ident_upper = Some((#(#constructors,)*));
            let #ident_lower = #ident_upper.as_mut().unwrap();
            #array_ident = Some([#(&mut #ident_lower.#index,)*]);
        );
        src
    }};
}
pub(crate) fn static_init(config: &Config) -> ExprUnsafe {
    let mut inits: Vec<Stmt> = vec![];
    inits.append(&mut init_static!("SYS", &vec![], &vec![]));
    inits.append(&mut init_static!(
        "INPUT_PINS",
        &config.input_idents(),
        &config.input_constructors()
    ));
    inits.append(&mut init_static!(
        "OUTPUT_PINS",
        &config.output_idents(),
        &config.output_constructors()
    ));
    inits.append(&mut init_static!(
        "PWM_PINS",
        &config.pwm_idents(),
        &config.pwm_constructors()
    ));
    inits.append(&mut init_static!("CHANNELS", &vec![], &vec![]));
    inits.append(&mut init_static!(
        "SERIALS",
        &config.serial_idents(),
        &config.serial_constructors()
    ));
    inits.append(&mut init_static!("TIMERS", &vec![], &vec![]));

    parse_quote!(
        unsafe{
           #(#inits)*
        }
    )
}

// TODO: integrate
fn sys_objects(config: &Config) -> Vec<Expr> {
    let heapsize = config.sys().heap_size();
    let sys_clock = config.sys().sys_clock();
    let heap_obj = parse_str(&format!("Heap::new({})", heapsize)).unwrap();
    if let Some(sys_clock) = sys_clock {
        let sys_clock_obj = parse_str(&format!("SysClock::new({})", sys_clock)).unwrap();
        vec![heap_obj, sys_clock_obj]
    } else {
        vec![heap_obj]
    }
}
