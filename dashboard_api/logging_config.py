"""
JSON logging configuration.
"""
import logging
import sys

from pythonjsonlogger import jsonlogger


def setup_logging(service_name: str) -> None:
    """Configure JSON logging for the service."""
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)

    # Remove default handlers
    logger.handlers.clear()

    # JSON formatter
    formatter = jsonlogger.JsonFormatter(
        "%(asctime)s %(name)s %(levelname)s %(message)s", timestamp=True
    )

    # Console handler with JSON format
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(formatter)
    logger.addHandler(handler)

    # Add service name to all logs
    logging.info("Logging initialized", extra={"service": service_name})
