#!/usr/bin/env python3

import os
import sys
import argparse
import tempfile
import subprocess
import shutil
import hashlib

from dataclasses import dataclass
from typing import Any

DEF_IMAGE_NAME = "webapp"


@dataclass
class UserArgs:
    image_name: str
    domain_name: str
    cert_file: str
    cert_key: str
    output: str

    def __post_init__(self) -> None:
        self.cert_file = os.path.abspath(self.cert_file)
        self.cert_key = os.path.abspath(self.cert_key)
        self.output = os.path.abspath(self.output)


def printkv(k: str, v: object) -> None:

    k = f"{k}:"
    print(f"    {k:<28}{v}")


def size_fmt(num: float, suffix: str = "B") -> str:
    for unit in ("", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi"):
        if abs(num) < 1024.0:
            return f"{num:3.1f} {unit}{suffix}"
        num /= 1024.0
    return f"{num:.1f} Yi{suffix}"


def file_size_fmt(file_path: str) -> str:
    return size_fmt(os.stat(file_path).st_size)


def sha512_file(file_path: str) -> str:
    hasher = hashlib.sha512()

    with open(file_path, 'rb') as f:

        while True:
            chunk = f.read(8 * 1024)
            if b'' == chunk:
                break
            hasher.update(chunk)

    return hasher.hexdigest()


def shell_exec(cmd_line: str,
               cwd: str | None = None,
               env: dict[str, Any] | None = None,
               check: bool = True) -> tuple[int, str, str]:

    out_str = ""
    out_err = ""

    env_copy = os.environ.copy()

    if env is not None:
        env_copy |= env

    p = subprocess.Popen(cmd_line,
                         shell=True,
                         text=True,
                         env=env_copy,
                         stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE,
                         cwd=cwd)

    out_str, out_err = p.communicate()

    if p.returncode is not None:
        ret = p.returncode
    else:
        ret = 1

    if True == check and 0 != ret:
        raise AssertionError(cmd_line, ret, out_str, out_err)

    return ret, out_str, out_err


class DockerBuilder:

    def __init__(self, args: UserArgs) -> None:
        self.args = args

        self.script_root = os.path.abspath(os.path.dirname(sys.argv[0]))

        self.docker = shutil.which("docker")

        if self.docker is None:
            raise AssertionError("docker is not installed")

        self.wasm_pack = shutil.which("wasm-pack")

        if self.wasm_pack is None:
            raise AssertionError("wasm-pack is not installed")

    def __get_template(self, file_name: str) -> str:

        template_dir = os.path.join(self.script_root, "templates")

        file_path = os.path.join(template_dir, file_name)

        with open(file_path) as f:
            return f.read()

    def __build_nginx(self, out_file: str) -> None:

        config_file = self.__get_template("nginx.conf")

        config_file = config_file.replace("__DOMAIN_NAME__",
                                          self.args.domain_name)

        with open(out_file, "w+") as f:
            f.write(config_file)

    def __copy_static_content(self, out_dir: str) -> None:

        www_root = os.path.join(self.script_root, os.pardir)
        www_root = os.path.join(www_root, "src", "lib", "adolib", "www")
        www_root = os.path.abspath(www_root)

        shutil.copytree(www_root, out_dir)

    def __copy_open_search(self, out_file: str) -> None:

        xml_file = self.__get_template("opensearch.xml")
        xml_file = xml_file.replace("__DOMAIN_NAME__", self.args.domain_name)

        with open(out_file, "w+") as f:
            f.write(xml_file)

    def __copy_cert_files(self, out_dir: str) -> None:

        shutil.copy2(self.args.cert_file, out_dir)
        shutil.copy2(self.args.cert_key, out_dir)

    def __build_image(self, wd: str) -> None:

        cmd_line = f"{self.docker} build -t {self.args.image_name} ."
        shell_exec(cmd_line, cwd=wd)

        try:
            cmd_line = f"{self.docker} save -o {self.args.output}"
            cmd_line += f" {self.args.image_name}"

            shell_exec(cmd_line, cwd=wd)
        finally:
            # always try to delete the new image regardless
            cmd_line = f"{self.docker} rmi {self.args.image_name}"

            # best effort. don't blow up if this fails
            # shell_exec(cmd_line, check=False)

    def __build_docker_file(self, out_file: str) -> None:

        docker_file = self.__get_template("Dockerfile")

        with open(out_file, "w+") as f:
            f.write(docker_file)

    def __build_wasm(self, wasm_pkg_root: str) -> None:

        wd = os.path.join(self.script_root, os.pardir)
        wd = os.path.abspath(wd)

        cmd_line = f"{self.wasm_pack} build"
        cmd_line += f" src/lib/adolib/ --target web -d {wasm_pkg_root}"

        shell_exec(cmd_line, cwd=wd)

    def build(self) -> None:

        with tempfile.TemporaryDirectory(prefix="docker_root_") as td:

            www_root = os.path.join(td, "www")
            self.__copy_static_content(www_root)

            open_search = os.path.join(www_root, "opensearch.xml")
            self.__copy_open_search(open_search)

            wasm_pkg_root = os.path.join(www_root, "pkg")
            self.__build_wasm(wasm_pkg_root)

            self.__copy_cert_files(td)

            nginx_conf = os.path.join(td, "nginx.conf")
            self.__build_nginx(nginx_conf)

            docker_file = os.path.join(td, "Dockerfile")
            self.__build_docker_file(docker_file)

            self.__build_image(td)


def main() -> int:

    status = 1

    parser = argparse.ArgumentParser()

    script_root = os.path.abspath(os.path.dirname(sys.argv[0]))
    def_cert_root = os.path.join(script_root, "certs")

    def_cert = os.path.join(def_cert_root, "fullchain.pem")
    def_key = os.path.join(def_cert_root, "privkey.pem")

    def_output = os.path.join(script_root, f"{DEF_IMAGE_NAME}.tar")

    parser.add_argument("-d",
                        "--domain-name",
                        type=str,
                        required=True,
                        help="Server Domain Name")
    parser.add_argument("-i",
                        "--image-name",
                        default=DEF_IMAGE_NAME,
                        type=str,
                        help=f"Docker image name. Default:{DEF_IMAGE_NAME}")

    parser.add_argument("-c",
                        "--cert-file",
                        type=str,
                        default=def_cert,
                        help=f"/path/to/fullchain.pem. Default: {def_cert}")

    parser.add_argument("-k",
                        "--cert-key",
                        type=str,
                        default=def_key,
                        help=f"/path/to/privkey.pem. Default: {def_key}")

    parser.add_argument("-o",
                        "--output",
                        type=str,
                        default=def_output,
                        help=f"Output image file. Default: {def_output}")

    try:

        args = parser.parse_args()
        # type check
        args = UserArgs(**vars(args))

        print("Docker Builder:")
        printkv("Image Name", args.image_name)
        printkv("Domain Name", args.domain_name)
        printkv("Certificate File", args.cert_file)
        printkv("Certificate Private Key", args.cert_key)

        assert os.path.isfile(args.cert_file)
        assert os.path.isfile(args.cert_key)

        builder = DockerBuilder(args)
        builder.build()

        printkv("Output Image", args.output)
        printkv("Output Image Size", file_size_fmt(args.output))
        printkv("Output Image Hash", sha512_file(args.output))

        status = 0
    except AssertionError as e:
        print(e)
    except KeyboardInterrupt:
        pass

    return status


if __name__ == '__main__':

    status = main()

    if 0 != status:
        sys.exit(status)
