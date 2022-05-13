#!/usr/bin/env nodejs

const crypto = require('crypto');
const fsPromises = require('fs').promises;

function genKeys() {
  return new Promise((resolve, reject) => {
    crypto.generateKeyPair('rsa', {
      modulusLength: 2048,
      publicKeyEncoding: {
        type: 'spki',
        format: 'pem',
      },
      privateKeyEncoding: {
        type: 'pkcs1',
        format: 'pem'
      },
    }, (error, publicKey, privateKey) => {
      if (error) {
        return reject(error);
      }

      return resolve([publicKey, privateKey]);
    });
  });
}

async function run() {
  const [publicKey, privateKey] = await genKeys();

  await fsPromises.writeFile('og_public.pem', publicKey);
  await fsPromises.writeFile('og_private.pem', privateKey);
}

run().then(() => console.log('Keys generated'));
