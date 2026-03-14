from __future__ import annotations

import argparse
from pathlib import Path

from .kitty_adapter import KittyAdapter, KittyLaunchConfig, KittyNotFoundError


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="kitty-desktop-core-adapter",
        description="kitty_desktop core-adapter PoC CLI",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    version_parser = subparsers.add_parser("version", help="Print detected kitty version")
    version_parser.set_defaults(command_handler=handle_version)

    launch_parser = subparsers.add_parser("launch", help="Launch kitty with desktop wrapper args")
    launch_parser.add_argument("--directory", type=Path, help="Working directory")
    launch_parser.add_argument("--shell", help="Shell executable to run inside kitty")
    launch_parser.add_argument("--session", type=Path, help="Kitty session file")
    launch_parser.add_argument("--title", help="Window title")
    launch_parser.add_argument("--config", type=Path, help="Kitty config file")
    launch_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Only print command without launching kitty",
    )
    launch_parser.add_argument(
        "extra_args",
        nargs=argparse.REMAINDER,
        help="Extra args passed directly to kitty (prefix with --)",
    )
    launch_parser.set_defaults(command_handler=handle_launch)

    return parser


def handle_version(args: argparse.Namespace) -> int:
    adapter = KittyAdapter()
    try:
        print(adapter.get_version())
    except KittyNotFoundError as exc:
        print(f"ERROR: {exc}")
        return 1
    return 0


def handle_launch(args: argparse.Namespace) -> int:
    adapter = KittyAdapter()
    cfg = KittyLaunchConfig(
        working_directory=args.directory,
        shell=args.shell,
        session_file=args.session,
        title=args.title,
        config_file=args.config,
        extra_args=_normalize_extra_args(args.extra_args),
    )

    try:
        result = adapter.launch(cfg, dry_run=args.dry_run)
    except KittyNotFoundError as exc:
        print(f"ERROR: {exc}")
        return 1

    if args.dry_run:
        print(" ".join(result))
    else:
        print("kitty launched")

    return 0


def _normalize_extra_args(extra_args: list[str]) -> list[str]:
    if extra_args and extra_args[0] == "--":
        return extra_args[1:]
    return extra_args


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.command_handler(args)


if __name__ == "__main__":
    raise SystemExit(main())
