use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io, path::Path};

#[derive(Debug, Serialize, Deserialize)]
pub struct PacketSet {
    pub packets: Vec<Packet>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Packet {
    pub name: String,
    pub description: String,
    pub id: u16,
    pub data: Vec<Field>,
}

impl Packet {
    pub fn id(p: Packet) -> Packet {
        p
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Field {
    // NB: serde tries to deser enums in order, so Bits should be first b/c it can be a superset of
    // Plain.
    Bits {
        name: String,
        bits: Vec<BitFieldItem>,
    },
    Plain {
        name: String,
        #[serde(rename = "type")]
        typ: String,
        #[serde(default)]
        units: String,
    },
}

impl Default for Field {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BitFieldItem {
    pub name: String,
    #[serde(default = "default_bitnum")]
    pub bitnum: u8,
    #[serde(default)]
    pub values: BTreeMap<u8, String>, // TODO(aptny): check value bounds, fixed size array
}

#[inline]
fn default_bitnum() -> u8 {
    1
}

// creatively named error type
#[derive(Debug)]
pub enum ModelError {
    Io(io::Error),
    Deser(serde_yaml::Error),
}

impl From<io::Error> for ModelError {
    #[inline]
    fn from(err: io::Error) -> ModelError {
        ModelError::Io(err)
    }
}

impl From<serde_yaml::Error> for ModelError {
    #[inline]
    fn from(err: serde_yaml::Error) -> ModelError {
        ModelError::Deser(err)
    }
}

// backing file, data, needs update
#[derive(Debug)]
pub struct Model(pub Vec<(File, PacketSet, bool)>);
type Result<T> = std::result::Result<T, ModelError>;

impl Model {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn new_with_backing(path: &Path) -> Result<Self> {
        let mut mdl = Self::new();
        mdl.add_backing(path)?;

        Ok(mdl)
    }

    pub fn add_backing(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let pkl: PacketSet = serde_yaml::from_reader(&file)?;

        self.0.push((file, pkl, false));

        Ok(())
    }

    pub fn persist(&mut self) -> Result<()> {
        for doc in &mut self.0 {
            if !doc.2 {
                continue;
            }

            serde_yaml::to_writer(&doc.0, &doc.1)?;
            doc.2 = false;
            break;
        }

        Ok(())
    }
}

// TODO(aptny): test for correctness or remove
#[cfg(test)]
mod tests {
    use super::{Packet, PacketSet};

    #[test]
    fn test_de() {
        let r = include_str!("../data/switchboard.yaml");
        let pks: PacketSet = serde_yaml::from_str(r).unwrap();
        println!("{:?}", pks);
    }
}
