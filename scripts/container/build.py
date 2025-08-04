#!/usr/bin/env python3

import os
import sys
import argparse
import tempfile
import subprocess
import shutil
import hashlib
import tarfile

from dataclasses import dataclass
from typing import Any


@dataclass
class UserArgs:
    domain_name: str
    cert_file: str
    cert_key: str
    output: str
    debug: bool
    www_root: str

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


def sha256_file(file_path: str) -> str:
    hasher = hashlib.sha256()

    with open(file_path, 'rb') as f:

        while True:
            chunk = f.read(8 * 1024)
            if b'' == chunk:
                break
            hasher.update(chunk)

    return hasher.hexdigest()


def tarball(directory: str, out_file: str, include_root: bool = True) -> None:

    if True == include_root:
        rel_dir = os.path.dirname(directory)
    else:
        rel_dir = directory

    with tarfile.open(out_file, "w:gz") as t:
        for root, _, files in os.walk(directory):
            for file in files:
                file_path = os.path.join(root, file)
                rel = os.path.relpath(file_path, rel_dir)
                t.add(file_path, rel)


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

        www_root = os.path.join(self.script_root, os.pardir, os.pardir)
        www_root = os.path.join(www_root, "src", "lib", "adolib", "www")
        www_root = os.path.abspath(www_root)

        shutil.copytree(www_root, out_dir)

    def __copy_open_search(self, out_file: str) -> None:

        xml_file = self.__get_template("opensearch.xml")
        xml_file = xml_file.replace("__DOMAIN_NAME__", self.args.domain_name)

        with open(out_file, "w+") as f:
            f.write(xml_file)

    def __build_wasm(self, wasm_pkg_root: str) -> None:

        wd = os.path.join(self.script_root, os.pardir, os.pardir)
        wd = os.path.abspath(wd)

        cmd_line = f"{self.wasm_pack} build"
        cmd_line += f" src/lib/adolib/ --target web -d {wasm_pkg_root}"

        if self.args.debug:
            cmd_line += " --no-opt --debug"

        shell_exec(cmd_line, cwd=wd)

    def __build_www(self, www_root: str) -> None:

        self.__copy_static_content(www_root)

        open_search = os.path.join(www_root, "opensearch.xml")
        self.__copy_open_search(open_search)

        wasm_pkg_root = os.path.join(www_root, "pkg")
        self.__build_wasm(wasm_pkg_root)

    def __build_certs(self, certs_root: str) -> None:

        shutil.copy2(self.args.cert_file, certs_root)
        shutil.copy2(self.args.cert_key, certs_root)

    def __build_conf_d(self, etc_root: str) -> None:
        nginx_conf = os.path.join(etc_root, "default.conf")
        self.__build_nginx(nginx_conf)

    def __build_webdis(self, redis_root: str) -> None:

        webdis_json = os.path.join(self.script_root,
                                   "webdis",
                                   "webdis.prod.json")
        shutil.copy2(webdis_json, redis_root)

    def __build_docker_compose(self, container_root: str) -> None:

        template = self.__get_template("docker-compose.yml")

        template = template.replace("__WWW_ROOT__", self.args.www_root)

        compose_file = os.path.join(container_root, "docker-compose.yml")

        with open(compose_file, "w+") as f:
            f.write(template)

    def build(self) -> None:

        with tempfile.TemporaryDirectory(prefix="docker_root_") as td:

            container_root = os.path.join(td, "ado_container")
            os.mkdir(container_root)

            #
            # www
            #
            www_root = os.path.join(container_root, "www")
            self.__build_www(www_root)

            #
            # /etc/certs
            #
            certs_root = os.path.join(container_root, "certs")
            os.mkdir(certs_root)
            self.__build_certs(certs_root)

            #
            # /etc/nginx/conf.d/
            #
            conf_d = os.path.join(container_root, "conf.d")
            os.mkdir(conf_d)
            self.__build_conf_d(conf_d)

            #
            # webdis + redis
            #
            webdis_root = os.path.join(container_root, "webdis")
            os.mkdir(webdis_root)
            self.__build_webdis(webdis_root)

            #
            # docker compose file
            #
            self.__build_docker_compose(container_root)

            tarball(container_root, self.args.output)


def main() -> int:

    status = 1

    parser = argparse.ArgumentParser()

    script_root = os.path.abspath(os.path.dirname(sys.argv[0]))
    def_cert_root = os.path.join(script_root, "certs")

    def_cert = os.path.join(def_cert_root, "fullchain.pem")
    def_key = os.path.join(def_cert_root, "privkey.pem")

    def_output = os.path.join(os.getcwd(), "container.tgz")

    parser.add_argument("-n",
                        "--domain-name",
                        type=str,
                        required=True,
                        help="Server Domain Name")

    parser.add_argument("-d",
                        "--debug",
                        action="store_true",
                        help="Debug build")

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
                        help=f"Output config archive. Default: {def_output}")

    parser.add_argument("-w",
                        "--www-root",
                        type=str,
                        default="./www",
                        help="Docker www root directory")

    try:

        args = parser.parse_args()
        # type check
        args = UserArgs(**vars(args))

        print("Docker Builder:")
        printkv("Domain Name", args.domain_name)
        printkv("Certificate File", args.cert_file)
        printkv("Certificate Private Key", args.cert_key)
        printkv("WWW Root", args.www_root)
        printkv("Debug Build", args.debug)

        assert os.path.isfile(args.cert_file)
        assert os.path.isfile(args.cert_key)

        builder = DockerBuilder(args)
        builder.build()

        printkv("Output Image", args.output)
        printkv("Output Image Size", file_size_fmt(args.output))
        printkv("Output Image Hash", sha256_file(args.output))

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
