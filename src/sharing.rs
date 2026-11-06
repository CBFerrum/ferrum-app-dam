//! Assets handed outside the workspace. A Brand Manager shares an approved asset with someone, for a
//! use within its rights, until a day no later than its last: a link shown once, of which only a
//! digest is kept, and a usage recorded. Opened at `/api/site/{workspace}/dam/s/{token}` by anyone
//! holding it, it gives the version approved, as the stored file, while the link is not stopped,
//! its day has not passed, its downloads last, and the asset stays approved with that version; every
//! other case answers as an unknown link does.

use crate::obj;
use chrono::Utc;
use ferrum_domain::util::{elevated, id, s};
use ferrum_kernel::{Ctx, Doc, Error, HookCtx, Kernel, Result, Site, SiteRequest, SiteResponse, Tx};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Downloads a link allows when none are given, and the most it may.
const DOWNLOADS: i64 = 20;
const MOST_DOWNLOADS: i64 = 1000;
const CHANNELS: &[&str] = &["Web", "Social", "Print", "Advertising", "Email", "Internal"];

pub(crate) fn digest(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// A link's token: 64 hexadecimal characters from the system's random source.
fn token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// `share_asset` on an approved asset: the link, shown once.
pub(crate) async fn share(hc: &mut HookCtx<'_>, doc: &Doc, params: &Value) -> Result<Value> {
    let asset = id(doc, "id").ok_or_else(|| Error::internal("an asset without an id"))?;
    let recipient = params.get("recipient").and_then(Value::as_str).map(str::trim).filter(|r| !r.is_empty());
    let recipient = recipient.ok_or_else(|| Error::field("recipient", "says who the link is for"))?;
    let channel =
        params.get("channel").and_then(Value::as_str).filter(|c| CHANNELS.contains(c)).ok_or_else(|| Error::field("channel", "says what it is used for"))?;
    let until = params
        .get("expires_on")
        .and_then(Value::as_str)
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| Error::field("expires_on", "is the link's last day, YYYY-MM-DD"))?;
    let today = hc.ctx.today();
    if until < today {
        return Err(Error::field("expires_on", "is today or later"));
    }
    let downloads = params.get("max_downloads").and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(DOWNLOADS);
    if !(1..=MOST_DOWNLOADS).contains(&downloads) {
        return Err(Error::field("max_downloads", format!("is from 1 to {MOST_DOWNLOADS}")));
    }
    if s(doc, "usage_expires_on").and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()).is_some_and(|last| until > last) {
        return Err(Error::field("expires_on", "is after the last day the asset may be used"));
    }
    let system = elevated(hc.ctx);
    // A use like any other: within the asset's rights and days, of the version approved.
    hc.kernel
        .insert_tx(
            &system,
            hc.tx,
            "AssetUsage",
            obj(json!({"asset": asset.to_string(), "channel": channel, "used_on": today.to_string(), "placement": format!("Shared with {recipient} until {until}")})),
        )
        .await?;
    let token = token();
    hc.kernel
        .insert_tx(
            &system,
            hc.tx,
            "AssetShare",
            obj(json!({"asset": asset.to_string(), "recipient": recipient, "channel": channel, "expires_on": until.to_string(), "max_downloads": downloads,
                       "downloads": 0, "shared_by": hc.ctx.email.to_lowercase(), "revoked": false, "token_hash": digest(&token)})),
        )
        .await?;
    let slug: String = sqlx::query_scalar("select slug from tenant where id = $1").bind(hc.ctx.tenant).fetch_one(&mut **hc.tx).await?;
    let path = format!("/api/site/{slug}/dam/s/{token}");
    let link = match &hc.kernel.settings.public_url {
        Some(base) => format!("{}{path}", base.trim_end_matches('/')),
        None => path,
    };
    Ok(json!({"link": link, "works_until": until.to_string(), "downloads": downloads, "note": "Shown once: copy it now"}))
}

pub struct DamSite;

#[async_trait::async_trait]
impl Site for DamSite {
    async fn handle(&self, k: &Kernel, req: SiteRequest) -> Result<SiteResponse> {
        let segs: Vec<&str> = req.path.split('/').filter(|p| !p.is_empty()).collect();
        match (req.method.as_str(), segs.as_slice()) {
            ("GET", ["s", token]) => open(k, req.tenant, token).await,
            ("GET", ["preview", asset]) => crate::preview::preview(k, &req, asset).await,
            _ => Err(Error::NotFound(format!("dam endpoint {} {}", req.method, req.path))),
        }
    }
}

/// A link opened: the version approved, as the stored file, or what an unknown link answers.
async fn open(k: &Kernel, tenant: Uuid, token: &str) -> Result<SiteResponse> {
    let gone = || Error::NotFound("link".into());
    let ctx = Ctx::system(tenant, k.registry.schema(tenant).await?);
    let today = ctx.today();
    let mut tx = k.db.tenant_tx(tenant).await?;
    // Held while it is counted: two at once never pass its downloads.
    let share: Option<(Uuid, Uuid)> = sqlx::query_as(
        "select id, asset from e_asset_share where tenant_id = $1 and token_hash = $2 and not coalesce(revoked, false) and expires_on >= $3 \
         and coalesce(downloads, 0) < coalesce(max_downloads, $4) for update",
    )
    .bind(tenant)
    .bind(digest(token))
    .bind(today)
    .bind(DOWNLOADS)
    .fetch_optional(&mut *tx)
    .await?;
    let (share, asset) = share.ok_or_else(gone)?;
    // The asset still approved, within its days, its file still the version approved.
    let approved: Option<(Option<String>, Option<Uuid>, Option<i64>, Option<i64>, Option<chrono::NaiveDate>)> = sqlx::query_as(
        "select a.status, a.drive_file, a.approved_version, f.version_no, a.usage_expires_on from e_media_asset a \
         left join e_drive_file f on f.tenant_id = a.tenant_id and f.id = a.drive_file where a.tenant_id = $1 and a.id = $2",
    )
    .bind(tenant)
    .bind(asset)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((Some(status), Some(file), Some(version), current, last)) = approved else { return Err(gone()) };
    if status != "Approved" || current != Some(version) || last.is_some_and(|l| l < today) {
        return Err(gone());
    }
    let stored: Option<Option<String>> =
        sqlx::query_scalar("select stored_file from e_drive_file_version where tenant_id = $1 and file = $2 and version_no = $3")
            .bind(tenant)
            .bind(file)
            .bind(version)
            .fetch_optional(&mut *tx)
            .await?;
    let object: Uuid = stored.flatten().and_then(|o| o.parse().ok()).ok_or_else(gone)?;
    sqlx::query("update e_asset_share set downloads = coalesce(downloads, 0) + 1, last_downloaded_at = $3 where tenant_id = $1 and id = $2")
        .bind(tenant)
        .bind(share)
        .bind(Utc::now())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let info = k.file_info(&ctx, object).await?;
    let name: String = info.name.chars().filter(|c| !c.is_control() && *c != '"' && *c != '\\').collect();
    let headers = vec![
        ("content-disposition".to_string(), format!("attachment; filename=\"{name}\"")),
        ("cache-control".to_string(), "no-store".to_string()),
        ("x-robots-tag".to_string(), "noindex".to_string()),
    ];
    Ok(SiteResponse::file(200, &info.content_type, headers, object))
}

/// `revoke_share`: the link stops working.
pub(crate) async fn revoke(k: &Kernel, ctx: &Ctx, tx: &mut Tx, share: Uuid) -> Result<Value> {
    k.set_values_tx(ctx, tx, "AssetShare", share, obj(json!({"revoked": true}))).await?;
    Ok(json!({"revoked": true}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_and_their_digests() {
        let (a, b) = (token(), token());
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
        assert_eq!(digest(&a), digest(&a));
        assert_eq!(digest(&a).len(), 64);
        assert_ne!(digest(&a), a, "only a digest is kept");
    }
}
