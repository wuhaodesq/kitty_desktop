from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
import shutil
import subprocess
from typing import Sequence


class KittyNotFoundError(RuntimeError):
    """Raised when kitty executable cannot be found."""


@dataclass(slots=True)
class KittyLaunchConfig:
    """Settings used to construct kitty launch command."""

    working_directory: Path | None = None
    shell: str | None = None
    session_file: Path | None = None
    title: str | None = None
    config_file: Path | None = None
    extra_args: list[str] = field(default_factory=list)


class KittyAdapter:
    """Adapter layer to detect kitty and build launch commands."""

    def __init__(self, executable_candidates: Sequence[str] | None = None) -> None:
        self._executable_candidates = tuple(executable_candidates or ("kitty",))

    def detect_executable(self) -> Path:
        """Detect kitty executable path from known candidates and PATH."""
        for candidate in self._executable_candidates:
            detected = shutil.which(candidate)
            if detected:
                return Path(detected)
        raise KittyNotFoundError(
            "kitty executable was not found in PATH. "
            "Please install kitty or configure executable candidates."
        )

    def build_launch_command(self, config: KittyLaunchConfig) -> list[str]:
        """Build a kitty command list based on launch config."""
        kitty_path = self.detect_executable()
        command: list[str] = [str(kitty_path)]

        if config.working_directory is not None:
            command.extend(["--directory", str(config.working_directory)])

        if config.title:
            command.extend(["--title", config.title])

        if config.config_file is not None:
            command.extend(["--config", str(config.config_file)])

        if config.session_file is not None:
            command.extend(["--session", str(config.session_file)])

        if config.extra_args:
            command.extend(config.extra_args)

        if config.shell:
            command.extend(["--", config.shell])

        return command

    def launch(
        self,
        config: KittyLaunchConfig,
        *,
        dry_run: bool = False,
        env: dict[str, str] | None = None,
    ) -> list[str] | subprocess.Popen[str]:
        """Launch kitty process or return command when dry_run is enabled."""
        command = self.build_launch_command(config)
        if dry_run:
            return command

        return subprocess.Popen(
            command,
            env=env,
            text=True,
        )

    def get_version(self) -> str:
        """Return kitty version string (first line)."""
        kitty_path = self.detect_executable()
        result = subprocess.run(
            [str(kitty_path), "--version"],
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip().splitlines()[0]
