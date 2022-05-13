#!/usr/bin/env python3

import subprocess


def run():
    try:
        subprocess.run(
            ["openssl", "genrsa", "-out", "og_private.pem", "2048"],
            capture_output=True,
            text=True,
            check=True,
        )

        subprocess.run(
            [
                "openssl",
                "rsa",
                "-in",
                "og_private.pem",
                "-outform",
                "PEM",
                "-pubout",
                "-out",
                "og_public.pem",
            ],
            capture_output=True,
            text=True,
            check=True,
        )

    except Exception as e:
        print(e)


if __name__ == "__main__":
    run()
