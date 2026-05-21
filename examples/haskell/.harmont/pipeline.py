"""Haskell example pipeline."""
from __future__ import annotations

import harmont as hm
from harmont.haskell import HaskellPackage, HaskellToolchain


@hm.target()
def ghc() -> HaskellToolchain:
    return hm.haskell(ghc="9.6.7")


@hm.target()
def project(ghc: hm.Target[HaskellToolchain]) -> HaskellPackage:
    return ghc.cabal(path=".")


@hm.pipeline(
    "ci",
    env={"CI": "true"},
    default_image="ubuntu:24.04",
    triggers=[hm.push(branch="main")],
)
def ci(project: hm.Target[HaskellPackage]) -> tuple[hm.Step, ...]:
    return (
        project.build(),
        project.test(),
        project.lint(),
        project.fmt(),
    )
