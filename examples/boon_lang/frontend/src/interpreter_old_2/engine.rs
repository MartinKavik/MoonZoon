// @TODO remove
#![allow(dead_code)]

use std::fmt;
use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::Arc;

// @TODO replace with https://without.boats/blog/waitmap/ 
// or https://crates.io/crates/whirlwind
// or https://github.com/wvwwvwwv/scalable-concurrent-containers/
use indexmap::IndexMap;

use zoon::futures_channel::{oneshot, mpsc};
use zoon::futures_util::stream::{self, Stream, StreamExt};
use zoon::futures_util::future::join_all;
use zoon::{Task, TaskHandle};

// @TODO bounded channels with timers + console.logs?

// @TODO optimize code with https://crates.io/crates/kanal
// and https://crates.io/crates/smallvec ?

pub type Functions = IndexMap<FunctionName, Function>;
pub type Arguments = IndexMap<ArgumentName, Argument>;
pub type Variables = IndexMap<VariableName, Variable>;

pub trait AsyncDebugFormat {
    async fn async_debug_format(&self) -> String {
        self.async_debug_format_with_formatter(Formatter::default()).await
    }

    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String;
}

#[derive(Debug, Clone, Copy)]
pub struct Formatter {
    indent_spaces: u32,
    indent_level: u32,
}

impl Default for Formatter {
    fn default() -> Self {
        Self {
            indent_spaces: 4,
            indent_level: 0,
        }
    }
}

impl Formatter {
    pub fn increase_level(self) -> Formatter {
        let mut updated = self.clone();
        updated.indent_level += 1;
        updated
    }

    pub fn indent(&self, text: &str) -> String {
        let indentation = (self.indent_spaces * self.indent_level) as usize;
        format!("{:indentation$}{text}", "")
    }
}

// @TODO Resolve unwraps - some of them fail on a dependency actor/variable drop

#[derive(Debug, Default)]
pub struct Engine {
    pub functions: Functions,
    pub variables: Variables,
}

impl Engine {
    // @TODO `address` should work for the scope, not only for the root
    pub async fn set_link_value(&self, address: &str, actor: VariableActor) {
        let address_parts = address.split(".").collect::<Vec<_>>();

        if address_parts.len() == 1 {
            let variable_actor = self
                .variables
                .get(&VariableName::new(address_parts[0]))
                .unwrap()
                .actor();
            match variable_actor.get_value().await {
                VariableValue::Link(variable_value_link) => {
                    variable_value_link.set_target(actor);
                }
                _ => panic!("Failed to set link value - the variable is not a Link")
            }
        } else {
            let root = self
                .variables
                .get(&VariableName::new(address_parts[0]))
                .unwrap()
                .actor();

            let mut parent_or_link_actor = root;
            for address_part in address_parts.into_iter().skip(1) {
                parent_or_link_actor = match parent_or_link_actor.get_value().await {
                    VariableValue::Object(VariableValueObject { variables }) => {
                        variables
                            .get(&VariableName::new(address_part))
                            .unwrap()
                            .actor()
                    }
                    _ => unreachable!("Link path parts have to be 'VariableValue::Object'")
                }
            }
            match parent_or_link_actor.get_value().await {
                VariableValue::Link(variable_value_link) => {
                    variable_value_link.set_target(actor);
                }
                _ => panic!("Failed to set link value - the variable is not a Link")
            }
        }
    }
}

impl AsyncDebugFormat for Engine {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        let mut output = String::new();
        output.push_str("--- ENGINE ---\n");

        let functions = {
            let formatter = formatter.increase_level();
            self
                .functions
                .keys()
                .map(|FunctionName(name)| formatter.indent(name))
                .collect::<Vec<_>>()
                .join("\n")
        };
        output.push_str(&format!("functions: LIST {{\n{functions}\n}}\n"));

        let variables = {
            let formatter = formatter.increase_level();
            self
                .variables
                .values()
                .map(move |variable| async move {
                    let variable = variable.async_debug_format_with_formatter(formatter).await;
                    formatter.indent(&format!("{variable},"))
                })
        };
        let variables = join_all(variables)
            .await
            .join("\n");
        output.push_str(&format!("variables: [\n{variables}\n]\n"));

        output.push_str("--------------\n");
        output
    }
}

pub struct Function {
    name: FunctionName,
    // @TODO Option -> special variable value type? (the same for Link?)
    closure: Arc<dyn Fn(Arguments, Option<PassedArgument>) -> Pin<Box<dyn Future<Output = VariableActor>>>>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
         .field("name", &self.name)
         .field("closure", &"[closure]")
         .finish()
    }
}

impl Function {
    pub fn new<Fut: Future<Output = VariableActor> + 'static>(name: FunctionName, closure: impl Fn(Arguments, Option<PassedArgument>) -> Fut + 'static) -> Self {
        let closure = Arc::new(move |arguments: Arguments, passed_argument: Option<PassedArgument>| { 
            Box::pin(closure(arguments, passed_argument)) as Pin<Box<dyn Future<Output = VariableActor>>>
        });
        Self { name, closure }
    }

    pub async fn run(&self, arguments: Arguments, passed_argument: Option<PassedArgument>) -> VariableActor {
        (self.closure)(arguments, passed_argument).await
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FunctionName(Arc<String>);

impl FunctionName {
    pub fn new(name: impl ToString) -> Self {
        Self(Arc::new(name.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct PassedArgument {
    actor: VariableActor,
}

impl PassedArgument {
    pub fn new(actor: VariableActor) -> Self {
        Self { actor }
    }

    pub fn actor(&self) -> VariableActor {
        self.actor.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Argument {
    // @TODO remove `allow`
    #[allow(dead_code)]
    name: ArgumentName,
    in_out: ArgumentInOut
}

#[derive(Debug, Clone)]
pub enum ArgumentInOut {
    In(ArgumentIn),
    Out(ArgumentOut),
}

#[derive(Debug, Clone)]
pub struct ArgumentIn {
    actor: VariableActor,
}

impl ArgumentIn {
    pub fn actor(&self) -> VariableActor {
        self.actor.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentOut {
    set_actor_sender: mpsc::UnboundedSender<VariableActor>,
    get_actor_sender: mpsc::UnboundedSender<oneshot::Sender<VariableActor>>,
    #[allow(dead_code)]
    task_with_actor: Arc<TaskHandle>,
}

impl ArgumentOut {
    pub async fn actor(&self) -> VariableActor {
        let (actor_sender, actor_receiver) = oneshot::channel::<VariableActor>();
        self.get_actor_sender.unbounded_send(actor_sender).unwrap();
        actor_receiver.await.unwrap()
    }
}

impl ArgumentOut {
    pub fn set_actor(&self, actor: VariableActor) {
        self
            .set_actor_sender
            .unbounded_send(actor)
            .unwrap();
        self.set_actor_sender.close_channel();
    }
}

impl Argument {
    pub fn new_in(name: ArgumentName, actor: VariableActor) -> Self {
        Self { name, in_out: ArgumentInOut::In(ArgumentIn { actor }) }
    }

    pub fn new_out(name: ArgumentName) -> Self {
        let (set_actor_sender, mut set_actor_receiver) = mpsc::unbounded();
        let (get_actor_sender, mut get_actor_receiver) = mpsc::unbounded();
        Self { 
            name, 
            in_out: ArgumentInOut::Out(ArgumentOut { 
                set_actor_sender,
                get_actor_sender,
                task_with_actor: Arc::new(Task::start_droppable(async move {
                    let actor = set_actor_receiver.next().await.unwrap();
                    while let Some(actor_sender) = get_actor_receiver.next().await {
                        actor_sender.send(actor.clone()).unwrap();
                    }
                }))
            })
        }
    }

    pub fn argument_in(&self) -> Option<&ArgumentIn> {
        match &self.in_out {
            ArgumentInOut::In(argument_in) => Some(argument_in),
            _ => None
        }
    }

    pub fn argument_out(&self) -> Option<&ArgumentOut> {
        match &self.in_out {
            ArgumentInOut::Out(argument_out) => Some(argument_out),
            _ => None
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ArgumentName(Arc<String>);

impl ArgumentName {
    pub fn new(name: impl ToString) -> Self {
        Self(Arc::new(name.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: VariableName,
    actor: VariableActor,
}

impl Variable {
    pub fn new(name: VariableName, actor: VariableActor) -> Self {
        Self { name, actor }
    }

    pub fn actor(&self) -> VariableActor {
        self.actor.clone()
    }
}

impl AsyncDebugFormat for Variable {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        let VariableName(name) = &self.name;
        let value = self.actor.async_debug_format_with_formatter(formatter).await;
        format!("{name}: {value}")
    }
}

#[derive(Debug)]
enum VariableActorMessage {
    GetValue { value_sender: oneshot::Sender<VariableValue> },
    // @TODO remove?
    // SetValue { new_value: VariableValue },
    ValueChanges { change_sender: mpsc::UnboundedSender<VariableValueChanged> },
}

enum VariableActorValueOrMessage {
    Value(VariableValue),
    Message(VariableActorMessage)
}

impl AsyncDebugFormat for VariableActorValueOrMessage {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        match self {
            Self::Value(variable_value) => {
                variable_value.async_debug_format_with_formatter(formatter).await
            }
            Self::Message(variable_actor_message) => {
                format!("{variable_actor_message:?}")
            }
        }
    }
}

// @TODO Don't clone - only weak references
#[derive(Debug, Clone)]
pub struct VariableActor {
    #[allow(dead_code)]
    task_handle: Arc<TaskHandle>,
    message_sender: Arc<mpsc::UnboundedSender<VariableActorMessage>>,
}

impl VariableActor {
    pub fn new(values: impl Future<Output = impl Stream<Item = VariableValue> + 'static> + 'static) -> Self {
        let (message_sender, message_receiver) = mpsc::unbounded::<VariableActorMessage>();

        let task_handle = Task::start_droppable(async move {
            let values = pin!(values.await);
            let mut value = None::<VariableValue>;

            let mut change_senders = Vec::<mpsc::UnboundedSender<VariableValueChanged>>::new();

            let mapped_values = pin!(values.map(VariableActorValueOrMessage::Value).then(|value| async {
                zoon::println!("New actor value: {}", value.async_debug_format().await);
                value
            }));

            let mapped_messages = pin!(message_receiver.map(VariableActorValueOrMessage::Message).then(|message| async {
                // zoon::println!("New actor message: {}", message.async_debug_format().await);
                message
            }));

            let mut values_and_messages = stream::select(
                mapped_values, 
                mapped_messages, 
            );

            let mut value_getters = Vec::<oneshot::Sender<VariableValue>>::new();

            let set_value = |
                old_value: &mut Option<VariableValue>, 
                new_value: VariableValue,
                change_senders: &mut Vec<mpsc::UnboundedSender<VariableValueChanged>>,
            | {
                *old_value = Some(new_value);
                // change_senders.retain(|change_sender| {
                //     change_sender.unbounded_send(VariableValueChanged).is_ok()
                // });
                zoon::println!("change_senders count: {}", change_senders.len());
                change_senders.retain(|change_sender| {
                    change_sender.unbounded_send(VariableValueChanged).unwrap();
                    true
                });
            };

            while let Some(value_or_message) = values_and_messages.next().await {
                match value_or_message {
                    VariableActorValueOrMessage::Value(new_value) => {
                        // @TODO move to `set_value` closure?
                        for value_getter in value_getters.drain(..) {
                            let _ = value_getter.send(new_value.clone());
                        };
                        set_value(&mut value, new_value, &mut change_senders);
                    }
                    VariableActorValueOrMessage::Message(message) => {
                        match message {
                            VariableActorMessage::GetValue { value_sender } => {
                                if let Some(value) = value.as_ref() {
                                    value_sender.send(value.clone()).unwrap();
                                } else {
                                    value_getters.push(value_sender);
                                }
                            }
                            // @TODO remove?
                            // VariableActorMessage::SetValue { new_value } => {
                            //     set_value(&mut value, new_value, &mut change_senders);
                            // }
                            VariableActorMessage::ValueChanges { change_sender } => {
                                if change_sender.unbounded_send(VariableValueChanged).is_ok() {
                                    change_senders.push(change_sender);
                                }
                                // change_sender.unbounded_send(VariableValueChanged).unwrap();
                                // change_senders.push(change_sender);
                            }
                        }
                    }
                }
            }
        });
        Self {
            task_handle: Arc::new(task_handle),
            message_sender: Arc::new(message_sender)
        }
    }

    pub async fn get_value(&self) -> VariableValue {
        let (value_sender, value_receiver) = oneshot::channel();
        let message = VariableActorMessage::GetValue { value_sender };
        self.message_sender.unbounded_send(message).unwrap();
        value_receiver.await.unwrap()
    }

    // @TODO remove?
    // pub fn set_value(&self, new_value: VariableValue) {
    //     let message = VariableActorMessage::SetValue { new_value };
    //     self.message_sender.unbounded_send(message).unwrap()
    // }

    pub fn value_changes(&self) -> mpsc::UnboundedReceiver<VariableValueChanged> {
        let (change_sender, change_receiver) = mpsc::unbounded();
        let message = VariableActorMessage::ValueChanges { change_sender };
        self.message_sender.unbounded_send(message).unwrap();
        zoon::println!("value_changes called");
        change_receiver
    }
}

pub struct VariableValueChanged;

impl AsyncDebugFormat for VariableActor {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        self.get_value().await.async_debug_format_with_formatter(formatter).await
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VariableName(Arc<String>);

impl VariableName {
    pub fn new(name: impl ToString) -> Self {
        Self(Arc::new(name.to_string()))
    }
}

#[derive(Debug, Clone)]
pub enum VariableValue {
    Link(VariableValueLink),
    List(VariableValueList),
    // @TODO remove `allow`
    #[allow(dead_code)]
    Map(VariableValueMap),
    Number(VariableValueNumber),
    Object(VariableValueObject),
    TaggedObject(VariableValueTaggedObject),
    Tag(VariableValueTag),
    Text(VariableValueText),
    Unset,
}

impl AsyncDebugFormat for VariableValue {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        match self {
            Self::Link(value) => value.async_debug_format_with_formatter(formatter).await,
            Self::List(value) => value.async_debug_format_with_formatter(formatter).await,
            Self::Map(value) => value.async_debug_format_with_formatter(formatter).await,
            Self::Number(value) => value.async_debug_format_with_formatter(formatter).await,
            Self::Object(value) => value.async_debug_format_with_formatter(formatter).await,
            Self::TaggedObject(value) => value.async_debug_format_with_formatter(formatter).await,
            Self::Tag(value) => value.async_debug_format_with_formatter(formatter).await,
            Self::Text(value) => value.async_debug_format_with_formatter(formatter).await,
            Self::Unset => String::from("UNSET"),
        }
    }
}

// --- VariableValueLink ---

#[derive(Debug, Clone)]
pub struct VariableValueLink {
    link_actor: VariableActor,
    actor_sender: Arc<mpsc::UnboundedSender<VariableActor>>,
}

impl VariableValueLink {
    pub fn new() -> Self {
        let (actor_sender, actor_receiver) = mpsc::unbounded::<VariableActor>();

        let value_stream = actor_receiver.flat_map(|actor| {
            let value_changes_receiver = actor.value_changes();
            value_changes_receiver.then({
                let actor = actor.clone();
                move |_change| { 
                    let actor = actor.clone();
                    async move { 
                        zoon::println!("KKKKK");
                        // zoon::println!("Link_actor: {}", actor.async_debug_format().await);
                        // let value = actor.get_value().await;
                        // zoon::println!("LLLL");
                        // zoon::println!("Link_actor new value: {}", value.async_debug_format().await);
                        // value

                        VariableValue::Object(VariableValueObject::new(Variables::new()))
                    }
                }
            })
        });

        let link_actor = VariableActor::new(async { 
            // stream::once( async { VariableValue::Unset }).chain(value_stream) 
            value_stream
        });

        Self { 
            link_actor,
            actor_sender: Arc::new(actor_sender)
        }
    }

    pub fn link_actor(&self) -> VariableActor {
        self.link_actor.clone()
    }

    pub fn set_target(&self, target_actor: VariableActor) {
        self.actor_sender.unbounded_send(target_actor).unwrap();
    }
}

impl AsyncDebugFormat for VariableValueLink {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        let link_actor = Box::pin(self.link_actor.async_debug_format_with_formatter(formatter)).await;
        format!("LINK {{ {link_actor} }}")
    }
}

// --- VariableValueList ---

#[derive(Debug, Clone)]
pub struct VariableValueList {
    list: Vec<VariableActor>
}

impl VariableValueList {
    pub fn new(list: Vec<VariableActor>) -> Self {
        Self { list }
    }

    pub fn list(&self) -> &[VariableActor] {
        &self.list
    }
}

impl AsyncDebugFormat for VariableValueList {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        if self.list.is_empty() {
            return String::from("LIST {}")
        }
        let mut output = String::new();
        output.push_str("LIST {\n");

        let values = {
            let formatter = formatter.increase_level();
            self.list.iter().map(move |value| async move {
                formatter.indent(&value.async_debug_format_with_formatter(formatter).await)
            })
        };
        output.push_str(&join_all(values).await.join("\n"));
        
        output.push_str("\n");
        output.push_str(&formatter.indent("}"));
        output
    }
}

// --- VariableValueMap ---

#[derive(Debug, Clone)]
pub struct VariableValueMap {

}

impl AsyncDebugFormat for VariableValueMap {
    async fn async_debug_format_with_formatter(&self, _: Formatter) -> String {
        String::from("MAP { @TODO }")
    }
}

// --- VariableValueNumber ---

#[derive(Debug, Clone)]
pub struct VariableValueNumber {
    number: f64
}

impl VariableValueNumber {
    pub fn new(number: f64) -> Self {
        Self { number }
    }

    pub fn number(&self) -> f64 {
        self.number
    }
}

impl AsyncDebugFormat for VariableValueNumber {
    async fn async_debug_format_with_formatter(&self, _: Formatter) -> String {
        self.number.to_string()
    }
}

// --- VariableValueObject ---

#[derive(Debug, Clone)]
pub struct VariableValueObject {
    variables: Variables
}

impl VariableValueObject {
    pub fn new(variables: Variables) -> Self {
        Self { variables }
    }

    pub fn variable(&self, variable_name: &VariableName) -> Option<&Variable> {
        self.variables.get(variable_name)
    }

    pub fn into_variables(self) -> Variables {
        self.variables
    }
}

impl AsyncDebugFormat for VariableValueObject {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        if self.variables.is_empty() {
            return String::from("[]")
        }
        let mut output = String::new();
        output.push_str("[\n");

        let variables = { 
            let formatter = formatter.increase_level();
            self.variables.values().map(move |variable| async move {
                formatter.indent(&variable.async_debug_format_with_formatter(formatter).await)
            })
        };
        output.push_str(&join_all(variables).await.join("\n"));
        output.push_str("\n");
        output.push_str(&formatter.indent("]"));
        output
    }
}

// --- VariableValueTaggedObject ---

#[derive(Debug, Clone)]
pub struct VariableValueTaggedObject {
    tag: String,
    variables: Variables
}

impl VariableValueTaggedObject {
    pub fn new(tag: impl ToString, variables: Variables) -> Self {
        Self { tag: tag.to_string(), variables }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn variable(&self, variable_name: &VariableName) -> Option<&Variable> {
        self.variables.get(variable_name)
    }

    pub fn into_variables(self) -> Variables {
        self.variables
    }
}

impl AsyncDebugFormat for VariableValueTaggedObject {
    async fn async_debug_format_with_formatter(&self, formatter: Formatter) -> String {
        let tag = &self.tag;

        if self.variables.is_empty() {
            return format!("{tag}[]")
        }

        let mut output = String::new();
        output.push_str(&format!("{tag}[\n"));

        let variables = { 
            let formatter = formatter.increase_level();
            self.variables.values().map(move |variable| async move {
                let variable = variable.async_debug_format_with_formatter(formatter).await;
                formatter.indent(&variable)
            })
        };
        output.push_str(&join_all(variables).await.join("\n"));
        output.push_str("\n");
        output.push_str(&formatter.indent("]"));
        output
    }
}

// --- VariableValueTag ---

#[derive(Debug, Clone)]
pub struct VariableValueTag {
    tag: String
}

impl VariableValueTag {
    pub fn new(tag: impl ToString) -> Self {
        Self { tag: tag.to_string() }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }
}

impl AsyncDebugFormat for VariableValueTag {
    async fn async_debug_format_with_formatter(&self, _: Formatter) -> String {
        self.tag.clone()
    }
}

// --- VariableValueText ---

#[derive(Debug, Clone)]
pub struct VariableValueText {
    text: String
}

impl VariableValueText {
    pub fn new(text: impl ToString) -> Self {
        Self { text: text.to_string() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl AsyncDebugFormat for VariableValueText {
    async fn async_debug_format_with_formatter(&self, _: Formatter) -> String {
        let text = &self.text;
        format!("'{text}'")
    }
}