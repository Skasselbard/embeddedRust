use embedded_rust_devices::*;
use quote::format_ident;
use syn::{parse_quote, parse_str, Expr, Stmt};

/// keywords to build the correct lines of gpio code
struct GpioKeys {
    channel: String,
    pin: String,
    control_reg: String,
    direction: String,
    pin_mode: String,
}

/// parse json to internal structures
pub(crate) fn parse(gpio_list: Vec<Vec<String>>) -> Vec<Gpio> {
    let mut parsed_gpio_list = Vec::new();
    for gpio in gpio_list {
        parsed_gpio_list.push(parse_gpio(gpio));
    }
    parsed_gpio_list
}

/// gpio code generation
pub(crate) fn generate(gpio_list: Vec<Gpio>) -> Vec<Stmt> {
    let mut stmts = channel_init_statements(&gpio_list);
    for gpio in gpio_list {
        stmts.append(&mut generate_gpio(gpio));
    }
    stmts
}

pub(crate) fn generate_components(gpio_list: &Vec<Gpio>) -> (Vec<syn::Expr>, Vec<syn::Expr>) {
    let mut configs = Vec::new();
    let mut identifiers = Vec::new();
    for gpio in gpio_list {
        let channel = match gpio.channel() {
            Channel::A => format_ident!("A"),
            Channel::B => format_ident!("B"),
            Channel::C => format_ident!("C"),
            Channel::D => format_ident!("D"),
            Channel::E => format_ident!("E"),
        };
        let port = if gpio.port() < Port::P10 {
            format_ident!("P0{}", port_to_string(gpio.port()))
        } else {
            format_ident!("P{}", port_to_string(gpio.port()))
        };
        let direction = match gpio.direction() {
            Direction::Alternate => parse_str::<Expr>("Alternate"),
            Direction::Input(edge) => parse_str::<Expr>(&format!(
                "Input({})",
                match edge {
                    Some(TriggerEdge::All) => "Some(TriggerEdge::All)",
                    Some(TriggerEdge::Falling) => "Some(TriggerEdge::Falling)",
                    Some(TriggerEdge::Rising) => "Some(TriggerEdge::Rising)",
                    None => "None",
                }
            )),
            Direction::Output => parse_str::<Expr>("Output"),
        }
        .unwrap();
        let mode = match gpio.mode() {
            PinMode::Analog => format_ident!("Analog"),
            PinMode::Floating => format_ident!("Floating"),
            PinMode::OpenDrain => format_ident!("OpenDrain"),
            PinMode::PullDown => format_ident!("PullDown"),
            PinMode::PullUp => format_ident!("PullUp"),
            PinMode::PushPull => format_ident!("PushPull"),
        };
        configs.push(parse_quote!(
            ComponentConfiguration::Gpio(Gpio::new(Pin::new(Channel::#channel, Port::#port), Direction::#direction, PinMode::#mode))
        ));
        let pin_var = format_ident!("pin_{}", to_component_keys(*gpio).pin);
        identifiers.push(parse_quote!(
            #pin_var.downgrade()
        ));
    }
    (configs, identifiers)
}

/// build the statements to initialize the gpio channels
fn channel_init_statements(gpio_list: &Vec<Gpio>) -> Vec<Stmt> {
    use std::collections::HashSet;
    let mut channels = HashSet::with_capacity(5);
    let mut stmts = Vec::with_capacity(5);
    for gpio in gpio_list {
        // only one initialization for each channel
        if !channels.contains(&gpio.channel()) {
            // remember initialized channel
            channels.insert(gpio.channel());
            // build channel identifiers
            let channel_lower = channel_to_string(gpio.channel());
            let channel_upper = channel_lower.to_uppercase().to_string();
            let channel_lower = format_ident!("{}", channel_lower);
            let channel_upper = format_ident!("{}", channel_upper);
            // build peripherals identifier
            let peripherals_ident = format_ident!("{}", super::PERIPHERALS_KEY);
            // expand: let mut gpiox = peripherals.GPIOX.split(&mut rcc.apb2);
            // its always apb2 on this boards
            stmts.push(parse_quote!(
                let mut #channel_lower = #peripherals_ident.#channel_upper.split(&mut rcc.apb2);
            ))
        }
    }
    stmts
}

fn generate_gpio(gpio: Gpio) -> Vec<Stmt> {
    // The gpios differ in initialization, so we need to know which components we have to use
    let component_keys = to_component_keys(gpio);
    // build identifiers
    let pin_ident = format_ident!("{}", component_keys.pin);
    let pin_var_ident = format_ident!("pin_{}", component_keys.pin);
    let channel_ident = format_ident!("{}", component_keys.channel);
    // the name of the gpio functions has no global pattern for all configurations
    // so we need to check the gpio configuration again
    let init_function_ident = if gpio.mode() == PinMode::Analog {
        format_ident!("into_analog")
    } else if gpio.direction() == Direction::Alternate {
        format_ident!("into_alternate_{}", component_keys.pin_mode)
    } else {
        format_ident!(
            "into_{}_{}",
            component_keys.pin_mode,
            component_keys.direction
        )
    };
    let control_reg_ident = format_ident!("{}", component_keys.control_reg);
    // expand: let mut pin_pxy = gpiox.pxy.into_smth(&mut gpiox.control_reg);
    let mut stmts: Vec<Stmt> = parse_quote!(
        let mut #pin_var_ident = #channel_ident.#pin_ident.#init_function_ident(&mut #channel_ident.#control_reg_ident);
    );
    // if the pin shall be an interrupt source, we need additional configuration
    match gpio.direction() {
        Direction::Input(Some(edge)) => {
            let peripherals_ident = format_ident!("{}", super::PERIPHERALS_KEY);
            let edge_ident = match edge {
                TriggerEdge::Rising => format_ident!("RISING"),
                TriggerEdge::Falling => format_ident!("FALLING"),
                TriggerEdge::All => format_ident!("RISING_FALLING"),
            };
            // expand:
            // pin_pxy.make_interrupt_source(&mut afio);
            // pin_pxy.trigger_on_edge(&peripherals.EXTI, Edge::EDGE_TYPE);
            // pin_pxy.enable_interrupt(&peripherals.EXTI);
            let mut interrupt_stmts: Vec<Stmt> = parse_quote!(
                #pin_var_ident.make_interrupt_source(&mut afio);
                #pin_var_ident.trigger_on_edge(&#peripherals_ident.EXTI, Edge::#edge_ident);
                #pin_var_ident.enable_interrupt(&#peripherals_ident.EXTI);
            );
            stmts.append(&mut interrupt_stmts);
        }
        _ => {}
    }
    stmts
}

fn to_component_keys(gpio: Gpio) -> GpioKeys {
    let pin_number = port_to_string(gpio.port());
    let pin_prefix = match gpio.channel() {
        Channel::A => "pa",
        Channel::B => "pb",
        Channel::C => "pc",
        Channel::D => "pd",
        Channel::E => "pe",
    };
    let pin = format!("{}{}", pin_prefix, pin_number);
    let channel: String = channel_to_string(gpio.channel());
    let control_reg: String = if gpio.port() < Port::P08 {
        "crl"
    } else {
        "crh"
    }
    .into();
    let direction: String = match gpio.direction() {
        Direction::Alternate => unimplemented!(),
        Direction::Input(_) => "input",
        Direction::Output => "output",
    }
    .into();
    let pin_mode: String = match gpio.mode() {
        PinMode::Analog => "analog",
        PinMode::Floating => "floating",
        PinMode::OpenDrain => "open_drain",
        PinMode::PullDown => "pull_down",
        PinMode::PullUp => "pull_up",
        PinMode::PushPull => "push_pull",
    }
    .into();
    GpioKeys {
        pin,
        channel,
        control_reg,
        direction,
        pin_mode,
    }
}

fn port_to_string(port: Port) -> String {
    match port {
        Port::P00 => "0",
        Port::P01 => "1",
        Port::P02 => "2",
        Port::P03 => "3",
        Port::P04 => "4",
        Port::P05 => "5",
        Port::P06 => "6",
        Port::P07 => "7",
        Port::P08 => "8",
        Port::P09 => "9",
        Port::P10 => "10",
        Port::P11 => "11",
        Port::P12 => "12",
        Port::P13 => "13",
        Port::P14 => "14",
        Port::P15 => "15",
    }
    .into()
}

fn channel_to_string(channel: Channel) -> String {
    match channel {
        Channel::A => "gpioa",
        Channel::B => "gpiob",
        Channel::C => "gpioc",
        Channel::D => "gpiod",
        Channel::E => "gpioe",
    }
    .into()
}

/// converts the parsed json into the internally used data structures
fn parse_gpio(gpio: Vec<String>) -> Gpio {
    let mut mode = None;
    let mut direction = None;
    let mut id = None;
    let mut trigger_edge = None;
    // parse all keys and panic if one is assigned multiple times
    for ref key in gpio {
        if let Some(parsed_mode) = pin_mode_from_key(key) {
            if mode.is_some() {
                panic!("multiplie mode keywords in gpio");
            } else {
                mode = Some(parsed_mode);
            }
        };
        if let Some(parsed_id) = pin_from_key(key) {
            if id.is_some() {
                panic!("multiplie pin keywords in gpio");
            } else {
                id = Some(parsed_id);
            }
        };
        if let Some(parsed_direction) = direction_from_key(key) {
            if direction.is_some() {
                panic!("multiplie direction keywords in gpio");
            } else {
                direction = Some(parsed_direction);
            }
        };
        if let Some(parsed_trigger_edge) = trigger_edge_from_key(key) {
            if trigger_edge.is_some() {
                panic!("multiplie interrupt keywords in gpio");
            } else {
                trigger_edge = Some(parsed_trigger_edge);
            }
        };
    }
    let direction = match direction.expect("no valid direction specified") {
        Direction::Input(_) => Direction::Input(trigger_edge),
        x => x,
    };
    let gpio = Gpio::new(
        id.expect("no valid pin specified"),
        direction,
        mode.expect("no valid pin mode specified"),
    );
    check_validity(gpio);
    gpio
}

/// checks the configuration and panics if it is invalid
fn check_validity(gpio: Gpio) {
    match gpio.mode() {
        PinMode::Analog => {
            if gpio.direction() == Direction::Output || gpio.direction() == Direction::Alternate {
                panic!("analog pins have to be input pins")
            }
        }
        PinMode::Floating => {
            if gpio.direction() == Direction::Output || gpio.direction() == Direction::Alternate {
                panic!("floating pins have to be input pins")
            }
        }
        PinMode::OpenDrain => match gpio.direction() {
            Direction::Input(_) => panic!("open drain pins have to be output or alternate pins"),
            _ => {}
        },
        PinMode::PullDown => {
            if gpio.direction() == Direction::Output || gpio.direction() == Direction::Alternate {
                panic!("pull down pins have to be input pins")
            }
        }
        PinMode::PullUp => {
            if gpio.direction() == Direction::Output || gpio.direction() == Direction::Alternate {
                panic!("pull up pins have to be input pins")
            }
        }
        PinMode::PushPull => match gpio.direction() {
            Direction::Input(_) => panic!("open drain pins have to be output or alternate pins"),
            _ => {}
        },
    }
}

fn trigger_edge_from_key(key: &str) -> Option<TriggerEdge> {
    match key {
        "Interrupt" | "INTERRUPT" | "interrupt" | "Rising" | "RISING" | "rising" => {
            Some(TriggerEdge::Rising)
        }
        "Falling" | "FALLING" | "falling" => Some(TriggerEdge::Falling),
        "All" | "ALL" | "all" | "RisingFalling" | "RISINGFALLING" | "risingfalling"
        | "rising_falling" => Some(TriggerEdge::All),
        _ => None,
    }
}

fn direction_from_key(key: &str) -> Option<Direction> {
    match key {
        "input" | "Input" | "INPUT" | "in" | "IN" => Some(Direction::Input(None)),
        "output" | "Output" | "OUTPUT" | "out" | "OUT" => Some(Direction::Output),
        "alternate" | "Alternate" | "ALTERNATE" | "alt" | "ALT" => Some(Direction::Alternate),
        _ => None,
    }
}

fn pin_mode_from_key(key: &str) -> Option<PinMode> {
    match key {
        "Analog" | "analog" | "ANALOG" => Some(PinMode::Analog),
        "Floating" | "floating" | "FLOATING" => Some(PinMode::Floating),
        "OpenDrain" | "open_drain" | "opendrain" | "OPENDRAIN" => Some(PinMode::OpenDrain),
        "PullDown" | "pull_down" | "pulldown" | "PULLDOWN" => Some(PinMode::PullDown),
        "PullUp" | "pull_up" | "pullup" | "PULLUP" => Some(PinMode::PullUp),
        "PushPull" | "push_pull" | "pushpull" | "PUSHPULL" => Some(PinMode::PushPull),
        _ => None,
    }
}

fn pin_from_key(key: &str) -> Option<Pin> {
    match key {
        "pa0" | "PA0" => Some(Pin::new(Channel::A, Port::P00)),
        "pa1" | "PA1" => Some(Pin::new(Channel::A, Port::P01)),
        "pa2" | "PA2" => Some(Pin::new(Channel::A, Port::P02)),
        "pa3" | "PA3" => Some(Pin::new(Channel::A, Port::P03)),
        "pa4" | "PA4" => Some(Pin::new(Channel::A, Port::P04)),
        "pa5" | "PA5" => Some(Pin::new(Channel::A, Port::P05)),
        "pa6" | "PA6" => Some(Pin::new(Channel::A, Port::P06)),
        "pa7" | "PA7" => Some(Pin::new(Channel::A, Port::P07)),
        "pa8" | "PA8" => Some(Pin::new(Channel::A, Port::P08)),
        "pa9" | "PA9" => Some(Pin::new(Channel::A, Port::P09)),
        "pa10" | "PA10" => Some(Pin::new(Channel::A, Port::P10)),
        "pa11" | "PA11" => Some(Pin::new(Channel::A, Port::P11)),
        "pa12" | "PA12" => Some(Pin::new(Channel::A, Port::P12)),
        "pa13" | "PA13" => Some(Pin::new(Channel::A, Port::P13)),
        "pa14" | "PA14" => Some(Pin::new(Channel::A, Port::P14)),
        "pa15" | "PA15" => Some(Pin::new(Channel::A, Port::P15)),
        "pb0" | "PB0" => Some(Pin::new(Channel::B, Port::P00)),
        "pb1" | "PB1" => Some(Pin::new(Channel::B, Port::P01)),
        "pb2" | "PB2" => Some(Pin::new(Channel::B, Port::P02)),
        "pb3" | "PB3" => Some(Pin::new(Channel::B, Port::P03)),
        "pb4" | "PB4" => Some(Pin::new(Channel::B, Port::P04)),
        "pb5" | "PB5" => Some(Pin::new(Channel::B, Port::P05)),
        "pb6" | "PB6" => Some(Pin::new(Channel::B, Port::P06)),
        "pb7" | "PB7" => Some(Pin::new(Channel::B, Port::P07)),
        "pb8" | "PB8" => Some(Pin::new(Channel::B, Port::P08)),
        "pb9" | "PB9" => Some(Pin::new(Channel::B, Port::P09)),
        "pb10" | "PB10" => Some(Pin::new(Channel::B, Port::P10)),
        "pb11" | "PB11" => Some(Pin::new(Channel::B, Port::P11)),
        "pb12" | "PB12" => Some(Pin::new(Channel::B, Port::P12)),
        "pb13" | "PB13" => Some(Pin::new(Channel::B, Port::P13)),
        "pb14" | "PB14" => Some(Pin::new(Channel::B, Port::P14)),
        "pb15" | "PB15" => Some(Pin::new(Channel::B, Port::P15)),
        "pc0" | "PC0" => Some(Pin::new(Channel::C, Port::P00)),
        "pc1" | "PC1" => Some(Pin::new(Channel::C, Port::P01)),
        "pc2" | "PC2" => Some(Pin::new(Channel::C, Port::P02)),
        "pc3" | "PC3" => Some(Pin::new(Channel::C, Port::P03)),
        "pc4" | "PC4" => Some(Pin::new(Channel::C, Port::P04)),
        "pc5" | "PC5" => Some(Pin::new(Channel::C, Port::P05)),
        "pc6" | "PC6" => Some(Pin::new(Channel::C, Port::P06)),
        "pc7" | "PC7" => Some(Pin::new(Channel::C, Port::P07)),
        "pc8" | "PC8" => Some(Pin::new(Channel::C, Port::P08)),
        "pc9" | "PC9" => Some(Pin::new(Channel::C, Port::P09)),
        "pc10" | "PC10" => Some(Pin::new(Channel::C, Port::P10)),
        "pc11" | "PC11" => Some(Pin::new(Channel::C, Port::P11)),
        "pc12" | "PC12" => Some(Pin::new(Channel::C, Port::P12)),
        "pc13" | "PC13" => Some(Pin::new(Channel::C, Port::P13)),
        "pc14" | "PC14" => Some(Pin::new(Channel::C, Port::P14)),
        "pc15" | "PC15" => Some(Pin::new(Channel::C, Port::P15)),
        "pd0" | "PD0" => Some(Pin::new(Channel::D, Port::P00)),
        "pd1" | "PD1" => Some(Pin::new(Channel::D, Port::P01)),
        "pd2" | "PD2" => Some(Pin::new(Channel::D, Port::P02)),
        "pd3" | "PD3" => Some(Pin::new(Channel::D, Port::P03)),
        "pd4" | "PD4" => Some(Pin::new(Channel::D, Port::P04)),
        "pd5" | "PD5" => Some(Pin::new(Channel::D, Port::P05)),
        "pd6" | "PD6" => Some(Pin::new(Channel::D, Port::P06)),
        "pd7" | "PD7" => Some(Pin::new(Channel::D, Port::P07)),
        "pd8" | "PD8" => Some(Pin::new(Channel::D, Port::P08)),
        "pd9" | "PD9" => Some(Pin::new(Channel::D, Port::P09)),
        "pd10" | "PD10" => Some(Pin::new(Channel::D, Port::P10)),
        "pd11" | "PD11" => Some(Pin::new(Channel::D, Port::P11)),
        "pd12" | "PD12" => Some(Pin::new(Channel::D, Port::P12)),
        "pd13" | "PD13" => Some(Pin::new(Channel::D, Port::P13)),
        "pd14" | "PD14" => Some(Pin::new(Channel::D, Port::P14)),
        "pd15" | "PD15" => Some(Pin::new(Channel::D, Port::P15)),
        "pe0" | "PE0" => Some(Pin::new(Channel::E, Port::P00)),
        "pe1" | "PE1" => Some(Pin::new(Channel::E, Port::P01)),
        "pe2" | "PE2" => Some(Pin::new(Channel::E, Port::P02)),
        "pe3" | "PE3" => Some(Pin::new(Channel::E, Port::P03)),
        "pe4" | "PE4" => Some(Pin::new(Channel::E, Port::P04)),
        "pe5" | "PE5" => Some(Pin::new(Channel::E, Port::P05)),
        "pe6" | "PE6" => Some(Pin::new(Channel::E, Port::P06)),
        "pe7" | "PE7" => Some(Pin::new(Channel::E, Port::P07)),
        "pe8" | "PE8" => Some(Pin::new(Channel::E, Port::P08)),
        "pe9" | "PE9" => Some(Pin::new(Channel::E, Port::P09)),
        "pe10" | "PE10" => Some(Pin::new(Channel::E, Port::P10)),
        "pe11" | "PE11" => Some(Pin::new(Channel::E, Port::P11)),
        "pe12" | "PE12" => Some(Pin::new(Channel::E, Port::P12)),
        "pe13" | "PE13" => Some(Pin::new(Channel::E, Port::P13)),
        "pe14" | "PE14" => Some(Pin::new(Channel::E, Port::P14)),
        "pe15" | "PE15" => Some(Pin::new(Channel::E, Port::P15)),
        _ => None,
    }
}
