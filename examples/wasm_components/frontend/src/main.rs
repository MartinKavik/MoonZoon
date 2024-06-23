use anyhow::anyhow;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;
use wasm_component_layer::{
    Component, ComponentType, Func, FuncType, OptionType, Record, RecordType, ResultType,
    ResultValue, TypeIdentifier, UnaryComponentType, Value, ValueType,
};
use zoon::{eprintln, println, *};

// @TODO Use macro to generate host bindings once implemented
// https://github.com/DouglasDwyer/wasm_component_layer

// @TODO There are probably bugs in `wasm_component_layer` (0.1.17), see `@TODO`s below

type Engine = wasm_component_layer::Engine<js_wasm_runtime_layer::Engine>;
type Store = Rc<RefCell<wasm_component_layer::Store<(), js_wasm_runtime_layer::Engine>>>;
type Linker = Rc<RefCell<wasm_component_layer::Linker>>;

static DROP_ZONE_ACTIVE: Lazy<Mutable<bool>> = lazy::default();
static COMPONENT_SAID: Lazy<Mutable<Option<String>>> = lazy::default();

struct InitData {
    instance_id: u32,
    host_name: String,
}

impl ComponentType for InitData {
    fn ty() -> ValueType {
        ValueType::Record(
            RecordType::new(
                Some(TypeIdentifier::new("init-data", None)),
                [
                    ("instance-id", ValueType::U32),
                    ("host-name", ValueType::String),
                ],
            )
            .unwrap_throw(),
        )
    }
    fn from_value(value: &Value) -> anyhow::Result<Self> {
        if let Value::Record(record) = value {
            // @TODO check record name?
            let instance_id =
                u32::from_value(&record.field("instance-id").unwrap_throw()).unwrap_throw();
            let host_name =
                String::from_value(&record.field("host-name").unwrap_throw()).unwrap_throw();
            return Ok(Self {
                instance_id,
                host_name,
            });
        }
        Err(anyhow!("InitData has to be Value::Record!"))
    }
    fn into_value(self) -> anyhow::Result<Value> {
        // @TODO why `Record::from_fields` set different field indices?
        Ok(Value::Record(
            Record::new(
                RecordType::new(
                    Some(TypeIdentifier::new("init-data", None)),
                    [
                        ("instance-id", ValueType::U32),
                        ("host-name", ValueType::String),
                    ],
                )
                .unwrap_throw(),
                [
                    ("instance-id", self.instance_id.into_value().unwrap_throw()),
                    ("host-name", self.host_name.into_value().unwrap_throw()),
                ],
            )
            .unwrap_throw(),
        ))
    }
}

impl UnaryComponentType for InitData {}

async fn load_and_use_component(
    file_list: web_sys::FileList,
    engine: Engine,
    store: Store,
    linker: Linker,
) -> anyhow::Result<()> {
    let file_bytes = file_list
        .get(0)
        .ok_or_else(|| anyhow!("failed to get the dropped file"))?
        .apply(|file| JsFuture::from(file.array_buffer()))
        .await
        .map_err(|error| anyhow!("{error:#?}"))?
        .apply_ref(js_sys::Uint8Array::new)
        .to_vec();

    let component = Component::new(&engine, &file_bytes)?;

    let instance = linker
        .borrow()
        .instantiate(store.borrow_mut().deref_mut(), &component)?;

    let calculator_interface = instance
        .exports()
        .instance(&"wasm-components:calculator/calculator".try_into()?)
        .unwrap_throw();

    let sum = calculator_interface
        .func("sum")
        .unwrap_throw()
        .typed::<(f64, f64), f64>()?;

    let sum_list = calculator_interface
        .func("sum-list")
        .unwrap_throw()
        .typed::<Vec<f64>, f64>()?;

    let plugin_interface = instance
        .exports()
        .instance(&"wasm-components:calculator/plugin".try_into()?)
        .unwrap_throw();

    let init_plugin = plugin_interface
        .func("init-plugin")
        .unwrap_throw()
        .typed::<InitData, ()>()?;

    let mut new_component_said = String::new();

    init_plugin.call(
        store.borrow_mut().deref_mut(),
        InitData {
            instance_id: 3,
            host_name: "MoonZoon Wasm app".to_owned(),
        },
    )?;

    let a = 1.2;
    let b = 3.4;
    let sum_a_b = sum.call(store.borrow_mut().deref_mut(), (a, b))?;
    new_component_said.push_str(&format!("\n{a} + {b} = {sum_a_b}"));

    let addends = vec![1.25, 2.5, 3.1, 60.];
    let addends_sum = sum_list.call(store.borrow_mut().deref_mut(), addends.clone())?;
    new_component_said.push_str(&format!("\nSum {addends:?} = {addends_sum}"));

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

    let mut linker = wasm_component_layer::Linker::default();
    let plugin_host_interface = linker
        .define_instance(
            "wasm-components:calculator/plugin-host"
                .try_into()
                .unwrap_throw(),
        )
        .unwrap_throw();

    let register_plugin_func = Func::new(
        store.borrow_mut().deref_mut(),
        FuncType::new(
            [ValueType::Record(
                RecordType::new(
                    Some(TypeIdentifier::new("plugin-params", None)),
                    [
                        ("name", ValueType::String),
                        (
                            "version",
                            ValueType::Option(OptionType::new(ValueType::F32)),
                        ),
                    ],
                )
                .unwrap_throw(),
            )],
            [ValueType::Result(ResultType::new(
                None,
                Some(ValueType::String),
            ))],
        ),
        // @TODO Why it's not fired?
        |_store, params, returns| {
            let plugin = &params[0];
            println!("[host]: Plugin to registrate: {plugin:#?}");
            returns[0] = Value::Result(
                ResultValue::new(
                    ResultType::new(None, Some(ValueType::String)),
                    Err(Some(Value::String(Arc::from("testing error :)")))),
                )
                .unwrap_throw(),
            );
            Ok(())
        },
    );
    plugin_host_interface
        .define_func("register-plugin", register_plugin_func)
        .unwrap_throw();

    let log_func = Func::new(
        store.borrow_mut().deref_mut(),
        FuncType::new([ValueType::String], []),
        |_store, params, _returns| {
            let message = String::from_value(&params[0]).unwrap_throw();
            println!("[guest]: {message}");
            Ok(())
        },
    );
    plugin_host_interface
        .define_func("log", log_func)
        .unwrap_throw();

    let linker = Rc::new(RefCell::new(linker));

    Column::new()
        .after_remove(clone!((engine, store, linker) move |_| {
            drop(linker);
            drop(store);
            drop(engine);
        }))
        .s(Width::exact(300))
        .s(Align::center())
        .s(Gap::new().y(20))
        .item(drop_zone(engine, store, linker))
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

fn drop_zone(engine: Engine, store: Store, linker: Linker) -> impl Element {
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
                        Task::start(clone!((engine, store, linker) async move {
                            if let Err(error) = load_and_use_component(file_list, engine, store, linker).await {
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
