mod gpio;
use embedded_rust_devices::{Gpio, TriggerEdge};
use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use serde_derive::Deserialize;
use syn::{parse_macro_input, parse_quote, Block, Stmt};

#[derive(Deserialize, Debug)]
struct Device {
    gpios: Vec<Vec<String>>,
}

const PERIPHERALS_KEY: &str = "peripherals";

#[proc_macro]
pub fn configure_device(input: TokenStream) -> TokenStream {
    // eprintln!("{}", input.to_string());
    let device = parse_json(&input);
    let peripherals_ident = format_ident!("{}", PERIPHERALS_KEY);
    let gpios = gpio::parse(device.gpios);
    let mut init_block: Block = parse_quote!(
            {
            use stm32f1xx_hal::prelude::*;
            use stm32f1xx_hal::gpio::*;
            use embedded_rust_devices::*;
            use embedded_rust_devices::resources::*;
            let #peripherals_ident = stm32f1xx_hal::pac::Peripherals::take().unwrap();
            let mut flash = #peripherals_ident.FLASH.constrain();
            let mut rcc = #peripherals_ident.RCC.constrain();
            let clocks = rcc.cfgr.freeze(&mut flash.acr);
            let mut afio = #peripherals_ident.AFIO.constrain(&mut rcc.apb2);
            }
    );
    let mut return_array = generate_component_config_array(&gpios);
    init_block.stmts.append(&mut gpio::generate(gpios));
    // init_block.stmts.push(Stmt::Expr(return_array));
    init_block.stmts.append(&mut return_array);
    init_block.to_token_stream().into()
}

fn parse_json(attributes: &TokenStream) -> Device {
    serde_json::from_str(&attributes.to_string()).expect("ParsingError")
}

fn generate_component_config_array(gpios: &Vec<Gpio>) -> Vec<Stmt> {
    let (gpio_configs, gpios) = gpio::generate_components(gpios);
    parse_quote!(
        let init_closure = ||{
            #(#gpios.register_component(#gpio_configs);)*
        };
        // return array with the component configurations
        ([#(#gpio_configs),*], init_closure)
    )
}
