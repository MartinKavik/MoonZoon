use zoon::{*, println};
use zoon::futures_util::stream;
use std::sync::Arc;

mod engine;
use engine::*;

type ArgumentName = &'static str;

// @TODO ObjectActor - value = stream of Vec<PropertyName> or Vec<VariableActor> ?
// @TODO ObjectActor, VariableActor and others (?) - add input parameter stream?

pub async fn run(_program: &str) -> impl Element {

    let document_new_function_definition = |arguments_actor: ArgumentsActor, _passed_actor: PassedActor| {
        let document_new_output_object_actor_32 = ObjectActor::new("Document/new output object", 32);
        let document_new_output_root_element_variable_actor_33 = VariableActor::new("Document/new output root_element", 33, "root_element");
        document_new_output_root_element_variable_actor_33.set_variable_actor_as_value(arguments_actor.argument_to_variable_actor("root"));
        document_new_output_object_actor_32.insert_variable_actor_as_property(document_new_output_root_element_variable_actor_33);
        stream::once(future::ready(document_new_output_object_actor_32))
    };

    let element_stripe_function_definition = |arguments_actor: ArgumentsActor, _passed_actor: PassedActor| {
        let element_actor_stream = arguments_actor.actor_stream("element");
        let direction_actor_stream = arguments_actor.actor_stream("direction");
        let style_actor_stream = arguments_actor.actor_stream("style");
        let items_actor_stream = arguments_actor.actor_stream("items");

        let element_stripe_output_object_actor_34 = ObjectActor::new("Element/stripe output object", 34);

        let element_stripe_output_settings_variable_actor_35 = VariableActor::new("Element/stripe output settings", 35, "settings");
        element_stripe_output_object_actor_34.insert_variable_actor_as_property(element_stripe_output_settings_variable_actor_35);

        let element_stripe_output_type_variable_output_36 = VariableActor::new("Element/stripe output type", 36, "type");
        element_stripe_output_object_actor_34.insert_variable_actor_as_property(element_stripe_output_type_variable_output_36);

        let element_stripe_output_type_tag_actor_37 = TagActor::new("Element/stripe output type tag", 37, stream::once(future::ready("Stripe")));
        element_stripe_output_type_variable_output_36.set_tag_actor_as_value(element_stripe_output_type_tag_actor_37);

        stream::once(future::ready(element_stripe_output_object_actor_34))
    };

    let element_button_function_definition = |arguments_actor: ArgumentsActor, _passed_actor: PassedActor| {
        let element_button_output_object_actor_38 = ObjectActor::new("Element/button output object", 38);
        element_button_output_object_actor_38.insert_variable_actors_as_properties(
            arguments_actor.argument_to_variable_actor("element").actor_stream().flat_map(|element_actor| {
                element_actor.unwrap_object_actor().property_stream()
            })
        );

        let element_button_output_settings_variable_actor_39 = VariableActor::new("Element/button output settings", 39, "settings");
        
        let element_button_output_settings_object_actor_42 = ObjectActor::new("Element/button output settings object", 42);
        element_button_output_settings_object_actor_42.insert_variable_actors_as_properties(stream::once(future::ready(arguments_actor.argument_to_variable_actor("style"))));
        element_button_output_settings_object_actor_42.insert_variable_actors_as_properties(stream::once(future::ready(arguments_actor.argument_to_variable_actor("label"))));
        element_button_output_settings_variable_actor_39.set_object_actor_as_value(element_button_output_settings_object_actor_42);
        
        element_button_output_object_actor_38.insert_variable_actors_as_properties(stream::once(future::ready(element_button_output_settings_variable_actor_39)));

        let element_button_output_type_variable_output_40 = VariableActor::new("Element/button output type", 40, "type");
        element_button_output_object_actor_38.insert_variable_actors_as_properties(stream::once(future::ready(element_button_output_type_variable_output_40)));

        let element_button_output_type_tag_actor_41 = TagActor::new("Element/button output type tag", 41, stream::once(future::ready("Button")));
        element_button_output_type_variable_output_40.set_tag_actor_as_value(element_button_output_type_tag_actor_41);

        stream::once(future::ready(element_button_output_object_actor_38))
    };

    let math_sum_function_definition = |arguments_actor: ArgumentsActor, _passed_actor: PassedActor| async {
        let counter_default_number_actor_43 = NumberActor::new(
            "counter default number", 
            43,
            arguments_actor
                .argument_to_variable_actor("increment")
                .actor_stream()
                .flat_map(|increment_actor| {
                    increment_actor.unwrap_number_actor().number_stream()
                })
                .scan(0, |state, increment| {
                    *state += increment;
                    future::ready(Some(*state))
                })
        );
        stream::once(future::ready(counter_default_number_actor_43))
    };


    let root_root_actor_0 = RootActor::new("root", 0);


    let document_variable_actor_1 = VariableActor::new("document", 1, "document");

    let document_new_function_actor_2 = FunctionActor::new("Document/new call", 2, "Document/new", document_new_function_definition);

    let root_argument_actor_3 = ArgumentActor::new("Document/new root", 3, "root");

    let element_stripe_function_actor_4 = FunctionActor::new("Element/stripe call", 4, "Element/stripe", element_stripe_function_definition);

    let element_stripe_element_argument_actor_5 = ArgumentActor::new("Element/stripe element", 5, "element");

    let element_stripe_element_object_actor_6 = ObjectActor::new("Element/stripe element object", 6);

    let element_stripe_direction_argument_actor_7 = ArgumentActor::new("Element/stripe direction", 7, "direction");

    let element_stripe_direction_tag_actor_8 = TagActor::new("Element/stripe direction tag", 8, "Column");

    let element_stripe_style_argument_actor_9 = ArgumentActor::new("Element/stripe style", 9, "style");

    let element_stripe_style_object_actor_10 = ObjectActor::new("Element/stripe style object", 10);

    let element_stripe_items_argument_actor_11 = ArgumentActor::new("Element/stripe items", 11, "items");

    let element_stripe_items_list_actor_12 = ListActor::new("Element/stripe items list", 12);


    let counter_variable_actor_13 = VariableActor::new("counter", 13, "counter");

    let counter_latest_actor_14 = LatestActor::new("counter latest", 14);

    let counter_default_number_actor_15 = NumberActor::new("counter default number", 15, stream::once(future::ready(0.)));

    let counter_button_press_reference_actor_16 = ReferenceActor::new("counter button press", 16, "increment_button.event.press");

    let counter_button_press_then_actor_17 = ThenActor::new("counter button press then", 17);

    let counter_after_button_press_number_actor_18 = NumberActor::new("counter after button press number", 18, stream::once(future::ready(1.)));

    let counter_math_sum_function_actor_19 = FunctionActor::new("Math/sum call", 19, "Math/sum", math_sum_function_definition);

    let counter_math_sum_increment_argument_actor_42 = ArgumentActor::new("Math/sum increment", 42, "increment");


    let increment_button_variable_actor_20 = VariableActor::new("increment_button", 20, "increment_button");

    let element_button_function_actor_21 = FunctionActor::new("Element/button call", 21, "Element/button", element_button_function_definition);

    let element_button_element_argument_actor_22 = ArgumentActor::new("Element/button element", 22, "element");

    let element_button_element_object_actor_23 = ObjectActor::new("Element/button element object", 23);

    let element_button_event_variable_actor_24 = VariableActor::new("Element/button element event", 24, "event");

    let element_button_event_object_actor_25 = ObjectActor::new("Element/button element event object", 25);

    let element_button_event_press_variable_actor_26 = VariableActor::new("Element/button element event press", 26, "press");

    let element_button_event_press_link_actor_27 = LinkActor::new("Element/button element event press link", 27);

    let element_button_style_argument_actor_28 = ArgumentActor::new("Element/button style", 28, "style");

    let element_button_style_object_actor_29 = ObjectActor::new("Element/button style object", 29);

    let element_button_label_argument_actor_30 = ArgumentActor::new("Element/button label", 30, "label");

    let element_button_label_text_actor_31 = TextActor::new("Element/button label text", 31, stream::once(future::ready("+")));

    El::new().child("3. attempt")
}
