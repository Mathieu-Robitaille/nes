use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, SerializeStruct, Serializer},
};

use crate::instructions::instruction::Instruction;

impl Serialize for Instruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Instruction", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("addr_mode", &self.name)?;
        state.serialize_field("function", &self.name)?;
        state.serialize_field("clock_cycles", &self.name)?;
        state.end()
    }
}
