/*
 *  writer.rs
 *
 *  Description: This module contains code for writing in .reg files from the windows registry
 *  (via the RegWriter struct) and util functions for translating between winreg and regashii values.
 *
 *  Created By: AustinBray77
 */

use std::{error::Error, path::Path};

use regashii::{Key, KeyName, Registry, ValueName};
use winreg::RegKey;

use crate::parsing::*;

#[derive(Debug)]
pub struct RegWriter {
    reg: Registry,
}

impl RegWriter {
    pub fn new() -> Self {
        Self {
            reg: Registry::new(regashii::Format::Regedit5),
        }
    }

    fn with_key(mut self, name: impl Into<KeyName>, key: Key) -> Self {
        self.reg = self.reg.with(name, key);
        self
    }

    pub fn with_all_subkeys(
        mut self,
        regkey: RegKey,
        name: String,
    ) -> Result<Self, Box<dyn Error>> {
        let curkey: Key = regkey
            .enum_values()
            .filter_map(|x| if let Ok(item) = x { Some(item) } else { None })
            .fold(Key::new(), |acc, (key, value)| {
                if let Some(regash_val) = winreg_to_regashii(value) {
                    if key.is_empty() {
                        acc.with(ValueName::Default, regash_val)
                    } else {
                        acc.with(ValueName::Named(key), regash_val)
                    }
                } else {
                    acc
                }
            });

        self = self.with_key(name.clone(), curkey);

        Ok(regkey
            .enum_keys()
            .filter_map(|x: Result<String, std::io::Error>| {
                if let Ok(key) = x {
                    if key.is_empty() { None } else { Some(key) }
                } else {
                    None
                }
            })
            .fold(
                Ok(self),
                |acc: Result<RegWriter, Box<dyn Error>>, val: String| {
                    let subkey = regkey.open_subkey(&val)?;
                    let subname = name.clone() + "\\" + &val;
                    Ok(acc?.with_all_subkeys(subkey, subname)?)
                },
            )?)
    }

    pub fn write_to<T: AsRef<Path>>(&self, path: T) -> Result<(), Box<dyn Error>> {
        Ok(self.reg.serialize_file(path)?)
    }
}
