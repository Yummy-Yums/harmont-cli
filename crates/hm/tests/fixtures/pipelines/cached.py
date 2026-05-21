"""Cached step: re-running should be a hit on the second invocation."""
from datetime import timedelta

import harmont as hm


@hm.pipeline("cached", default_image="alpine:3.20")
def cached() -> hm.Step:
    t = hm.sh(
        "date +%s > /tmp/ts && cat /tmp/ts",
        label="t", image="alpine:3.20",
        cache=hm.ttl(timedelta(days=1)),
    )
    return t.sh("cat /tmp/ts", label="r")
