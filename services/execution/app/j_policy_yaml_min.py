from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any


class YamlMinError(RuntimeError):
    pass


@dataclass(frozen=True)
class _Line:
    indent: int
    text: str


def _strip_comment(line: str) -> str:
    if "#" in line:
        idx = line.find("#")
        before = line[:idx]
        if before.endswith(" ") or before == "":
            return before.rstrip()
    return line.rstrip()


def _parse_scalar(raw: str) -> Any:
    s = raw.strip()
    if s == "":
        return ""
    if s.startswith("[") and s.endswith("]"):
        inner = s[1:-1].strip()
        if inner == "":
            return []
        return [_parse_scalar(part.strip()) for part in inner.split(",")]
    if s in ("true", "True"):
        return True
    if s in ("false", "False"):
        return False
    if s.isdigit() or (s.startswith("-") and s[1:].isdigit()):
        return int(s, 10)
    if any(ch in s for ch in (".", "e", "E")):
        try:
            return float(s)
        except Exception:
            pass
    if (s.startswith('"') and s.endswith('"')) or (s.startswith("'") and s.endswith("'")):
        return s[1:-1]
    return s


def load_yaml_min(path: Path) -> Any:
    if not path.exists():
        raise YamlMinError(f"file not found: {path}")

    raw_lines: list[_Line] = []
    for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if "\t" in line:
            raise YamlMinError(f"tab is not allowed (line {i})")
        stripped = _strip_comment(line)
        if stripped.strip() == "":
            continue
        indent = len(stripped) - len(stripped.lstrip(" "))
        if indent % 2 != 0:
            raise YamlMinError(f"indent must be multiple of 2 (line {i})")
        raw_lines.append(_Line(indent=indent, text=stripped.lstrip(" ")))

    idx = 0

    def parse_map(expected_indent: int) -> dict[str, Any]:
        nonlocal idx
        out: dict[str, Any] = {}
        while idx < len(raw_lines):
            line = raw_lines[idx]
            if line.indent < expected_indent:
                break
            if line.indent > expected_indent:
                raise YamlMinError("unexpected indent jump")
            if line.text.startswith("- "):
                break
            if ":" not in line.text:
                raise YamlMinError(f"expected key: value, got {line.text}")
            key, rest = line.text.split(":", 1)
            key = key.strip()
            rest = rest.strip()
            if not key:
                raise YamlMinError("empty key")
            idx += 1
            if rest == "":
                if idx >= len(raw_lines) or raw_lines[idx].indent <= expected_indent:
                    out[key] = {}
                else:
                    out[key] = parse_block(expected_indent + 2)
            else:
                out[key] = _parse_scalar(rest)
        return out

    def parse_list(expected_indent: int) -> list[Any]:
        nonlocal idx
        arr: list[Any] = []
        while idx < len(raw_lines):
            line = raw_lines[idx]
            if line.indent < expected_indent:
                break
            if line.indent > expected_indent:
                raise YamlMinError("unexpected indent jump")
            if not line.text.startswith("- "):
                break

            rest = line.text[2:].strip()
            idx += 1
            if rest == "":
                arr.append(parse_block(expected_indent + 2))
                continue

            if ":" in rest and not rest.startswith('"') and not rest.startswith("'"):
                k, v = rest.split(":", 1)
                item: dict[str, Any] = {k.strip(): _parse_scalar(v.strip()) if v.strip() else {}}

                while idx < len(raw_lines) and raw_lines[idx].indent == expected_indent + 2 and not raw_lines[idx].text.startswith("- "):
                    t = raw_lines[idx].text
                    if ":" not in t:
                        raise YamlMinError(f"expected key: value, got {t}")
                    k2, v2 = t.split(":", 1)
                    k2 = k2.strip()
                    v2 = v2.strip()
                    idx += 1
                    if v2 == "":
                        if idx < len(raw_lines) and raw_lines[idx].indent > expected_indent + 2:
                            item[k2] = parse_block(expected_indent + 4)
                        else:
                            item[k2] = {}
                    else:
                        item[k2] = _parse_scalar(v2)

                arr.append(item)
                continue

            arr.append(_parse_scalar(rest))

        return arr

    def parse_block(expected_indent: int) -> Any:
        if idx >= len(raw_lines):
            return {}
        line = raw_lines[idx]
        if line.indent != expected_indent:
            raise YamlMinError("unexpected indentation level")
        if line.text.startswith("- "):
            return parse_list(expected_indent)
        return parse_map(expected_indent)

    out = parse_block(0)
    if not isinstance(out, dict):
        raise YamlMinError("top-level must be a mapping")
    return out
