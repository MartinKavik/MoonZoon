wit_bindgen::generate!({
    inline: r#"
        package wasm-components:calculator;

        interface host {
            type error = string;

            record plugin {
                name: string,
                version: option<float32>,
            }

            // Register plugin in the host
            register-plugin: func(plugin: plugin) -> result<_, error>;

            log: func(message: string);
        }

        interface plugin {
            record init-data {
                instance-id: u32,
                host-name: string,
            }

            init-plugin: func(data: init-data);
        }

        interface calculator {
            sum: func(a: float64, b: float64) -> float64;

            sum-list: func(addends: list<float64>) -> float64;
        }

        world guest {
            import host;
            export plugin;
            export calculator;
        }
    "#,
});

use wasm_components::calculator::host as host_interface;
use exports::wasm_components::calculator::{
    calculator as calculator_interface,
    plugin as plugin_interface,
};

macro_rules! log {
    ($($arg:tt)*) => (host_interface::log(&format!($($arg)*)))
}

struct Calculator;

impl calculator_interface::Guest for Calculator {
    fn sum(a: f64, b: f64) -> f64 {
        let result = a + b;
        log!("sum result is {result}");
        result
    }

    fn sum_list(addends: Vec<f64>) -> f64 {
        let result = addends.iter().sum();
        log!("sum_array result is {result}");
        result
    }
}

impl plugin_interface::Guest for Calculator {
    fn init_plugin(data: plugin_interface::InitData) {
        log!("calculator init-data: '{data:#?}'");
        let plugin = host_interface::Plugin {
            name: "Calculator".to_owned(),
            version: None,
        };
        if let Err(error) = host_interface::register_plugin(&plugin) {
            log!("plugin registration failed: '{error}'");
        }
    }
}

export!(Calculator);
