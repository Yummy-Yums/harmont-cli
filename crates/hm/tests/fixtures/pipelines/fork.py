"""Fork: two children of the same parent, both inherit its state."""
import harmont as hm


@hm.pipeline("fork", default_image="alpine:3.20")
def fork() -> tuple[hm.Step, hm.Step]:
    base = hm.sh("touch /tmp/x && echo base", label="base",
                            image="alpine:3.20")
    left = base.sh("test -f /tmp/x && echo left", label="left")
    right = base.sh("test -f /tmp/x && echo right", label="right")
    return (left, right)
