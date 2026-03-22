#!/usr/bin/env python3

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import uuid
from dataclasses import asdict, dataclass
from typing import Any

WWW_IGNORE_LIST = ["config.toml", "test_config.json"]


@dataclass
class UserArgs:
    domain_name: str
    output: str
    www_root: str
    test_config: str | None
    ado_bin: str | None
    ado_config: str | None

    def __post_init__(self) -> None:
        self.output = os.path.abspath(self.output)

        if self.test_config is not None:
            self.test_config = os.path.abspath(self.test_config)

        if self.ado_bin is not None:
            self.ado_bin = os.path.abspath(self.ado_bin)

        if self.ado_config is not None:
            self.ado_config = os.path.abspath(self.ado_config)


@dataclass
class TestConfig:
    user_id: str
    config_file: str


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

    if include_root:
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

    if check and 0 != ret:
        raise AssertionError(cmd_line, ret, out_str, out_err)

    return ret, out_str, out_err


def find_container_runtime() -> tuple[str, str]:
    """Return (runtime_bin, compose_cmd) for docker or podman."""
    for runtime_name in ("docker", "podman"):
        runtime = shutil.which(runtime_name)
        if runtime is None:
            continue
        # prefer the subcommand form (v2-style)
        ret, _, _ = shell_exec(f"{runtime} compose version", check=False)
        if ret == 0:
            return runtime, f"{runtime} compose"
        # fall back to standalone docker-compose / podman-compose
        standalone = shutil.which(f"{runtime_name}-compose")
        if standalone is not None:
            return runtime, standalone
    raise AssertionError("neither docker nor podman is installed")


class DockerBuilder:

    def __init__(self, args: UserArgs) -> None:
        self.args = args

        self.script_root = os.path.abspath(os.path.dirname(sys.argv[0]))

        self.runtime, self.compose = find_container_runtime()

    def __get_template(self, file_name: str) -> str:

        template_dir = os.path.join(self.script_root, "templates")

        file_path = os.path.join(template_dir, file_name)

        with open(file_path) as f:
            return f.read()

    def __www_copytree_ignore(self, dir: str, files: list[str]) -> list[str]:

        basename = os.path.basename(dir)

        if basename == "www":

            ignores: list[str] = []

            for file_name in WWW_IGNORE_LIST:

                if file_name in files:
                    ignores.append(file_name)

            return ignores

        return []

    def __copy_static_content(self, out_dir: str) -> None:

        www_root = os.path.join(self.script_root, os.pardir, os.pardir)
        www_root = os.path.join(www_root, "www")
        www_root = os.path.abspath(www_root)

        shutil.copytree(www_root, out_dir, ignore=self.__www_copytree_ignore)

    def __copy_open_search(self, out_file: str) -> None:

        xml_file = self.__get_template("opensearch.xml")
        xml_file = xml_file.replace("__DOMAIN_NAME__", self.args.domain_name)

        with open(out_file, "w+") as f:
            f.write(xml_file)

    def __build_www(self, www_root: str) -> None:

        self.__copy_static_content(www_root)

        open_search = os.path.join(www_root, "opensearch.xml")
        self.__copy_open_search(open_search)

    def __build_nginx(self, container_root: str) -> None:

        template = self.__get_template("nginx.conf")
        template = template.replace("__DOMAIN_NAME__", self.args.domain_name)

        conf_d_dir = os.path.join(container_root, "conf.d")
        os.mkdir(conf_d_dir)

        nginx_conf = os.path.join(conf_d_dir, "default.conf")

        with open(nginx_conf, "w+") as f:
            f.write(template)

    def __cargo_build_ado(self) -> str:

        wd = os.path.join(self.script_root, os.pardir, os.pardir)
        wd = os.path.abspath(wd)

        target = "x86_64-unknown-linux-gnu"

        cross = shutil.which("cross")
        tool = cross if cross is not None else shutil.which("cargo")

        if tool is None:
            raise AssertionError("neither cross nor cargo is installed")

        env = {}
        if cross is not None:
            env["CROSS_CONTAINER_ENGINE"] = self.runtime

        shell_exec(f"{tool} build --release --target {target} --bin ado",
                   cwd=wd, env=env)

        return os.path.join(wd, "target", target, "release", "ado")

    def __build_ado(self, ado_root: str) -> None:

        dockerfile = os.path.join(self.script_root, "ado", "Dockerfile")
        shutil.copy2(dockerfile, ado_root)

        ado_bin = self.args.ado_bin if self.args.ado_bin is not None else self.__cargo_build_ado()
        shutil.copy2(ado_bin, os.path.join(ado_root, "ado"))

        if self.args.ado_config is not None:
            shutil.copy2(self.args.ado_config,
                         os.path.join(ado_root, "config.toml"))

    def __build_docker_compose(self, container_root: str) -> None:

        template = self.__get_template("docker-compose.yml")

        template = template.replace("__WWW_ROOT__", self.args.www_root)

        compose_file = os.path.join(container_root, "docker-compose.yml")

        with open(compose_file, "w+") as f:
            f.write(template)

    def __build_containers(self, container_root: str) -> None:

        shell_exec(f"{self.compose} build", cwd=container_root)

    def __build_test_config(self, config_path: str, www_root: str) -> None:

        with open(config_path) as f:
            config_file = f.read()

        user_id = str(uuid.uuid4())

        config = TestConfig(user_id, config_file)

        config_json = json.dumps(asdict(config), indent=4)

        www_config = os.path.join(www_root, "test_config.json")

        with open(www_config, "w+") as f:
            f.write(config_json)

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
            # build test config
            #
            if self.args.test_config is not None:
                self.__build_test_config(self.args.test_config, www_root)

            #
            # /etc/nginx/conf.d/
            #
            self.__build_nginx(container_root)

            #
            # ado (headless + ttyd)
            #
            ado_root = os.path.join(container_root, "ado")
            os.mkdir(ado_root)
            self.__build_ado(ado_root)

            #
            # docker compose file
            #
            self.__build_docker_compose(container_root)

            #
            # build containers
            #
            self.__build_containers(container_root)

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

    parser.add_argument("--test-config",
                        type=str,
                        help="/path/to/test/config.toml")

    parser.add_argument("--ado-bin",
                        type=str,
                        help="/path/to/ado linux binary")

    parser.add_argument("--ado-config",
                        type=str,
                        help="/path/to/ado/config.toml")

    try:

        args = parser.parse_args()
        # type check
        args = UserArgs(**vars(args))

        builder = DockerBuilder(args)

        print(f"Container Builder ({builder.runtime}):")
        printkv("Domain Name", args.domain_name)
        printkv("WWW Root", args.www_root)
        printkv("Test Config", args.test_config)
        printkv("Ado Binary", args.ado_bin)
        printkv("Ado Config", args.ado_config)

        builder.build()

        printkv("Output Image", args.output)
        printkv("Output Image Size", file_size_fmt(args.output))
        printkv("Output Image Hash", sha256_file(args.output))

        status = 0
    except AssertionError as e:
        print(f"Assertion failure: {e}")
    except KeyboardInterrupt:
        pass

    return status


if __name__ == '__main__':

    status = main()

    if 0 != status:
        sys.exit(status)
