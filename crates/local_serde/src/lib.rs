pub use serde_lite::{self, Deserialize, Intermediate, Serialize, Error};

pub trait LocalSerialize<L> {
    fn local_serialize(&self) -> Result<Intermediate, Error>;
}

pub trait LocalDeserialize<L> {
    fn local_deserialize(intermediate: &Intermediate) -> Result<Self, Error> where Self: Sized;
}

#[macro_export]
macro_rules! init_local_serde {
    ($local_type:ty) => {
        // impl<T: Serialize> LocalSerialize<$local_type> for T {
        //     fn local_serialize(&self) -> Result<Intermediate, Error> {
        //         self.serialize()
        //     }
        // }
    }
}
