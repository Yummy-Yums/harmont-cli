"""Cache attached to a non-leaf step in a chain."""
from datetime import timedelta

import harmont as hm


@hm.pipeline("mid-chain-cache", default_image="alpine:3.20")
def mid_chain_cache() -> hm.Step:
    a = hm.sh("echo a > /tmp/a", label="a", image="alpine:3.20")
    b = a.sh(
        "cat /tmp/a && date +%s > /tmp/b",
        label="b",
        cache=hm.ttl(timedelta(hours=1)),
    )
    return b.sh("cat /tmp/b", label="c")
