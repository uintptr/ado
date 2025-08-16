#!/usr/bin/env python3
import os
import sys
import subprocess
import argparse
import shutil
import tarfile

from typing import Any
from dataclasses import dataclass

DEF_ARCHIVE_NAME = "container.tgz"


@dataclass
class UserArgs:
    archive_file: str
    install_directory: str

    def __post_init__(self) -> None:
        self.archive_file = os.path.abspath(self.archive_file)
        self.install_directory = os.path.abspath(self.install_directory)


@dataclass
class UpdateConfig:
    archive_mod_ts: float


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


class DockerCompose:

    def __init__(self, directory: str) -> None:

        self.directory = directory
        self.docker_compose = shutil.which("docker-compose")

        if self.docker_compose is None:
            raise FileNotFoundError("docker-compose is missing")

        compose_file = os.path.join(directory, "docker-compose.yml")

        if False == os.path.isfile(compose_file):
            raise FileNotFoundError(f"{compose_file} is missing")

    def stop(self) -> None:

        cmd_line = "docker-compose stop"

        # this'll work even if the container(s) isn't running
        shell_exec(cmd_line, cwd=self.directory)

    def start(self, background: bool = True) -> None:

        cmd_line = "docker-compose up"

        if True == background:
            cmd_line += " -d"

        shell_exec(cmd_line, cwd=self.directory)


def printkv(k: str, v: object) -> None:

    k = f"{k}:"
    print(f"    {k:<35}{v}")


def update_container(args: UserArgs) -> bool:

    compose = DockerCompose(args.install_directory)

    compose.stop()

    # replace the files
    with tarfile.open(args.archive_file, 'r|gz') as tar:
        tar.extractall(os.path.dirname(args.install_directory))

    compose.start()

    return True


def env_check() -> None:

    if 0 == os.getuid() or 0 == os.geteuid():
        raise AssertionError("shouldn't run as root")

    docker_compose = shutil.which("docker-compose")

    if docker_compose is None:
        raise FileNotFoundError("docker-compose was not found")


def main() -> int:

    status = 1

    def_archive = os.path.expanduser(f"~/{DEF_ARCHIVE_NAME}")

    def_install = os.path.join(os.getcwd(), "ado_container")

    parser = argparse.ArgumentParser()

    parser.add_argument("-a",
                        "--archive-file",
                        type=str,
                        default=def_archive,
                        help=f"Archive file path. Default: {def_archive}")

    parser.add_argument("-i",
                        "--install-directory",
                        type=str,
                        default=def_install,
                        help=f"/path/to/ado_container")

    try:
        env_check()

        args = UserArgs(**vars(parser.parse_args()))

        print("Container Updater:")
        printkv("Archive File", args.archive_file)
        printkv("Install Directory", args.install_directory)

        if True == os.path.isfile(args.archive_file):
            updated = update_container(args)
            os.unlink(args.archive_file)
        else:
            updated = False

        printkv("Updated", updated)

        status = 0
    except KeyboardInterrupt:
        pass
    except FileNotFoundError as e:
        print(e)

    return status


if __name__ == '__main__':

    status = main()

    if 0 != status:
        sys.exit(status)
