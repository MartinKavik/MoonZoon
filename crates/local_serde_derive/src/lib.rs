mod attributes;
mod local_deserialize;
mod local_serialize;

use proc_macro::TokenStream;

#[proc_macro_derive(LocalSerialize, attributes(local_serde))]
pub fn derive_local_serialize(input: TokenStream) -> TokenStream {
    local_serialize::derive_local_serialize(input)
}

#[proc_macro_derive(LocalDeserialize, attributes(local_serde))]
pub fn derive_local_deserialize(input: TokenStream) -> TokenStream {
    local_deserialize::derive_local_deserialize(input)
}

