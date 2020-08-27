use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeMap, fs::File, io, path::Path};

#[derive(Debug, Serialize, Deserialize)]
pub struct PacketSet {
    pub packets: Vec<Packet>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Packet {
    pub name: String,
    pub id: u16,
    pub description: String,
    pub data: Vec<Field>,
}

impl std::fmt::Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Packet")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("id", &self.id)
            .field("data", &self.data)
            .finish()
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
        #[serde(deserialize_with = "de_null_string")]
        units: String,
    },
}

fn de_null_string<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let r: Option<String> = Deserialize::deserialize(d)?;
    Ok(r.unwrap_or(String::new()))
}

impl Default for Field {
    fn default() -> Self {
        unimplemented!()
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

use anyhow::{Context, Result};
use io::Read;
// backing file, data, needs update
#[derive(Debug)]
pub struct Model(pub Vec<(File, PacketSet, bool)>);
//type Result<T> = std::result::Result<T, ModelError>;

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
        println!("mdl on {:?}", path);
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // slice out utf-8 BOM
        let sl = if data.len() > 2 && data[0] == 0xef && data[1] == 0xbb && data[2] == 0xbf {
            &data[3..]
        } else {
            &data
        };
        let pkl: PacketSet = serde_yaml::from_slice(sl)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        self.0.push((file, pkl, false));

        Ok(())
    }

    pub fn persist(&self) -> Result<()> {
        for doc in &self.0 {
            if !doc.2 {
                //continue;
            }

            serde_yaml::to_writer(&doc.0, &doc.1)?;
            //doc.2 = false;
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
