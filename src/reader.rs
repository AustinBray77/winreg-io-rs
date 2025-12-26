/*
 *  reader.rs
 *
 *  Description: This module contains code for reading in .reg files to the windows registry
 *  (via the RegReader struct) and util functions for translating between regashii and winreg values.
 *
 *  Created By: AustinBray77
 */

use std::{error::Error, path::Path};

use regashii::{KeyName, Registry};
use winreg::{HKEY, RegKey, transaction::Transaction};

use crate::NormalResult;
use crate::parsing::*;

fn keyname_to_regkey(name: &KeyName) -> Result<RegKey, Box<dyn Error>> {
    let mut split = name.raw().split("\\");

    let hkey: HKEY =
        hkey_from_string(split.next().ok_or("Var path empty")?).ok_or("HKEY not found")?;

    Ok(split.fold(
        Ok(RegKey::predef(hkey)),
        |acc: Result<RegKey, Box<dyn Error>>, next| Ok(acc?.open_subkey(next)?),
    )?)
}

fn keyname_to_regkey_transacted(
    name: &KeyName,
    tr: &Transaction,
) -> Result<RegKey, Box<dyn Error>> {
    let mut split = name.raw().split("\\");

    let hkey: HKEY =
        hkey_from_string(split.next().ok_or("Var path empty")?).ok_or("HKEY not found")?;

    Ok(split.fold(
        Ok(RegKey::predef(hkey)),
        |acc: Result<RegKey, Box<dyn Error>>, next| Ok(acc?.open_subkey_transacted(next, tr)?),
    )?)
}

#[derive(Debug)]
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
                let regkey = keyname_to_regkey(name)?;

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
                let regkey = keyname_to_regkey_transacted(name, &tr)?;

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
