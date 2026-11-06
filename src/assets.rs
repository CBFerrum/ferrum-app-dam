//! Assets and their usage. An asset is a Drive file with its rights: what it is is read from the
//! file, never typed in. A licensed asset names its licensor and the last day it may be used; a
//! Creative Commons one names whom it credits. Approval is a Brand Manager's, of the file's version
//! then; a new version of the file sends the asset back for review, and its last day passing
//! expires it. A usage is of an approved asset, of the version approved, within its days, and an
//! editorial-only asset is not used to sell.

use crate::obj;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use ferrum_domain::util::{elevated, id, s};
use ferrum_kernel::{Ctx, Doc, DocHook, Error, HookCtx, Kernel, Result, Tx};
use serde_json::{Value, json};
use uuid::Uuid;

fn decides(ctx: &Ctx) -> bool {
    ctx.is_admin || ctx.roles.iter().any(|r| r == "Brand Manager")
}

/// The Drive file's type, content type, size, digest and version.
async fn file_facts(
    tx: &mut Tx,
    tenant: Uuid,
    file: Option<Uuid>,
) -> Result<Option<(Option<String>, Option<String>, Option<i64>, Option<String>, Option<i64>)>> {
    Ok(sqlx::query_as(
        "select file_type, content_type, size, sha256, version_no from e_drive_file where tenant_id = $1 and id = $2 and not coalesce(trashed, false)",
    )
    .bind(tenant)
    .bind(file)
    .fetch_optional(&mut **tx)
    .await?)
}

pub struct AssetHook;

#[async_trait]
impl DocHook for AssetHook {
    fn name(&self) -> &'static str {
        "dam.asset"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["kind", "content_type", "size", "sha256"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        if let Some((kind, ctype, size, sha, _)) = file_facts(hc.tx, hc.ctx.tenant, id(doc, "drive_file")).await? {
            doc.insert("kind".into(), json!(kind));
            doc.insert("content_type".into(), json!(ctype));
            doc.insert("size".into(), json!(size));
            doc.insert("sha256".into(), json!(sha));
        }
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        if file_facts(hc.tx, hc.ctx.tenant, id(doc, "drive_file")).await?.is_none() {
            return Err(Error::field("drive_file", "is not a file in Drive, or it is in the trash"));
        }
        match s(doc, "license") {
            Some("Licensed") if s(doc, "licensor").is_none() || s(doc, "usage_expires_on").is_none() => {
                Err(Error::field("licensor", "a licensed asset names its licensor and the last day it may be used"))
            }
            Some("Creative Commons") if s(doc, "licensor").is_none() => Err(Error::field("licensor", "a Creative Commons asset names whom it credits")),
            _ => Ok(()),
        }
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        if hc.ctx.is_system {
            return Ok(());
        }
        if let Some(o) = old
            && s(o, "status") == Some("Approved")
            && ["drive_file", "license", "licensor", "usage_expires_on", "territories"].iter().any(|k| s(o, k) != s(doc, k))
        {
            return Err(Error::invalid("an approved asset keeps its file and rights: send it back for review to change them"));
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, params: &Value) -> Result<Option<Value>> {
        let asset = id(doc, "id").ok_or_else(|| Error::internal("an asset without an id"))?;
        let system = elevated(hc.ctx);
        let values = match name {
            "submit_asset" => json!({"status": "In Review", "review_note": null}),
            "approve_asset" | "reject_asset" => {
                if !decides(hc.ctx) {
                    return Err(Error::Forbidden("a Brand Manager decides an asset".into()));
                }
                if name == "approve_asset" {
                    let version = file_facts(hc.tx, hc.ctx.tenant, id(doc, "drive_file")).await?.and_then(|f| f.4);
                    json!({"status": "Approved", "approved_by": hc.ctx.email, "approved_version": version})
                } else {
                    let why = params
                        .get("note")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|n| !n.is_empty())
                        .ok_or_else(|| Error::field("note", "sending it back says why"))?;
                    json!({"status": "Draft", "review_note": why})
                }
            }
            "archive_asset" => json!({"status": "Archived"}),
            "share_asset" => {
                if !decides(hc.ctx) {
                    return Err(Error::Forbidden("a Brand Manager shares an asset outside".into()));
                }
                return Ok(Some(crate::sharing::share(hc, doc, params).await?));
            }
            _ => return Ok(None),
        };
        let status = values["status"].clone();
        hc.kernel.set_values_tx(&system, hc.tx, "MediaAsset", asset, obj(values)).await?;
        Ok(Some(json!({"status": status})))
    }
}

pub struct UsageHook;

#[async_trait]
impl DocHook for UsageHook {
    fn name(&self) -> &'static str {
        "dam.usage"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["used_by"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        if s(doc, "used_by").is_none() && !hc.ctx.email.is_empty() {
            doc.insert("used_by".into(), json!(hc.ctx.email));
        }
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let asset =
            hc.kernel.get_tx(&elevated(hc.ctx), hc.tx, "MediaAsset", id(doc, "asset").ok_or_else(|| Error::field("asset", "is required"))?, false).await?;
        if s(&asset, "status") != Some("Approved") {
            return Err(Error::field("asset", "is not approved for use"));
        }
        let on = s(doc, "used_on").and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok()).ok_or_else(|| Error::field("used_on", "is required"))?;
        if s(&asset, "usage_expires_on").and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok()).is_some_and(|last| on > last) {
            return Err(Error::field("used_on", "is after the last day the asset may be used"));
        }
        let now = file_facts(hc.tx, hc.ctx.tenant, id(&asset, "drive_file")).await?.and_then(|f| f.4);
        if now != asset.get("approved_version").and_then(Value::as_i64) {
            return Err(Error::field("asset", "its file changed since it was approved"));
        }
        if s(&asset, "license") == Some("Editorial Only") && matches!(s(doc, "channel"), Some("Advertising" | "Social")) {
            return Err(Error::field("channel", "an editorial-only asset is not used to sell"));
        }
        Ok(())
    }

    async fn after_save(&self, hc: &mut HookCtx<'_>, doc: &Doc, _old: Option<&Doc>) -> Result<()> {
        let Some(asset) = id(doc, "asset") else { return Ok(()) };
        let n: i64 = sqlx::query_scalar("select count(*) from e_asset_usage where tenant_id = $1 and asset = $2")
            .bind(hc.ctx.tenant)
            .bind(asset)
            .fetch_one(&mut **hc.tx)
            .await?;
        hc.kernel.set_values_tx(&elevated(hc.ctx), hc.tx, "MediaAsset", asset, obj(json!({"usages": n}))).await
    }
}

/// Approved assets as of `today`: expired past their last day, and back to review when their file
/// has a version newer than the one approved.
pub(crate) async fn upkeep(k: &Kernel, ctx: &Ctx, tx: &mut Tx, today: NaiveDate) -> Result<()> {
    let approved: Vec<(Uuid, Option<NaiveDate>, Option<i64>, Option<i64>)> = sqlx::query_as(
        "select a.id, a.usage_expires_on, a.approved_version, f.version_no from e_media_asset a left join e_drive_file f on f.tenant_id = a.tenant_id and f.id = a.drive_file \
         where a.tenant_id = $1 and a.status = 'Approved'",
    )
    .bind(ctx.tenant)
    .fetch_all(&mut **tx)
    .await?;
    for (asset, last, approved_version, now) in approved {
        let values = if last.is_some_and(|l| l < today) {
            json!({"status": "Expired"})
        } else if now != approved_version {
            json!({"status": "In Review", "review_note": format!("The file has a new version since version {} was approved", approved_version.unwrap_or(0))})
        } else {
            continue;
        };
        k.set_values_tx(ctx, tx, "MediaAsset", asset, obj(values)).await?;
    }
    Ok(())
}

/// Today, for callers without a date of their own.
pub(crate) fn today() -> NaiveDate {
    Utc::now().date_naive()
}

pub struct ShareHook;

#[async_trait]
impl DocHook for ShareHook {
    fn name(&self) -> &'static str {
        "dam.share"
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        if name != "revoke_share" {
            return Ok(None);
        }
        if !decides(hc.ctx) {
            return Err(Error::Forbidden("a Brand Manager stops a shared link".into()));
        }
        let share = id(doc, "id").ok_or_else(|| Error::internal("a link without an id"))?;
        Ok(Some(crate::sharing::revoke(hc.kernel, &elevated(hc.ctx), hc.tx, share).await?))
    }
}
