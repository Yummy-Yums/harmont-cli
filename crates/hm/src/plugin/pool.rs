//! Instance pool for a loaded plugin.
//!
//! Each `LoadedPlugin` owns a `PluginPool`. Concurrent calls into the
//! plugin acquire an instance from the pool (creating one on demand
//! up to a pre-set max); when the call finishes, the instance returns
//! to the pool for reuse. Bounded by a `tokio::sync::Semaphore` so the
//! orchestrator's parallelism doesn't exceed `max_instances`.

// Pedantic-bucket nags accepted at module scope:
// - `missing_errors_doc`: every fallible fn returns `anyhow::Result`
//   with rich `context` messages.
// - `missing_panics_doc` on `PluginPool::from_*`: the only panic path
//   is the `try_lock().expect()` on a Mutex we just constructed; it
//   cannot be contended. Documenting it would be noise.
// - `expect_used`: same — these are on freshly-created Mutexes and
//   are infallible by construction.
// - `collapsible_if`: the nested `if g.len() < self.max_instances`
//   reads more clearly one rule per line.
// - `needless_pass_by_value` on `from_file(path: PathBuf, ...)`: we
//   clone the path into `bytes` AND store the original in the pool
//   field; passing by value avoids forcing every caller to clone.
//   Suppressed at the call site below.
// - `missing_const_for_fn`/`missing_panics_doc` on `PoolGuard::plugin`:
//   the `expect` lives on an `Option` we control; the guard contract
//   guarantees the plugin is present until drop.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::collapsible_if,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value
)]

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use extism::{Manifest as ExtismManifest, Plugin, Wasm};
use tokio::sync::{Mutex, Semaphore};

use super::host_fns;

#[derive(Debug, Clone)]
enum PluginBytes {
    Embedded(&'static [u8]),
    Disk(PathBuf),
}

#[derive(Debug)]
pub struct PluginPool {
    bytes: PluginBytes,
    instances: Mutex<Vec<Plugin>>,
    semaphore: Arc<Semaphore>,
    max_instances: usize,
    /// HTTPS hosts the plugin is permitted to contact via extism's
    /// HTTP host fn. Threaded into the per-instance
    /// [`ExtismManifest`] at spawn time. Empty means "no outbound
    /// HTTP", which is the default until the plugin's own manifest
    /// declares otherwise.
    allowed_hosts: Vec<String>,
}

impl PluginPool {
    pub fn from_bytes(bytes: &'static [u8], max_instances: usize) -> Result<Self> {
        Self::from_bytes_with_hosts(bytes, max_instances, Vec::new())
    }

    pub fn from_bytes_with_hosts(
        bytes: &'static [u8],
        max_instances: usize,
        allowed_hosts: Vec<String>,
    ) -> Result<Self> {
        let max_instances = max_instances.max(1);
        let pool = Self {
            bytes: PluginBytes::Embedded(bytes),
            instances: Mutex::new(Vec::with_capacity(max_instances)),
            semaphore: Arc::new(Semaphore::new(max_instances)),
            max_instances,
            allowed_hosts,
        };
        // Pre-instantiate one — the first acquire is the most latency-sensitive.
        let plugin = pool
            .spawn_instance()
            .context("preallocate first plugin instance")?;
        pool.instances
            .try_lock()
            .expect("just-created mutex is uncontended")
            .push(plugin);
        Ok(pool)
    }

    pub fn from_file(path: PathBuf, max_instances: usize) -> Result<Self> {
        Self::from_file_with_hosts(path, max_instances, Vec::new())
    }

    pub fn from_file_with_hosts(
        path: PathBuf,
        max_instances: usize,
        allowed_hosts: Vec<String>,
    ) -> Result<Self> {
        let max_instances = max_instances.max(1);
        let pool = Self {
            bytes: PluginBytes::Disk(path.clone()),
            instances: Mutex::new(Vec::with_capacity(max_instances)),
            semaphore: Arc::new(Semaphore::new(max_instances)),
            max_instances,
            allowed_hosts,
        };
        let plugin = pool.spawn_instance().with_context(|| {
            format!("preallocate first plugin instance from {}", path.display())
        })?;
        pool.instances
            .try_lock()
            .expect("just-created mutex is uncontended")
            .push(plugin);
        Ok(pool)
    }

    fn spawn_instance(&self) -> Result<Plugin> {
        let wasm = match &self.bytes {
            PluginBytes::Embedded(b) => Wasm::data(*b),
            PluginBytes::Disk(p) => Wasm::file(p),
        };
        let manifest =
            ExtismManifest::new([wasm]).with_allowed_hosts(self.allowed_hosts.iter().cloned());
        Plugin::new(&manifest, host_fns::all(), true).context("spawn extism plugin instance")
    }

    /// Acquire an instance. Returns a guard that holds the instance
    /// until dropped; on drop, the instance returns to the pool.
    ///
    /// If the pool is at capacity, blocks on the semaphore until a
    /// slot is freed.
    pub async fn acquire(&self) -> Result<PoolGuard<'_>> {
        // Reserve a slot.
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .context("semaphore closed")?;
        // Take an instance from the pool, or spawn a fresh one.
        let plugin = {
            let mut g = self.instances.lock().await;
            g.pop()
        };
        let plugin = if let Some(p) = plugin {
            p
        } else {
            self.spawn_instance()?
        };
        Ok(PoolGuard {
            pool: self,
            plugin: Some(plugin),
            _permit: permit,
        })
    }

    fn put_back(&self, plugin: Plugin) {
        // Best-effort: if the pool is full (more than max), drop on floor.
        // The semaphore guarantees we never have more than `max_instances`
        // outstanding, so the pool can hold up to `max_instances` safely.
        if let Ok(mut g) = self.instances.try_lock()
            && g.len() < self.max_instances
        {
            g.push(plugin);
        }
    }

    #[must_use]
    pub fn max_instances(&self) -> usize {
        self.max_instances
    }
}

#[derive(Debug)]
pub struct PoolGuard<'a> {
    pool: &'a PluginPool,
    plugin: Option<Plugin>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl PoolGuard<'_> {
    pub fn plugin(&mut self) -> &mut Plugin {
        self.plugin.as_mut().expect("plugin present until drop")
    }
}

impl Drop for PoolGuard<'_> {
    fn drop(&mut self) {
        if let Some(p) = self.plugin.take() {
            self.pool.put_back(p);
        }
    }
}
