#!/usr/bin/env nodejs

const crypto = require('crypto');
const path = require('path');
const fsPromises = require('fs').promises;

function printUsage() {
  console.log(`
  extract_decryption_key_nky.js <path_to_nky_file> <path_to_og_private_key>
  
    path_to_nky_file:           Path to the decryption key nky.
    path_to_og_keys_dir:        Path to the original keys: og_private.pem and pg_public.pem.
  
  `)
}

async function testKeypair(ogPrivateKey, ogPublicKey) {
  const plaintext = Buffer.from('Hello world!', 'utf8');

  const testCiphertext = crypto.publicEncrypt({
    key: ogPublicKey,
    padding: crypto.constants.RSA_PKCS1_OAEP_PADDING,
  }, plaintext);

  const decrypted = crypto.privateDecrypt({
    key: ogPrivateKey,
    padding: crypto.constants.RSA_PKCS1_OAEP_PADDING,
  }, testCiphertext);

  if (Buffer.compare(plaintext, decrypted) !== 0) {
    return Promise.reject(new Error('Key pair test failed'));
  }
}

async function parseNkyFile(nkyFilePath) {
  const encoded = await fsPromises.readFile(nkyFilePath);
  const decoded = Buffer.from(encoded.toString(), 'base64');

  const keyCiphertext = decoded.slice(0, 256);
  const safeword = decoded.slice(256, 259);
  const iv = decoded.slice(259, 275);
  const privateKeyCiphertext = decoded.slice(275);

  if (safeword.toString() !== 'H4k') {
    return Promise.reject(new Error('Safeword does not match'));
  }

  /*console.log('keyCiphertext =>', [...keyCiphertext]);
  console.log('safeword =>', [...safeword]);
  console.log('iv =>', [...iv]);
  console.log('privateKeyCiphertext =>', [...privateKeyCiphertext]);*/

  return Promise.resolve({
    keyCiphertext,
    safeword,
    iv,
    privateKeyCiphertext,
  });
}

async function run() {
  if (process.argv.length !== 4) {
    console.error('Please provide the correct arguments');
    printUsage();
    process.exit(1);
  }

  let ogPrivateKey = await fsPromises.readFile(path.join(process.argv[3], 'og_private.pem'));
  let ogPublicKey = await fsPromises.readFile(path.join(process.argv[3], 'og_public.pem'));

  try {
    const {
      keyCiphertext,
      safeword,
      iv,
      privateKeyCiphertext,
    } = await parseNkyFile(process.argv[2]);

    if (safeword.toString() !== 'H4k') {
      console.error('Safword check failed');
      process.exit(1);
    }

    await testKeypair(ogPrivateKey, ogPublicKey);

    const key = crypto.privateDecrypt(ogPrivateKey, keyCiphertext);
    const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
    const privateKey = decipher.update(privateKeyCiphertext, null, 'utf8');

    console.log((new Buffer(privateKey)).toString('base64'));
  } catch (e) {
    console.error(e);
    process.exit(1);
  }


}

run();
