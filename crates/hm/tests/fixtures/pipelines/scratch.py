"""Single-step pipeline: simplest end-to-end test."""
import harmont as hm


@hm.pipeline("scratch", default_image="alpine:3.20")
def scratch() -> hm.Step:
    return hm.sh("echo hi", label="hi", image="alpine:3.20")
