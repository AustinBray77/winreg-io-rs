use std::cmp::min;

use regashii::{Kind, Value};
use winreg::{HKEY, RegValue, enums::*};

const U32SIZE: usize = 4;
const U64SIZE: usize = 8;

pub fn u32_to_u8s(val: u32) -> [u8; U32SIZE] {
    let mut output = [0_u8; U32SIZE];

    let mut mask = 1_u32 << 8 - 1;

    for i in 0..U32SIZE {
        output[i] = ((val & mask) >> (i * 8)) as u8;
        mask <<= 8;
    }

    output
}

pub fn u64_to_u8s(val: u64) -> [u8; U64SIZE] {
    let mut output = [0_u8; U64SIZE];

    let mut mask = 1_u64 << 8 - 1;

    for i in 0..U64SIZE {
        output[i] = ((val & mask) >> (i * 8)) as u8;
        mask <<= 8;
    }

    output
}

pub fn kind_to_regtype(kind: Kind) -> Option<RegType> {
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

pub fn regasii_to_winreg(value: regashii::Value) -> Option<RegValue> {
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

pub fn hkey_from_string(string: &str) -> Option<HKEY> {
    println!("{:?}", string);

    match string {
        "HKEY_CLASSES_ROOT" => Some(HKEY_CLASSES_ROOT),
        "HKEY_CURRENT_CONFIG" => Some(HKEY_CURRENT_CONFIG),
        "HKEY_CURRENT_USER" => Some(HKEY_CURRENT_USER),
        "HKEY_LOCAL_MACHINE" => Some(HKEY_LOCAL_MACHINE),
        "HKEY_USERS" => Some(HKEY_USERS),
        "HKEY_PERFORMANCE_DATA" => Some(HKEY_PERFORMANCE_DATA),
        "HKEY_DYN_DATA" => Some(HKEY_DYN_DATA),
        "HKEY_CURRENT_USER_LOCAL_SETTINGS" => Some(HKEY_CURRENT_USER_LOCAL_SETTINGS),
        "HKEY_PERFORMANCE_TEXT" => Some(HKEY_PERFORMANCE_TEXT),
        "HKEY_PERFORMANCE_NLSTEXT" => Some(HKEY_PERFORMANCE_NLSTEXT),
        _ => None,
    }
}

pub fn u8s_to_u32(vec: Vec<u8>) -> u32 {
    let mut output: u32 = 0_u32;

    for i in 0..(min(vec.len(), 4)) {
        output += (vec[0] as u32) << (8 * (3 - i));
    }

    output
}

pub fn u8s_to_u64(vec: Vec<u8>) -> u64 {
    let mut output: u64 = 0_64;

    for i in 0..(min(vec.len(), 8)) {
        output += (vec[0] as u64) << (8 * (7 - i));
    }

    output
}

pub fn winreg_to_regashii(value: RegValue) -> Option<Value> {
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
