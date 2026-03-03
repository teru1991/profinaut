from __future__ import annotations

import pytest

from libs.safety_core.errors import SecError
from libs.safety_core.lease_guard import LeaseGuard


def test_split_brain_denied() -> None:
    g = LeaseGuard()
    g.acquire(scope="bot:example:live", owner_id="A")
    with pytest.raises(SecError):
        g.acquire(scope="bot:example:live", owner_id="B")
