#!/usr/bin/env python3
from logging import INFO, LoggerAdapter, getLogger

CLAIRE = "CLAIRE"


def get_logger(investigation_id: str):
    logger = getLogger(__name__)
    logger.setLevel(INFO)
    logger = LoggerAdapter(logger, {
        "referrer": CLAIRE,
        "investigation_id": investigation_id,
    })

    return logger.info
