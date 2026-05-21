//! Raw `host_fn!` imports for the `hm_docker_*` host fns. The
//! generic host fns from `hm_plugin_sdk::host` cover everything
//! else; this module covers the docker-specific surface.

use extism_pdk::*;
use hm_plugin_protocol::{DockerCommitArgs, DockerExecArgs, DockerExtractArgs, DockerStartArgs};

#[host_fn]
extern "ExtismHost" {
    pub fn hm_docker_ping() -> u32;
    pub fn hm_docker_image_exists(tag: String) -> u32;
    pub fn hm_docker_pull(tag: String);
    pub fn hm_docker_start_container(args: Json<DockerStartArgs>) -> String;
    pub fn hm_docker_extract_workspace(args: Json<DockerExtractArgs>);
    pub fn hm_docker_exec(args: Json<DockerExecArgs>) -> i32;
    pub fn hm_docker_commit(args: Json<DockerCommitArgs>) -> String;
    pub fn hm_docker_remove_image(tag: String);
    pub fn hm_docker_stop_remove(container_id: String);
}

// Safe wrappers.

#[allow(dead_code, reason = "host fn surface; not used by run_step yet")]
pub(crate) fn ping() -> bool {
    unsafe { hm_docker_ping() }.map(|n| n != 0).unwrap_or(false)
}

pub(crate) fn image_exists(tag: &str) -> bool {
    unsafe { hm_docker_image_exists(tag.to_string()) }
        .map(|n| n != 0)
        .unwrap_or(false)
}

pub(crate) fn pull(tag: &str) -> Result<(), Error> {
    unsafe { hm_docker_pull(tag.to_string()) }
}

pub(crate) fn start_container(args: DockerStartArgs) -> Result<String, Error> {
    unsafe { hm_docker_start_container(Json(args)) }
}

pub(crate) fn extract_workspace(args: DockerExtractArgs) -> Result<(), Error> {
    unsafe { hm_docker_extract_workspace(Json(args)) }
}

pub(crate) fn exec(args: DockerExecArgs) -> Result<i32, Error> {
    unsafe { hm_docker_exec(Json(args)) }
}

pub(crate) fn commit(args: DockerCommitArgs) -> Result<String, Error> {
    unsafe { hm_docker_commit(Json(args)) }
}

#[allow(dead_code, reason = "host fn surface; not used by run_step yet")]
pub(crate) fn remove_image(tag: &str) {
    let _ = unsafe { hm_docker_remove_image(tag.to_string()) };
}

pub(crate) fn stop_remove(container_id: &str) {
    let _ = unsafe { hm_docker_stop_remove(container_id.to_string()) };
}
