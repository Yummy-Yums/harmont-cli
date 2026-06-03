import type { Step, StepOptions } from "../step.js";
import { forever, onChange } from "../cache.js";
import { makeInstallChain, bunInstallCmd } from "./shared.js";

const APT_PACKAGES = ["curl", "ca-certificates", "unzip"] as const;
const VERSION_RE = /^[0-9]+\.[0-9]+(\.[0-9]+)?$/;

export interface BunOptions {
  readonly path?: string;
  readonly version?: string;
  readonly image?: string;
  readonly base?: Step;
}

type ActionOptions = Omit<StepOptions, "cwd">;

export class BunProject {
  readonly path: string;
  private readonly _installed: Step;

  constructor(path: string, installed: Step) {
    this.path = path;
    this._installed = installed;
  }

  install(): Step {
    return this._installed;
  }

  run(script: string, opts?: ActionOptions): Step {
    return this._installed.sh(`cd ${this.path} && bun run ${script}`, {
      label: `:bun: ${script}`,
      ...opts,
    });
  }

  test(opts?: ActionOptions): Step {
    return this._installed.sh(`cd ${this.path} && bun test`, {
      label: ":bun: test",
      ...opts,
    });
  }

  lint(opts?: ActionOptions): Step {
    return this.run("lint", opts);
  }

  fmt(opts?: ActionOptions): Step {
    return this.run("fmt", opts);
  }
}

export function bun(opts?: BunOptions): BunProject {
  const path = opts?.path ?? ".";
  const version = opts?.version;

  if (version != null && !VERSION_RE.test(version)) {
    throw new Error(
      `hm.bun: invalid version "${version}"\n  → use a semver version like "1.2.0" or "1.2"`,
    );
  }

  const bunInstalled = makeInstallChain({
    aptPackages: [...APT_PACKAGES],
    installCmd: bunInstallCmd(version),
    installCache: forever(),
    langTag: "bun",
    installTag: "install",
    image: opts?.image,
    base: opts?.base,
  });

  const bunDeps = bunInstalled.sh(`cd ${path} && bun install --frozen-lockfile`, {
    label: ":bun: deps",
    cache: onChange(`${path}/bun.lock`),
  });

  return new BunProject(path, bunDeps);
}
