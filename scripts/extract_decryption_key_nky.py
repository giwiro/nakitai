#!/usr/bin/env python3

import base64
import subprocess
import sys
import tempfile


def print_usage():
    print("""
extract_decryption_key_nky.py <path_to_nky_file> <path_to_og_private_key>
    
    path_to_nky_file:           Path to the decryption key nky.
    path_to_og_private_key:     Path to the original private key that is NOT embedded.
    
""")


def run(file_path: str, og_private_key_path: str):
    try:
        with open(file_path, "rb") as f:
            decoded = base64.decodebytes(f.read())

            key_ciphertext = decoded[:256]
            safeword = decoded[256:259]
            iv = decoded[259:275]
            private_key_ciphertext = decoded[275:]

            if safeword.decode('utf-8') != "H4k":
                sys.exit("Safeword does not match. Probably corrupted key.")

            print(f"key_ciphertext => {list(key_ciphertext)}")
            print(f"safeword => {list(safeword)}")
            print(f"iv => {list(iv)}")
            print(f"private_key_ciphertext => {list(private_key_ciphertext)}")
            print(f"key_ciphertext => {key_ciphertext}")

            with tempfile.NamedTemporaryFile(delete=False) as tmp:
                print(tmp.name)
                tmp.write(key_ciphertext)

                decrypt_output = subprocess.run(
                    ["openssl", "rsautl", "-decrypt", "-oaep", "-in", tmp.name, "-inkey", og_private_key_path],
                    capture_output=True,
                    text=True,
                    check=True,
                )

                print(f"decrypt_output: {decrypt_output.stdout}")

    except FileNotFoundError:
        sys.exit("File was not found")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Please provide the correct arguments")
        print_usage()
        sys.exit()

    run(sys.argv[1], sys.argv[2])
