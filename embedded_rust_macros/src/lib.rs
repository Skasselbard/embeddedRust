#![feature(proc_macro_span)]
mod config;

mod devices;
mod generation;
mod types;
// use embedded_rust::device::stm32f1xx::{Gpio, TriggerEdge};
use config::Config;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
use types::*;

#[proc_macro_attribute]
pub fn device_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_yaml(&attr);
    let strukt = parse_macro_input!(item as ItemStruct);
    let struct_name = strukt.ident.clone();
    let statiks = generation::component_statics(&config);
    let static_init = generation::static_init(&config);
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
                        INPUT_PINS_ARRAY.as_mut().unwrap(),
                        OUTPUT_PINS_ARRAY.as_mut().unwrap(),
                        PWM_PINS_ARRAY.as_mut().unwrap(),
                        CHANNELS_ARRAY.as_mut().unwrap(),
                        SERIALS_ARRAY.as_mut().unwrap(),
                        TIMERS_ARRAY.as_mut().unwrap(),
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
}

pub(crate) fn parse_yaml(attributes: &TokenStream) -> Config {
    let mut att_folded = Vec::new();
    // extract all spans from the attribute token stream
    attributes
        .clone()
        .into_iter()
        .for_each(|elem| att_folded.push(elem.span()));
    // join all spans
    let att_span = att_folded
        .iter()
        .fold(att_folded[0], |acc, elem| acc.join(*elem).unwrap());
    let device_definition = att_span.source_text().expect("msg");
    serde_yaml::from_str(&device_definition).expect("ParsingError")
}
