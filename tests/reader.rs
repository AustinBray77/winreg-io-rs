use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use winreg::transaction::Transaction;
use winreg::types::ToRegValue;
use winreg::{RegKey, RegValue, enums::*};
use winreg_io_rs::reader::RegReader;

/*fn print_contents(file: &Path) {
    //let mut buf = String::new();

    let bytes = std::fs::read(file).unwrap();

    println!("{:?}", bytes);
}*/

#[test]
fn test_file_read() {
    let resource_path = std::env::var("RES_PATH").unwrap();
    let test_file = PathBuf::from(resource_path).join("test.reg");
    const VAL_COUNT: usize = 14;
    let test_values: [(&'static str, RegValue); VAL_COUNT] = [
        ("sz-test", "test".to_reg_value()),
        ("dword-test", 1u32.to_reg_value()),
        ("qword-test", 1u64.to_reg_value()),
        (
            "binary-test",
            RegValue {
                bytes: vec![1u8, 2u8, 3u8, 4u8, 5u8],
                vtype: REG_BINARY,
            },
        ),
        (
            "multi-sz-test",
            vec!["test1".to_string(), "test2".to_string()].to_reg_value(),
        ),
        (
            "expand-sz-test",
            RegValue {
                bytes: "%PATH%\0"
                    .encode_utf16()
                    .flat_map(|c| c.to_le_bytes())
                    .collect(),
                vtype: REG_EXPAND_SZ,
            },
        ),
        (
            "none-test",
            RegValue {
                bytes: vec![],
                vtype: REG_NONE,
            },
        ),
        (
            "le-test",
            RegValue {
                bytes: vec![1, 2, 3, 4],
                vtype: REG_DWORD,
            },
        ),
        (
            "be-test",
            RegValue {
                bytes: vec![1, 2, 3, 4],
                vtype: REG_DWORD_BIG_ENDIAN,
            },
        ),
        (
            "link-test",
            RegValue {
                bytes: "test\0"
                    .encode_utf16()
                    .flat_map(|c| c.to_le_bytes())
                    .collect(),
                vtype: REG_LINK,
            },
        ),
        (
            "resource-list-test",
            RegValue {
                bytes: vec![0u8, 0u8, 0u8, 0u8],
                vtype: REG_RESOURCE_LIST,
            },
        ),
        (
            "full-resource-descriptor-test",
            RegValue {
                bytes: vec![0u8, 0u8, 0u8, 0u8],
                vtype: REG_FULL_RESOURCE_DESCRIPTOR,
            },
        ),
        (
            "resource-requirements-list-test",
            RegValue {
                bytes: vec![0u8, 0u8, 0u8, 0u8],
                vtype: REG_RESOURCE_REQUIREMENTS_LIST,
            },
        ),
        ("qword-little-endian-test", 1u64.to_reg_value()),
    ];
    let test_map: HashMap<&'static str, RegValue> = HashMap::from(test_values);

    //print_contents(&test_file);

    let rr = RegReader::try_read_file(test_file).unwrap();

    let tr = Transaction::new().unwrap();

    rr.load_all_transacted(&tr).unwrap();

    tr.commit().unwrap();

    let test_key = RegKey::predef(HKEY_CLASSES_ROOT)
        .open_subkey("winreg-io-rs")
        .unwrap()
        .open_subkey("test")
        .unwrap();

    let amount_matched =
        test_key
            .enum_values()
            .into_iter()
            .map(|x| x.unwrap())
            .fold(0, |acc, (key, value)| {
                let expect = test_map.get(key.as_str()).unwrap();

                assert!(expect == &value);

                acc + 1
            });

    assert!(amount_matched == VAL_COUNT);
}
