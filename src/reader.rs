/*
 *  reader.rs
 *
 *  Description: This module contains code for reading in .reg files to the windows registry
 *  (via the RegReader struct) and util functions for translating between regashii and winreg values.
 *
 *  Created By: AustinBray77
 */

use std::{error::Error, path::Path};

use regashii::{KeyName, Kind, Registry};
use winreg::{
    enums::{
        RegType::{self},
        HKEY_CLASSES_ROOT,
    },
    transaction::Transaction,
    RegKey, RegValue, HKEY,
};

use crate::NormalResult;

const U32SIZE: usize = 4;
const U64SIZE: usize = 8;

fn u32_to_u8s(val: u32) -> [u8; U32SIZE] {
    let mut output = [0_u8; U32SIZE];

    let mut mask = 1_u32 << 8 - 1;

    for i in 0..U32SIZE {
        output[i] = ((val & mask) >> (i * 8)) as u8;
        mask <<= 8;
    }

    output
}

fn u64_to_u8s(val: u64) -> [u8; U64SIZE] {
    let mut output = [0_u8; U64SIZE];

    let mut mask = 1_u64 << 8 - 1;

    for i in 0..U64SIZE {
        output[i] = ((val & mask) >> (i * 8)) as u8;
        mask <<= 8;
    }

    output
}

fn kind_to_regtype(kind: Kind) -> Option<RegType> {
    use winreg::enums::RegType::*;
    match kind {
        Kind::None => None,
        Kind::Sz => Some(REG_SZ),
        Kind::ExpandSz => Some(REG_EXPAND_SZ),
        Kind::Binary => Some(REG_BINARY),
        Kind::Dword => Some(REG_DWORD),
        Kind::DwordBigEndian => Some(REG_DWORD_BIG_ENDIAN),
        Kind::Link => Some(REG_LINK),
        Kind::MultiSz => Some(REG_MULTI_SZ),
        Kind::ResourceList => Some(REG_RESOURCE_LIST),
        Kind::FullResourceList => Some(REG_FULL_RESOURCE_DESCRIPTOR),
        Kind::ResourceRequirementsList => Some(REG_RESOURCE_REQUIREMENTS_LIST),
        Kind::Qword => Some(REG_QWORD),
        Kind::Unknown(_) => None,
    }
}

fn regasii_to_winreg(value: regashii::Value) -> Option<RegValue> {
    use regashii::Value::*;
    use winreg::enums::RegType::*;

    match value {
        Delete => None,
        Sz(str) => Some(RegValue {
            bytes: str.bytes().collect(),
            vtype: REG_SZ,
        }),
        ExpandSz(str) => Some(RegValue {
            bytes: str.bytes().collect(),
            vtype: REG_EXPAND_SZ,
        }),
        Binary(items) => Some(RegValue {
            bytes: items,
            vtype: REG_BINARY,
        }),
        Dword(val) => Some(RegValue {
            bytes: u32_to_u8s(val).to_vec(),
            vtype: REG_DWORD,
        }),
        DwordBigEndian(val) => Some(RegValue {
            bytes: u32_to_u8s(val).to_vec(),
            vtype: REG_DWORD_BIG_ENDIAN,
        }),
        MultiSz(items) => Some(RegValue {
            bytes: items
                .into_iter()
                .reduce(|acc, next| acc + &next)?
                .bytes()
                .collect(),
            vtype: REG_MULTI_SZ,
        }),
        Qword(val) => Some(RegValue {
            bytes: u64_to_u8s(val).to_vec(),
            vtype: REG_QWORD,
        }),
        Hex { kind, bytes } => Some(RegValue {
            bytes,
            vtype: kind_to_regtype(kind)?,
        }),
    }
}

fn keyname_to_regkey(name: &KeyName, hkey: HKEY) -> Result<RegKey, Box<dyn Error>> {
    Ok(name.raw().split("\\").skip(1).fold(
        Ok(RegKey::predef(hkey)),
        |acc: Result<RegKey, Box<dyn Error>>, next| Ok(acc?.open_subkey(next)?),
    )?)
}

fn keyname_to_regkey_transacted(
    name: &KeyName,
    hkey: HKEY,
    tr: &Transaction,
) -> Result<RegKey, Box<dyn Error>> {
    Ok(name.raw().split("\\").skip(1).fold(
        Ok(RegKey::predef(hkey)),
        |acc: Result<RegKey, Box<dyn Error>>, next| Ok(acc?.open_subkey_transacted(next, tr)?),
    )?)
}
pub struct RegReader {
    reg: Registry,
}

impl RegReader {
    pub fn try_read_file<T: AsRef<Path>>(path: T) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            reg: Registry::deserialize_file(path)?,
        })
    }

    pub unsafe fn load_all_immediate(&self) -> NormalResult {
        self.reg
            .keys()
            .iter()
            .fold(Ok(()), |acc: NormalResult, (name, key)| {
                let regkey = keyname_to_regkey(name, HKEY_CLASSES_ROOT)?;

                key.values()
                    .iter()
                    .fold(Ok(()), |acc: NormalResult, (vname, value)| {
                        if let Some(regvalue) = regasii_to_winreg(value.clone()) {
                            regkey.set_raw_value::<&str>(vname.raw(), &regvalue)?;
                            acc
                        } else {
                            Err(format!("Key has invalid value:{:?}, {:?}", name, vname).into())
                        }
                    })?;

                acc
            })
    }

    pub fn load_all_transacted(&self, tr: &Transaction) -> NormalResult {
        self.reg
            .keys()
            .iter()
            .fold(Ok(()), |acc: NormalResult, (name, key)| {
                let regkey = keyname_to_regkey_transacted(name, HKEY_CLASSES_ROOT, &tr)?;

                key.values()
                    .iter()
                    .fold(Ok(()), |acc: NormalResult, (vname, value)| {
                        if let Some(regvalue) = regasii_to_winreg(value.clone()) {
                            regkey.set_raw_value::<&str>(vname.raw(), &regvalue)?;
                            acc
                        } else {
                            Err(format!("Key has invalid value:{:?}, {:?}", name, vname).into())
                        }
                    })?;

                acc
            })?;

        Ok(())
    }
}
