use model_macro::TreeNode;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeMap, fs::File, io, path::Path};

extern crate self as _common;

#[derive(Debug, Serialize, Deserialize, TreeNode)]
#[tm(item = "Packet")]
pub struct PacketSet {
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    #[tm(skip)]
    pub file: Option<File>,

    #[tm(children)]
    pub packets: Vec<Packet>,
}

#[derive(Serialize, Deserialize, Default, TreeNode)]
#[tm(item = "Field")]
pub struct Packet {
    pub name: String,
    pub id: u16,
    pub description: String,

    #[tm(children)]
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

#[derive(Debug, Serialize, Deserialize, TreeNode)]
#[serde(untagged)]
#[tm(item = "BitFieldItem")]
pub enum Field {
    // NB: serde tries to deser enums in order, so Bits should be first b/c it can be a superset of
    // Plain.
    Bits {
        name: String,
        #[serde(skip)]
        #[tm(skip)]
        typ: String,
        #[serde(skip)]
        #[tm(skip)]
        units: String,
        #[tm(children)]
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

#[derive(Debug, Serialize, Deserialize, TreeNode)]
#[tm(item = "()")]
pub struct BitFieldItem {
    pub name: String,
    #[serde(default = "default_bitnum")]
    pub bitnum: u8,
    #[serde(default)]
    #[tm(skip)]
    pub values: BTreeMap<u8, String>, // TODO(aptny): check value bounds, fixed size array
}

#[inline]
fn default_bitnum() -> u8 {
    1
}

use anyhow::{anyhow, Context, Result};
use io::Read;

#[derive(Debug, TreeNode)]
#[tm(item = "PacketSet")]
#[tm(columns("Name", "Type", "Units"))]
pub struct Model {
    #[tm(children)]
    pub sets: Vec<PacketSet>,
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    pub fn new() -> Self {
        Model { sets: Vec::new() }
    }

    pub fn new_with_backing(path: &Path) -> Result<Self> {
        let mut mdl = Self::new();
        mdl.add_backing(path)?;

        Ok(mdl)
    }

    pub fn add_backing(&mut self, path: &Path) -> Result<()> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // slice out utf-8 BOM
        let sl = if data.len() > 2 && data[0] == 0xef && data[1] == 0xbb && data[2] == 0xbf {
            &data[3..]
        } else {
            &data
        };
        let mut pkl: PacketSet = serde_yaml::from_slice(sl)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        pkl.name = path.file_stem().unwrap().to_string_lossy().into(); // asdfjkl
        pkl.file = Some(file);
        self.sets.push(pkl);

        Ok(())
    }

    pub fn persist(&self) -> Result<()> {
        for doc in &self.sets {
            serde_yaml::to_writer(
                doc.file
                    .as_ref()
                    .ok_or_else(|| anyhow!("file {}.yaml doesn't exist", doc.name))?,
                &doc.packets,
            )?;
        }

        Ok(())
    }
}

// TODO(aptny): test on whole set
#[cfg(test)]
mod tests {
    use super::{Packet, PacketSet};

    #[test]
    fn test_de() {
        let r = include_str!("../../data/switchboard.yaml");
        let pks: PacketSet = serde_yaml::from_str(r).unwrap();
        println!("{:?}", pks);
    }
}
