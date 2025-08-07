#!/usr/bin/env python3
import urllib.request
import urllib
from dataclasses import dataclass
import os
import sys
import argparse
import ssl


DEF_STORAGE_SERVER = "http://localhost:7379"


@dataclass
class UserArgs:
    user_id: str
    config_file: str
    storage_server: str
    allow_self_signed: bool

    def __post_init__(self) -> None:
        self.config_file = os.path.abspath(self.config_file)


def printkv(k: str, v: object) -> None:

    k = f"{k}:"
    print(f"    {k:<35}{v}")


def add_user(args: UserArgs) -> None:

    url = f"{args.storage_server}/SET/{args.user_id}"

    with open(args.config_file) as f:
        data = f.read()

    encoded_data = data.encode("utf-8")

    req = urllib.request.Request(url, data=encoded_data, method="PUT")
    req.add_header('Content-Type', 'application/json')

    if args.allow_self_signed:
        context = ssl._create_unverified_context()  # type: ignore
    else:
        context = None

    with urllib.request.urlopen(req, context=context) as res:
        assert 200 == res.status

    # force a SAVE
    save_url = f"{args.storage_server}/SAVE"
    req = urllib.request.Request(save_url)

    with urllib.request.urlopen(req, context=context) as res:
        assert 200 == res.status


def main() -> int:

    status = 1

    parser = argparse.ArgumentParser()

    parser.add_argument("-u",
                        "--user-id",
                        type=str,
                        required=True,
                        help="User ID")

    parser.add_argument("-f",
                        "--config-file",
                        type=str,
                        required=True,
                        help="/path/to/config.toml")

    parser.add_argument("-s",
                        "--storage-server",
                        type=str,
                        default=DEF_STORAGE_SERVER,
                        help=f"Storage Server. Default: {DEF_STORAGE_SERVER}")

    parser.add_argument("--allow-self-signed",
                        action="store_true",
                        help="Allow self signed certs")

    try:

        args = UserArgs(**vars(parser.parse_args()))

        print("Add User:")
        printkv("User Id", args.user_id)
        printkv("Config File", args.config_file)
        printkv("Storage Server", args.storage_server)
        printkv("Allow Self-signed Certificates", args.allow_self_signed)

        add_user(args)

        status = 0
    except KeyboardInterrupt:
        pass

    return status


if __name__ == '__main__':

    status = main()

    if 0 != status:
        sys.exit(status)
