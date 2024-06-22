use anyhow::anyhow;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use wasm_component_layer::{Component, Linker, Func, FuncType, ValueType, ComponentType};
use zoon::{eprintln, println, *};

type Engine = wasm_component_layer::Engine<js_wasm_runtime_layer::Engine>;
type Store = Rc<RefCell<wasm_component_layer::Store<(), js_wasm_runtime_layer::Engine>>>;

static DROP_ZONE_ACTIVE: Lazy<Mutable<bool>> = lazy::default();
static COMPONENT_SAID: Lazy<Mutable<Option<String>>> = lazy::default();

// struct PluginHost;

// impl plugin_host_interface::Guest for PluginHost {
//     fn register_plugin(&mut self, plugin: host::Plugin) -> Result<(), host::Error> {
//         println!("[host]: Plugin to registrate: {plugin:#?}");
//         Err("testing error :)".to_owned())
//     }

//     fn log(&mut self, message: &str) {
//         println!("[guest]: {message}");
//     }
// }

async fn load_and_use_component(
    file_list: web_sys::FileList,
    engine: Engine,
    store: Store,
) -> anyhow::Result<()> {
    let mut borrowed_store = store.borrow_mut();
    let mut store = borrowed_store.deref_mut();

    let mut new_component_said = String::new();

    let file_bytes = file_list
        .get(0)
        .ok_or_else(|| anyhow!("failed to get the dropped file"))?
        .apply(|file| JsFuture::from(file.array_buffer()))
        .await
        .map_err(|error| anyhow!("{error:#?}"))?
        .apply_ref(js_sys::Uint8Array::new)
        .to_vec();

    let component = Component::new(&engine, &file_bytes)?;
    // component.imports().

    let mut linker = Linker::default();
    
    let plugin_host_interface = linker.define_instance("wasm-components:calculator/plugin-host".try_into().unwrap_throw())?;
    
    // let register_plugin_func = Func::new(
    //     &mut store, 
    //     FuncType::new(params, results), 
    //     f
    // );
    // plugin_host_interface.define_func("register-plugin", register_plugin_func);
    // register-plugin: func(plugin: plugin) -> result<_, error>;
    
    let log_func = Func::new(
        &mut store, 
        FuncType::new([ValueType::String], []),
        |_store, params, _returns| {
            let message = String::from_value(&params[0]).unwrap_throw();
            println!("[guest]: {message}");
            Ok(())
        }
    );
    plugin_host_interface.define_func("log", log_func)?;

    let instance = linker.instantiate(&mut store, &component)?;

    let calculator_interface = instance
        .exports()
        .instance(&"wasm-components:calculator/calculator".try_into()?)
        .unwrap_throw();

    let plugin_interface = instance
        .exports()
        .instance(&"wasm-components:calculator/plugin".try_into()?)
        .unwrap_throw();

    let sum = calculator_interface
        .func("sum")
        .unwrap_throw()
        .typed::<(f64, f64), f64>()?;

    let a = 1.2;
    let b = 3.4;
    let sum_a_b = sum.call(&mut store, (a, b))?;

    new_component_said.push_str(&format!("\n{a} + {b} = {sum_a_b}"));

    // struct Host;

    // impl host::Host for Host {
    //     fn register_plugin(&mut self, plugin: host::Plugin) -> Result<(), host::Error> {
    //         println!("[host]: Plugin to registrate: {plugin:#?}");
    //         Err("testing error :)".to_owned())
    //     }

    //     fn log(&mut self, message: &str) {
    //         println!("[guest]: {message}");
    //     }
    // }

    // let mut store = Store::default();
    // let module = Module::new(&store, file_bytes).await?;
    // let mut imports = Imports::new();

    // let init_host = host::add_to_imports(&mut store, &mut imports, Host);
    // let (calculator, instance) = calculator::Calculator::instantiate(&mut store, &module, &mut imports).await?;
    // init_host(&instance, &store)?;

    // let init_data = calculator::InitData {
    //     instance_id: 3,
    //     host_name: "MoonZoon Wasm app",
    // };
    // calculator.init_plugin(&mut store, init_data)?;

    // let addends = [1.25, 2.5, 3.1, 60.];
    // let addends_sum = calculator.sum_list(&mut store, &addends)?;
    // new_component_said.push_str(&format!("\nSum {addends:?} = {addends_sum}"));

    COMPONENT_SAID.set(Some(new_component_said));
    println!("Done!");
    Ok(())
}

fn main() {
    start_app("app", root);
}

fn root() -> impl Element {
    let engine = wasm_component_layer::Engine::new(js_wasm_runtime_layer::Engine::default());
    let store = Rc::new(RefCell::new(wasm_component_layer::Store::new(&engine, ())));
    Column::new()
        .after_remove(clone!((engine, store) move |_| {
            drop(store);
            drop(engine);
        }))
        .s(Width::exact(300))
        .s(Align::center())
        .s(Gap::new().y(20))
        .item(drop_zone(engine, store))
        .item_signal(COMPONENT_SAID.signal_cloned().map_some(|text| {
            Paragraph::new()
                .s(Align::new().center_x())
                .content("Component said: ")
                .content(
                    El::new()
                        .s(Font::new().weight(FontWeight::SemiBold))
                        .child(text),
                )
        }))
}

fn drop_zone(engine: Engine, store: Store) -> impl Element {
    El::new()
        .s(Height::exact(200))
        .s(RoundedCorners::all(30))
        .s(Borders::all(Border::new().color(color!("Green")).width(2)))
        .s(Background::new().color_signal(DROP_ZONE_ACTIVE.signal().map_true(|| color!("DarkGreen"))))
        // @TODO refactor with a new MoonZoon ability
        .update_raw_el(move |raw_el| {
            raw_el
                .event_handler_with_options(
                    EventOptions::new().preventable(),
                    |event: events::DragEnter| {
                        event.stop_propagation();
                        event.prevent_default();
                        DROP_ZONE_ACTIVE.set_neq(true);
                    },
                )
                .event_handler_with_options(
                    EventOptions::new().preventable(),
                    |event: events::DragOver| {
                        event.stop_propagation();
                        event.prevent_default();
                        event.data_transfer().unwrap_throw().set_drop_effect("copy");
                    },
                )
                .event_handler_with_options(
                    EventOptions::new().preventable(),
                    |event: events::DragLeave| {
                        event.stop_propagation();
                        event.prevent_default();
                        DROP_ZONE_ACTIVE.set_neq(false);
                    },
                )
                .event_handler_with_options(
                    EventOptions::new().preventable(),
                    move |event: events::Drop| {
                        event.stop_propagation();
                        event.prevent_default();
                        DROP_ZONE_ACTIVE.set_neq(false);
                        let file_list = event.data_transfer().unwrap_throw().files().unwrap_throw();
                        Task::start(clone!((engine, store) async move {
                            if let Err(error) = load_and_use_component(file_list, engine, store).await {
                                eprintln!("{error:#}");
                            }
                        }));
                    },
                )
        })
        .child(
            El::new()
                .s(Align::center())
                // @TODO the new ability shouldn't fire `dragleave` on moving to a child
                .pointer_handling(PointerHandling::none())
                .child("Drop Wasm component here"),
        )
}
