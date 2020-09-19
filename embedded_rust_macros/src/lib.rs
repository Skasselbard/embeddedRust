mod components;
mod devices;
mod generation;
mod types;
// use embedded_rust::device::stm32f1xx::{Gpio, TriggerEdge};
use components::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, parse_str, Expr, ExprUnsafe, ItemStruct, Stmt};
use types::*;

#[proc_macro_attribute]
pub fn device_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_json(&attr);
    let components = parse_components(&config);
    let strukt = parse_macro_input!(item as ItemStruct);
    let struct_name = strukt.ident.clone();
    let statiks = generate_component_statics(&components);
    let static_init = generate_static_init(&components);
    let heap_size = config.sys().heap_size();

    let init_stmts = components.init_stmts;
    let interrupt_unmasks = components.interrupt_unmasks;
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
            ) -> Result<embedded_rust::resources::ResourceID, embedded_rust::RuntimeError>{
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

fn generate_component_statics(components: &Components) -> Vec<Stmt> {
    let sys_tys = &components.sys.ty;
    let in_tys = &components.input_pins.ty;
    let out_tys = &components.output_pins.ty;
    let pwm_tys = &components.pwm_pins.ty;
    let chan_tys = &components.channels.ty;
    let ser_tys = &components.serials.ty;
    let tim_tys = &components.serials.ty;

    let sys_len = components.sys.identifiers.len();
    let in_len = components.input_pins.identifiers.len();
    let out_len = components.output_pins.identifiers.len();
    let pwm_len = components.pwm_pins.identifiers.len();
    let chan_len = components.channels.identifiers.len();
    let ser_len = components.serials.identifiers.len();
    let tim_len = components.timers.identifiers.len();

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

fn generate_static_init(components: &Components) -> ExprUnsafe {
    let sys_idents = &components.sys.identifiers;
    let in_idents = &components.input_pins.identifiers;
    let out_idents = &components.output_pins.identifiers;
    let pwm_idents = &components.pwm_pins.identifiers;
    let channel_idents = &components.channels.identifiers;
    let serial_idents = &components.serials.identifiers;
    let timer_idents = &components.timers.identifiers;

    let sys_index = (0..components.sys.identifiers.len()).map(syn::Index::from);
    let in_index = (0..components.input_pins.identifiers.len()).map(syn::Index::from);
    let out_index = (0..components.output_pins.identifiers.len()).map(syn::Index::from);
    let pwm_index = (0..components.pwm_pins.identifiers.len()).map(syn::Index::from);
    let chan_index = (0..components.channels.identifiers.len()).map(syn::Index::from);
    let ser_index = (0..components.serials.identifiers.len()).map(syn::Index::from);
    let tim_index = (0..components.timers.identifiers.len()).map(syn::Index::from);

    let in_channels = &components.input_pins.channels;
    let in_ports = &components.input_pins.ports;

    let out_channels = &components.output_pins.channels;
    let out_ports = &components.output_pins.ports;

    let pwm_channels = &components.pwm_pins.channels;
    let pwm_ports = &components.pwm_pins.ports;

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

pub(crate) fn parse_json(attributes: &TokenStream) -> types::Config {
    serde_json::from_str(&attributes.to_string()).expect("ParsingError")
}
