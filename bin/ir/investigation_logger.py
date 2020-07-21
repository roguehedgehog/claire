#!/usr/bin/env python3
from logging import getLogger, INFO, StreamHandler, basicConfig
from json import dumps
from sys import argv, stderr

CLAIRE = "CLAIRE"


def get_logger(investigation_id: str):
    logger = getLogger()
    logger.setLevel(INFO)
    if __name__ == "__main__":
        logger.addHandler(StreamHandler(stderr))

    return lambda msg, something=None: logger.info("{} {}".format(
        msg,
        dumps({
            "referrer": CLAIRE,
            "investigation_id": investigation_id
        }),
    ))


def to_json(o: object):
    return dumps(o, indent=2, skipkeys=True, default=str)


def log_to_console():
    basicConfig(stream=stderr, level=INFO)


def main():
    log_to_console()
    try:
        logger = get_logger(argv[1])
        logger(argv)
    except IndexError:
        print("Usage {} [investigation_id] [...]".format(argv[0]))


if __name__ == "__main__":
    main()
