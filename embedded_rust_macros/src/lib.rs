mod config;
mod devices;
mod generation;
mod types;
// use embedded_rust::device::stm32f1xx::{Gpio, TriggerEdge};
use config::Config;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, parse_str, Expr, ExprUnsafe, ItemStruct, Stmt, Type};
use types::*;

#[proc_macro_attribute]
pub fn device_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_json(&attr);
    let strukt = parse_macro_input!(item as ItemStruct);
    let struct_name = strukt.ident.clone();
    let statiks = generate_component_statics(&config);
    let static_init = generate_static_init(&config);
    let heap_size = config.sys().heap_size();

    let init_stmts = config.init_statements();
    let interrupt_unmasks = config.interrupt_unmasks();
    quote!(
        #strukt
        impl #struct_name{
            #[inline]
            fn init(){
                #(#init_stmts)*
                #(#statiks)*
                #static_init
                unsafe{
                    Runtime::init(
                        #heap_size,
                        SYS_ARRAY.as_mut().unwrap(),
                        INPUT_ARRAY.as_mut().unwrap(),
                        OUTPUT_ARRAY.as_mut().unwrap(),
                        PWM_ARRAY.as_mut().unwrap(),
                        CHANNEL_ARRAY.as_mut().unwrap(),
                        SERIAL_ARRAY.as_mut().unwrap(),
                        TIMER_ARRAY.as_mut().unwrap(),
                    ).expect("Runtime initialization failed");
                }
            }
            #[inline]
            fn get_resource(
                uri: &str
            ) -> Result<embedded_rust::resources::ResourceID, embedded_rust::resources::ResourceError>{
                embedded_rust::Runtime::get().get_resource(uri)
            }
            #[inline]
            fn run() -> !{
                unsafe{
                    #(#interrupt_unmasks)*
                }
                embedded_rust::Runtime::get().run()
            }
        }
    )
    .into()
    // quote!().into()
}

fn generate_component_statics(config: &Config) -> Vec<Stmt> {
    let sys_tys: &Vec<Type> = &vec![];
    let in_tys = &config.input_tys();
    let out_tys = &config.output_tys();
    let pwm_tys = &config.pwm_tys();
    let chan_tys: &Vec<Type> = &vec![];
    let ser_tys: &Vec<Type> = &vec![];
    let tim_tys: &Vec<Type> = &vec![];

    let sys_len = 0usize;
    let in_len = config.input_idents().len();
    let out_len = config.output_idents().len();
    let pwm_len = config.pwm_idents().len();
    let chan_len = 0usize;
    let ser_len = 0usize;
    let tim_len = 0usize;

    let statics: Vec<Stmt> = parse_quote!(
        // Tuple witch concrete objects
        static mut SYS: Option<(#(#sys_tys,)*)> = None;
        static mut INPUT_PINS: Option<(#(InputPin<#in_tys>,)*)> = None;
        static mut OUTPUT_PINS: Option<(#(OutputPin<#out_tys>,)*)> = None;
        static mut PWM_PINS: Option<(#(PWMPin<#pwm_tys>,)*)> = None;
        static mut CHANNELS: Option<(#(#chan_tys,)*)> = None;
        static mut SERIALS: Option<(#(#ser_tys,)*)> = None;
        static mut TIMERS: Option<(#(#tim_tys,)*)> = None;

        // Arrays with pointers to concrete objects
        static mut SYS_ARRAY: Option<[&'static mut dyn Resource;#sys_len]> = None;
        static mut INPUT_ARRAY: Option<[&'static mut dyn Resource; #in_len]> = None;
        static mut OUTPUT_ARRAY: Option<[&'static mut dyn Resource; #out_len]> = None;
        static mut PWM_ARRAY: Option<[&'static mut dyn Resource; #pwm_len]> = None;
        static mut CHANNEL_ARRAY: Option<[&'static mut dyn Resource; #chan_len]> = None;
        static mut SERIAL_ARRAY: Option<[&'static mut dyn Resource; #ser_len]> = None;
        static mut TIMER_ARRAY: Option<[&'static mut dyn Resource; #tim_len]> = None;
    );
    statics.into()
}

fn generate_static_init(config: &Config) -> ExprUnsafe {
    let sys_idents: &Vec<Type> = &vec![];
    let in_idents = &config.input_idents();
    let out_idents = &config.output_idents();
    let pwm_idents = &config.pwm_idents();
    let channel_idents: &Vec<Type> = &vec![];
    let serial_idents: &Vec<Type> = &vec![];
    let timer_idents: &Vec<Type> = &vec![];

    let sys_index = (0..0usize).map(syn::Index::from);
    let in_index = (0..config.input_idents().len()).map(syn::Index::from);
    let out_index = (0..config.output_idents().len()).map(syn::Index::from);
    let pwm_index = (0..config.pwm_idents().len()).map(syn::Index::from);
    let chan_index = (0..0usize).map(syn::Index::from);
    let ser_index = (0..0usize).map(syn::Index::from);
    let tim_index = (0..0usize).map(syn::Index::from);

    let in_channels = &config.input_channels();
    let in_ports = &config.input_ports();

    let out_channels = &config.output_channels();
    let out_ports = &config.output_ports();

    let pwm_channels = &config.pwm_channels();
    let pwm_ports = &config.pwm_ports();

    parse_quote!(
        unsafe{
            //TODO: populate sys
           SYS = Some((#(#sys_idents,)*));
           INPUT_PINS = Some((#(InputPin::new(Pin::new(#in_channels , #in_ports), #in_idents),)*));
           OUTPUT_PINS = Some((#(OutputPin::new(Pin::new(#out_channels, #out_ports), #out_idents),)*));
           PWM_PINS = Some((#(PWMPin::new(Pin::new(#pwm_channels, #pwm_ports), #pwm_idents),)*));
           CHANNELS = Some((#(#channel_idents,)*));
           SERIALS = Some((#(#serial_idents,)*));
           TIMERS = Some((#(#timer_idents,)*));

            let sys = SYS.as_mut().unwrap();
            let input_pins = INPUT_PINS.as_mut().unwrap();
            let output_pins = OUTPUT_PINS.as_mut().unwrap();
            let pwm = PWM_PINS.as_mut().unwrap();
            let channels = CHANNELS.as_mut().unwrap();
            let serials = SERIALS.as_mut().unwrap();
            let timers = TIMERS.as_mut().unwrap();

            SYS_ARRAY = Some([#(&mut sys.#sys_index,)*]);
            INPUT_ARRAY = Some([#(&mut input_pins.#in_index,)*]);
            OUTPUT_ARRAY = Some([#(&mut output_pins.#out_index,)*]);
            PWM_ARRAY = Some([#(&mut pwm.#pwm_index,)*]);
            CHANNEL_ARRAY = Some([#(&mut channels.#chan_index,)*]);
            SERIAL_ARRAY = Some([#(&mut serials.#ser_index,)*]);
            TIMER_ARRAY = Some([#(&mut timers.#tim_index,)*]);
        }
    )
}

// TODO: integrate
fn generate_sys_objects(config: &Config) -> Vec<Expr> {
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

pub(crate) fn parse_json(attributes: &TokenStream) -> Config {
    serde_json::from_str(&attributes.to_string()).expect("ParsingError")
}
