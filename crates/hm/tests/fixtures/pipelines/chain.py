"""Linear chain: each step builds_in its predecessor; the chained
filesystem state must be visible (test -f /tmp/a)."""
import harmont as hm


@hm.pipeline("chain", default_image="alpine:3.20")
def chain() -> hm.Step:
    a = hm.sh("touch /tmp/a && echo a", label="a", image="alpine:3.20")
    b = a.sh("test -f /tmp/a && echo b-saw-a", label="b")
    return b.sh("test -f /tmp/a && echo c-also-saw-a", label="c")
