import unittest
from pathlib import Path
from unittest.mock import patch

from core_adapter.cli import _normalize_extra_args
from core_adapter.kitty_adapter import KittyAdapter, KittyLaunchConfig, KittyNotFoundError


class CoreAdapterTests(unittest.TestCase):
    def test_detect_executable_success(self) -> None:
        adapter = KittyAdapter()
        with patch("core_adapter.kitty_adapter.shutil.which", return_value="/usr/bin/kitty"):
            detected = adapter.detect_executable()

        self.assertEqual(detected, Path("/usr/bin/kitty"))

    def test_detect_executable_not_found(self) -> None:
        adapter = KittyAdapter()
        with patch("core_adapter.kitty_adapter.shutil.which", return_value=None):
            with self.assertRaises(KittyNotFoundError):
                adapter.detect_executable()

    def test_build_launch_command_with_all_options(self) -> None:
        adapter = KittyAdapter()
        config = KittyLaunchConfig(
            working_directory=Path("/work"),
            shell="/bin/zsh",
            session_file=Path("/tmp/session.conf"),
            title="Dev",
            config_file=Path("/tmp/kitty.conf"),
            extra_args=["--single-instance"],
        )

        with patch.object(adapter, "detect_executable", return_value=Path("/usr/bin/kitty")):
            command = adapter.build_launch_command(config)

        self.assertEqual(
            command,
            [
                "/usr/bin/kitty",
                "--directory",
                "/work",
                "--title",
                "Dev",
                "--config",
                "/tmp/kitty.conf",
                "--session",
                "/tmp/session.conf",
                "--single-instance",
                "--",
                "/bin/zsh",
            ],
        )

    def test_normalize_extra_args_strips_separator(self) -> None:
        self.assertEqual(_normalize_extra_args(["--", "--single-instance"]), ["--single-instance"])
        self.assertEqual(_normalize_extra_args(["--single-instance"]), ["--single-instance"])


if __name__ == "__main__":
    unittest.main()
