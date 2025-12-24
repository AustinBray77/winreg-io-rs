/*
 *  writer.rs
 *
 *  Description: This module contains code for writing in .reg files from the windows registry
 *  (via the RegWriter struct) and util functions for translating between winreg and regashii values.
 *
 *  Created By: AustinBray77
 */

use std::{cmp::min, error::Error, path::Path};

use regashii::{Key, KeyName, Registry, Value, ValueName};
use winreg::{RegKey, RegValue};

fn u8s_to_u32(vec: Vec<u8>) -> u32 {
    let mut output: u32 = 0_u32;

    for i in 0..(min(vec.len(), 4)) {
        output += (vec[0] as u32) << (8 * (3 - i));
    }

    output
}

fn u8s_to_u64(vec: Vec<u8>) -> u64 {
    let mut output: u64 = 0_64;

    for i in 0..(min(vec.len(), 8)) {
        output += (vec[0] as u64) << (8 * (7 - i));
    }

    output
}

fn winreg_to_regashii(value: RegValue) -> Option<Value> {
    use regashii::Value::*;
    use winreg::enums::RegType::*;

    match value.vtype {
        REG_SZ => Some(Sz(value.to_string())),
        REG_EXPAND_SZ => Some(ExpandSz(value.to_string())),
        REG_NONE => None,
        REG_BINARY => Some(Binary(value.bytes)),
        REG_DWORD => Some(Dword(u8s_to_u32(value.bytes))),
        REG_DWORD_BIG_ENDIAN => Some(DwordBigEndian(u8s_to_u32(value.bytes))),
        REG_LINK => panic!("Link values are not supported!"),
        REG_MULTI_SZ => panic!("Multi sz are not supported!"),
        REG_RESOURCE_LIST => panic!("Resource lists are not supported!"),
        REG_FULL_RESOURCE_DESCRIPTOR => panic!("Full resource descriptors are not supported!"),
        REG_RESOURCE_REQUIREMENTS_LIST => panic!("Resource requirements lists are not supported!"),
        REG_QWORD => Some(Qword(u8s_to_u64(value.bytes))),
    }
}

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
                    if key.is_empty() {
                        None
                    } else {
                        Some(key)
                    }
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
