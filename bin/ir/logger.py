#!/usr/bin/env python3
from logging import getLogger, INFO, StreamHandler, basicConfig
from json import dumps
from sys import argv, stderr

CLAIRE = "CLAIRE"


def get_investigation_logger(investigation_id: str, is_console_logger: bool):
    logger = getLogger()
    logger.setLevel(INFO)
    if is_console_logger:
        logger.addHandler(StreamHandler(stderr))

    return lambda msg: logger.info("{} {}".format(
        msg,
        dumps({
            "referrer": CLAIRE,
            "investigation_id": investigation_id
        }),
    ))


def main():
    basicConfig(stream=stderr, level=INFO)
    if 1 not in argv:
        print("Usage {} [investigation_id] [...]".format(argv[0]))
        exit(1)

    logger = get_investigation_logger(argv[1], is_console_logger=True)
    logger(argv)


if __name__ == "__main__":
    main()
