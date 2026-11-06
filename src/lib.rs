//! Digital asset management (FERRUM-ECOSYSTEM-012) on Drive: assets with their rights, approved
//! by version, collections, and usage kept within the rights.
//!
//! * `assets`: what an asset is read from its Drive file; licences that name their licensor and
//!   days; approval by a Brand Manager of the file's version then; a new version back to review;
//!   expiry; usages only of the approved version, within its days, and editorial-only assets not
//!   used to sell.
//! * `renditions`: a thumbnail and a preview of each image, and what its file says of it (its size,
//!   when it was taken, the camera, the creator and copyright it names, whether it records where),
//!   made again for each new version.
//! * `sharing`: approved assets handed outside by links shown once, each a usage, giving the
//!   version approved while the link and the asset allow.
//!
//! This app's own design; no code was copied (docs/PROVENANCE.md). Hooks are registered per entity,
//! so a workspace without the `dam` package never runs them.

mod assets;
mod embedded;
mod renditions;
mod sharing;

use chrono::NaiveDate;
use ferrum_domain::reports::ReportSource;
use ferrum_kernel::{Ctx, Doc, Error, HookRegistry, Kernel, Result};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

include!(concat!(env!("OUT_DIR"), "/packages.rs"));
include!(concat!(env!("OUT_DIR"), "/ui.rs"));

/// This app's manifest (`app.yaml`).
pub const MANIFEST: &str = include_str!("../app.yaml");

/// This app's metadata packages, embedded at build time.
pub fn packages() -> std::result::Result<Vec<ferrum_meta::Package>, ferrum_meta::MetaError> {
    ferrum_meta::catalog::load_embedded(EMBEDDED)
}

/// Registers this app's document hooks, and the shared links' public route.
pub fn register(hooks: &mut HookRegistry) {
    hooks
        .register("MediaAsset", Arc::new(assets::AssetHook))
        .register("AssetUsage", Arc::new(assets::UsageHook))
        .register("AssetShare", Arc::new(assets::ShareHook));
    hooks.add_site("dam", Arc::new(sharing::DamSite));
}

/// This app's reports: none of its own yet.
pub fn reports() -> Vec<Arc<dyn ReportSource>> {
    vec![]
}

/// Upkeep: approved assets expired, or back to review when their file has changed; and renditions
/// of the assets whose file changed, a few each round.
pub async fn maintenance(k: &Kernel) -> Result<()> {
    maintenance_on(k, assets::today()).await
}

/// [`maintenance`] as of `today`.
pub async fn maintenance_on(k: &Kernel, today: NaiveDate) -> Result<()> {
    let mut tx = k.db.system_tx().await?;
    let tenants: Vec<Uuid> = sqlx::query_scalar("select id from tenant where status = 'active' order by id").fetch_all(&mut *tx).await?;
    tx.commit().await?;
    let mut first_error: Option<Error> = None;
    for tenant in tenants {
        let run = async {
            let schema = k.registry.schema(tenant).await?;
            if !schema.packages.iter().any(|p| p == "dam") {
                return Ok(());
            }
            let ctx = Ctx::system(tenant, schema);
            let mut tx = k.db.tenant_tx(tenant).await?;
            assets::upkeep(k, &ctx, &mut tx, today).await?;
            tx.commit().await?;
            renditions::round(k, &ctx).await?;
            Ok::<(), Error>(())
        };
        if let Err(e) = run.await {
            first_error.get_or_insert(e);
        }
    }
    first_error.map_or(Ok(()), Err)
}

/// A document from JSON.
pub(crate) fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interface_is_a_release_the_platform_takes() {
        let m = ferrum_apps::AppManifest::parse(MANIFEST).expect("app.yaml");
        assert_eq!(m.routes(), vec![("dam".to_string(), vec![])]);
        let files = UI.iter().map(|(p, b)| (p.to_string(), b.to_vec())).collect();
        let ui = ferrum_apps::ui::prepare("dam", files).expect("ui/dist is an interface release");
        assert!(UI.iter().any(|(p, _)| *p == "index.html") && !UI.iter().any(|(p, _)| p.starts_with('.')), "{}", ui.digest);
    }
}
