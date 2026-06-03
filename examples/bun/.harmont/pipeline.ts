import { pipeline, push, type PipelineDefinition } from "harmont";
import { bun } from "harmont/toolchains";

const project = bun({ path: "." });

const pipelines: PipelineDefinition[] = [
  {
    slug: "ci",
    triggers: [push({ branch: "main" })],
    pipeline: pipeline(project.run("build"), project.test(), project.run("lint"), {
      env: { CI: "true" },
      defaultImage: "ubuntu:24.04",
    }),
  },
];

export default pipelines;
