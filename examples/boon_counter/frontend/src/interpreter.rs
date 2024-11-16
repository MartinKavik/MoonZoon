use zoon::{*, println};
use std::sync::{Arc, RwLock};

mod engine;
use engine::*;

pub async fn run(_program: &str) -> impl Element {
    let mut engine = Arc::new(RwLock::new(Engine::default()));

    let function_name = FunctionName::new("Element/stripe");
    let function_closure = |function_arguments: Arguments| {
        VariableActor::new(Some(VariableValue::Object(VariableValueObject::new({
            let mut variables = Variables::new();
            
            let variable_name = VariableName::new("element");
            let variable = Variable::new(
                variable_name.clone(),
                VariableActor::new(Some(VariableValue::Object(VariableValueObject::new(Variables::new()))))
            );
            function_arguments
                .get(&ArgumentName::new("element"))
                .unwrap()
                .argument_out()
                .unwrap()
                .send_actor(variable.actor());
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("direction");
            let variable = Variable::new(
                variable_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("direction"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("gap");
            let variable = Variable::new(
                variable_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("gap"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("style");
            let variable = Variable::new(
                variable_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("style"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("items");
            let variable = Variable::new(
                variable_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("items"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("extra");
            let variable = Variable::new(
                variable_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("extra"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            variables.insert(variable_name, variable);

            variables
        }))))
    };
    let function = Function::new(function_name.clone(), function_closure);
    engine.write().unwrap().functions.insert(function_name, function);

    let function_name = FunctionName::new("root_element");
    let function_closure = {
        let engine = engine.clone();
        move |function_arguments: Arguments| {
            let mut arguments = Arguments::new();
            
            let argument_name = ArgumentName::new("element");
            let (argument, element_kind_receiver) = Argument::new_out(
                argument_name.clone(),
            );
            arguments.insert(argument_name, argument);

            let argument_name = ArgumentName::new("direction");
            let argument = Argument::new_in(
                argument_name.clone(),
                VariableActor::new(Some(VariableValue::Tag(VariableValueTag::new("Row")))),
            );
            arguments.insert(argument_name, argument);

            let argument_name = ArgumentName::new("gap");
            let argument = Argument::new_in(
                argument_name.clone(),
                VariableActor::new(Some(VariableValue::Number(VariableValueNumber::new(15.)))),
            );
            arguments.insert(argument_name, argument);

            let argument_name = ArgumentName::new("style");
            let argument = Argument::new_in(
                argument_name.clone(),
                VariableActor::new(Some(VariableValue::Object(VariableValueObject::new({
                    let mut variables = Variables::new();

                    let variable_name = VariableName::new("align");
                    let variable = Variable::new(
                        variable_name.clone(),
                        VariableActor::new(Some(VariableValue::Tag(VariableValueTag::new("Center"))))
                    );
                    variables.insert(variable_name, variable);

                    variables
                })))),
            );
            arguments.insert(argument_name, argument);

            let argument_name = ArgumentName::new("items");
            let argument = Argument::new_in(
                argument_name.clone(),
                VariableActor::new(Some(VariableValue::List(VariableValueList::new({
                    let mut list = Vec::new();

                    list.push({
                        let mut arguments= Arguments::new();

                        let argument_name = ArgumentName::new("label");
                        let argument = Argument::new_in(
                            argument_name.clone(),
                            VariableActor::new(Some(VariableValue::Text(VariableValueText::new("-")))),
                        );
                        arguments.insert(argument_name, argument);

                       let variable_actor = engine
                            .read()
                            .unwrap()
                            .functions
                            .get(&FunctionName::new("counter_button"))
                            .unwrap()
                            .run(arguments);

                        engine.read().unwrap().set_link_value("elements.decrement_button", variable_actor.clone()).await;

                        variable_actor
                    });

                    list.push({
                        engine
                            .read()
                            .unwrap()
                            .variables
                            .get(&VariableName::new("counter"))
                            .unwrap()
                            .actor()
                    });

                    list.push({
                        let mut arguments= Arguments::new();

                        let argument_name = ArgumentName::new("label");
                        let argument = Argument::new_in(
                            argument_name.clone(),
                            VariableActor::new(Some(VariableValue::Text(VariableValueText::new("+")))),
                        );
                        arguments.insert(argument_name, argument);

                        let variable_actor = engine
                            .read()
                            .unwrap()
                            .functions
                            .get(&FunctionName::new("counter_button"))
                            .unwrap()
                            .run(arguments);

                        engine.read().unwrap().set_link_value("elements.increment_button", variable_actor.clone()).await;

                        variable_actor
                    });

                    list
                })))),
            );
            arguments.insert(argument_name, argument);

            let argument_name = ArgumentName::new("extra");
            let argument = Argument::new_in(
                argument_name.clone(),
                VariableActor::new(Some(VariableValue::Object(VariableValueObject::new(Variables::new())))),
            );
            arguments.insert(argument_name, argument);

            engine.read().unwrap().functions.get(&FunctionName::new("Element/stripe")).unwrap().run(arguments)
        }
    };
    let function = Function::new(function_name.clone(), function_closure);
    engine.write().unwrap().functions.insert(function_name, function);

    let function_name: FunctionName = FunctionName::new("Element/button");
    let function_closure = |function_arguments: Arguments| {
        VariableActor::new(Some(VariableValue::Object(VariableValueObject::new({
            let mut variables = Variables::new();
            
            let variable_name = VariableName::new("element");
            let variable = Variable::new(
                variable_name.clone(),
                VariableActor::new(Some(VariableValue::Object(VariableValueObject::new({
                    let mut variables = Variables::new();

                    let variable_name = VariableName::new("event");
                    let variable = Variable::new(
                        variable_name.clone(),
                        VariableActor::new(Some(VariableValue::Object(VariableValueObject::new({
                            let mut variables = Variables::new();

                            let variable_name = VariableName::new("hovered");
                            let variable = Variable::new(
                                variable_name.clone(),
                                VariableActor::new(Some(VariableValue::Tag(VariableValueTag::new("False"))))
                            );
                            variables.insert(variable_name, variable);

                            variables
                        }))))
                    );
                    variables.insert(variable_name, variable);

                    variables
                }))))
            );
            function_arguments
                .get(&ArgumentName::new("element"))
                .unwrap()
                .argument_out()
                .unwrap()
                .send_actor(variable.actor());
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("style");
            let variable = Variable::new(
                variable_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("style"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("label");
            let variable = Variable::new(
                variable_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("label"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("extra");
            let variable = Variable::new(
                variable_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("extra"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            variables.insert(variable_name, variable);

            variables
        }))))
    };
    let function = Function::new(function_name.clone(), function_closure);
    engine.write().unwrap().functions.insert(function_name, function);

    let function_name: FunctionName = FunctionName::new("counter_button");
    let function_closure = { 
        let engine = engine.clone();
            move |function_arguments: Arguments| {
            let mut arguments = Arguments::new();
            
            let argument_name = ArgumentName::new("element");
            let (argument, element_kind_receiver) = Argument::new_out(
                argument_name.clone(),
            );
            arguments.insert(argument_name, argument);

            let argument_name = ArgumentName::new("style");
            let argument = Argument::new_in(
                argument_name.clone(),
                VariableActor::new(Some(VariableValue::Object(VariableValueObject::new({
                    let mut variables = Variables::new();

                    let variable_name = VariableName::new("width");
                    let variable = Variable::new(
                        variable_name.clone(),
                        VariableActor::new(Some(VariableValue::Number(VariableValueNumber::new(45.))))
                    );
                    variables.insert(variable_name, variable);

                    let variable_name = VariableName::new("rounded_corners");
                    let variable = Variable::new(
                        variable_name.clone(),
                        VariableActor::new(Some(VariableValue::Tag(VariableValueTag::new("Fully"))))
                    );
                    variables.insert(variable_name, variable);

                    let variable_name = VariableName::new("background");
                    let variable = Variable::new(
                        variable_name.clone(),
                        VariableActor::new(Some(VariableValue::Object(VariableValueObject::new({
                            let mut variables = Variables::new();

                            let variable_name = VariableName::new("color");
                            let variable = Variable::new(
                                variable_name.clone(),
                                VariableActor::new(Some(VariableValue::TaggedObject(VariableValueTaggedObject::new("Oklch", {
                                    let mut variables = Variables::new();

                                    let variable_name = VariableName::new("lightness");
                                    let variable = Variable::new(
                                        variable_name.clone(),
                                        // element.hovered |> WHEN { True => 0.85, False => 0.75 }
                                        VariableActor::new(Some(VariableValue::Number(VariableValueNumber::new(0.75))))
                                    );
                                    variables.insert(variable_name, variable);

                                    let variable_name = VariableName::new("chroma");
                                    let variable = Variable::new(
                                        variable_name.clone(),
                                        VariableActor::new(Some(VariableValue::Number(VariableValueNumber::new(0.07))))
                                    );
                                    variables.insert(variable_name, variable);

                                    let variable_name = VariableName::new("hue");
                                    let variable = Variable::new(
                                        variable_name.clone(),
                                        VariableActor::new(Some(VariableValue::Number(VariableValueNumber::new(320.))))
                                    );
                                    variables.insert(variable_name, variable);

                                    variables
                                }))))
                            );
                            variables.insert(variable_name, variable);

                            variables
                        }))))
                    );
                    variables.insert(variable_name, variable);

                    variables
                }))))
            );
            arguments.insert(argument_name, argument);

            let argument_name = ArgumentName::new("label");
            let argument = Argument::new_in(
                argument_name.clone(),
                function_arguments
                    .get(&ArgumentName::new("label"))
                    .unwrap()
                    .argument_in()
                    .unwrap()
                    .actor()
            );
            arguments.insert(argument_name, argument);

            let argument_name = ArgumentName::new("extra");
            let argument = Argument::new_in(
                argument_name.clone(),
                VariableActor::new(Some(VariableValue::Object(VariableValueObject::new(Variables::new()))))
            );
            arguments.insert(argument_name, argument);

            engine.read().unwrap().functions.get(&FunctionName::new("Element/button")).unwrap().run(arguments)
        }
    };
    let function = Function::new(function_name.clone(), function_closure);
    engine.write().unwrap().functions.insert(function_name, function);

    let variable_name = VariableName::new("elements");
    let variable = Variable::new(
        variable_name.clone(),
        VariableActor::new(Some(VariableValue::Object(VariableValueObject::new({
            let mut variables = Variables::new();
            
            let variable_name = VariableName::new("decrement_button");
            let variable = Variable::new(
                variable_name.clone(),
                VariableActor::new(Some(VariableValue::Link(VariableValueLink::new())))
            );
            variables.insert(variable_name, variable);

            let variable_name = VariableName::new("increment_button");
            let variable = Variable::new(
                variable_name.clone(),
                VariableActor::new(Some(VariableValue::Link(VariableValueLink::new())))
            );
            variables.insert(variable_name, variable);

            variables
        }))))
    );
    engine.write().unwrap().variables.insert(variable_name, variable);

    let variable_name = VariableName::new("counter");
    let variable = Variable::new(
        variable_name.clone(),
        VariableActor::new(Some(VariableValue::Number(VariableValueNumber::new(6.))))
    );
    engine.write().unwrap().variables.insert(variable_name, variable);

    let variable_name = VariableName::new("document");
    let variable = Variable::new(
        variable_name.clone(),
        engine.read().unwrap().functions.get(&FunctionName::new("root_element")).unwrap().run(Arguments::new()),
    );
    engine.write().unwrap().variables.insert(variable_name, variable);

    println!("{}", engine.read().unwrap().async_debug_format().await);

    El::new().child("Boon root")
} 
