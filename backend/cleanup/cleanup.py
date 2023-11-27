import sqlite3
import tomllib as toml
import argparse
import datetime
import time

ROCKET_CONFIG = {}
REQ_SECTION = {}
DB_CONNECTION = ""
MY_EPOCH = None


def get_args():
    p = argparse.ArgumentParser(prog="req.tnhs.dev Cleanup", epilog="Have fun :)")
    p.add_argument(
        "rocket_config",
        type=argparse.FileType("rb"),
        nargs="?",
        default="./Rocket.toml",
    )
    return p.parse_args()


def main():
    args = get_args()
    ROCKET_CONFIG = toml.load(args.rocket_config)
    REQ_SECTION = ROCKET_CONFIG["default"]["req"]

    ep = REQ_SECTION["my_epoch"]
    MY_EPOCH = datetime.datetime.strptime(ep, "%Y-%m-%d %H:%M:%S")
    interval = float(REQ_SECTION["cleanup_interval"])
    max_age = int(REQ_SECTION["max_age"])

    DB_CONNECTION = ROCKET_CONFIG["default"]["databases"]["auth"]["url"]

    while True:
        cutoff = int(
            (
                (datetime.datetime.now() - datetime.timedelta(seconds=max_age))
                - MY_EPOCH
            ).total_seconds()
        )

        con = sqlite3.connect(DB_CONNECTION)
        cur = con.cursor()

        (to_delete,) = cur.execute(
            "SELECT COUNT(*) FROM auth WHERE ts < ?;", (cutoff,)
        ).fetchone()
        if to_delete > 0:
            print(f"Cleared {to_delete} sessions")
        cur.execute("DELETE FROM auth WHERE ts < ?;", (cutoff,)).fetchall()

        con.commit()
        con.close()
        time.sleep(interval)


if __name__ == "__main__":
    main()
