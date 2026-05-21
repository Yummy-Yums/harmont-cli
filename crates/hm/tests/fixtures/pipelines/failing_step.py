"""Pipeline whose first step exits non-zero — used to test failure propagation."""
import harmont as hm


@hm.pipeline("failing-step", default_image="alpine:3.20")
def failing_step() -> hm.Step:
    return hm.sh("exit 7", label="boom", image="alpine:3.20")
