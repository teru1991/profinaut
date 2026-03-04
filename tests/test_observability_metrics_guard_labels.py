import pytest

from libs.observability.metrics_catalog import METRICS
from libs.observability.metrics_guard import validate_catalog, validate_labels


def test_catalog_labels_ok():
    validate_catalog(METRICS)


def test_forbidden_label_rejected():
    with pytest.raises(ValueError):
        validate_labels(["service", "symbol"])
