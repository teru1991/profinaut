from __future__ import annotations

import os
from pathlib import Path

from services.marketdata.app.storage.object_store import ObjectStore


class FilesystemObjectStore(ObjectStore):
    def __init__(self, root: str | Path):
        self._root = Path(root)
        self._root.mkdir(parents=True, exist_ok=True)

    def _path_for_key(self, key: str) -> Path:
        normalized = key.lstrip("/")
        return self._root / normalized

    def put_bytes(self, key: str, data: bytes, content_type: str = "application/octet-stream") -> None:
        del content_type
        final_path = self._path_for_key(key)
        final_path.parent.mkdir(parents=True, exist_ok=True)

        tmp_path = final_path.with_suffix(final_path.suffix + ".tmpwrite")
        with open(tmp_path, "wb") as fp:
            fp.write(data)
            fp.flush()
            os.fsync(fp.fileno())
        os.replace(tmp_path, final_path)

    def get_bytes(self, key: str) -> bytes:
        with open(self._path_for_key(key), "rb") as fp:
            return fp.read()

    def list(self, prefix: str) -> list[str]:
        normalized_prefix = prefix.lstrip("/")
        base = self._path_for_key(normalized_prefix)

        if not base.exists():
            return []

        result: list[str] = []
        for path in base.rglob("*"):
            if path.is_file():
                result.append(path.relative_to(self._root).as_posix())
        return sorted(result)

    def rename(self, src_key: str, dst_key: str) -> None:
        src = self._path_for_key(src_key)
        dst = self._path_for_key(dst_key)
        dst.parent.mkdir(parents=True, exist_ok=True)
        os.replace(src, dst)
