<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/banner-dark.svg">
  <img alt="Ferrum Digital Assets: brand files, rights and approval in one library" src="docs/assets/banner-light.svg" width="100%">
</picture>

<br>

[![Version](https://img.shields.io/badge/version-0.8.9-c2410c?style=flat-square)](CHANGELOG.md)
[![Licence: AGPL-3.0](https://img.shields.io/badge/licence-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![Ferrum platform](https://img.shields.io/badge/runs%20on-Ferrum-2f5bea?style=flat-square)](https://github.com/CBFerrum/Ferrum)
![Rust](https://img.shields.io/badge/Rust-server-b7410e?style=flat-square&logo=rust)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-data-336791?style=flat-square&logo=postgresql&logoColor=white)
![React](https://img.shields.io/badge/React%20%2B%20TypeScript-interface-0ea5e9?style=flat-square&logo=react&logoColor=white)
![Tests](https://img.shields.io/badge/tests-12%20Rust-16a34a?style=flat-square)

### Keep brand files with what you may do with them, and let only approved ones out.

Ferrum Digital Assets is for marketing and brand teams that keep images, videos and documents in Drive and
need to know, for each file, who approved it, whose rights it carries and until when. It adds approval,
rights, previews, file metadata, collections, usage records and outside sharing on top of Drive.

[💡 Why](#-why-ferrum-digital-assets) · [📸 Screenshots](#-screenshots) · [✨ Features](#-features) · [📊 Compare](#-how-it-compares) · [📍 Roadmap](#-roadmap) · [🚀 Install](#-install) · [🔧 Develop](#-develop)

</div>

## 💡 Why Ferrum Digital Assets

- 📁 **The files stay in Drive.** An asset is a file in Ferrum Drive. Its type, size and SHA-256 are read from the file, never typed in, and a file in the trash is no asset.
- ⚖️ **Rights are checked, not just noted.** A usage is refused for an asset that is not approved, for a version other than the approved one, after the asset's last day, or for selling when the asset is editorial-only.
- ✅ **Approval is of a file version.** A new version of the file sends an approved asset back to review.
- 🔗 **Outside sharing with an end date.** A link is shown once, only its digest is kept, it stops on its day or after its downloads, and a picture can carry a watermark.
- 🏠 **Self-hosted and open source.** AGPL-3.0, on your own PostgreSQL and Drive. No per-seat or per-storage licence fee from Ferrum.
- 🤝 **Same login and people as the rest of Ferrum.** Roles, audit trail, personal-data requests and the REST and MCP APIs come with the platform.
- 🌍 **English, Arabic (right to left) and Hindi.**

## 🎯 At a glance

| | |
|---|---|
| **Version** | 0.8.9 (package `dam` 0.1.1) |
| **Kind** | Its own interface at `/dam/` (home, library with thumbnails, an asset seen whole) plus Ferrum's standard lists and forms for every record |
| **What it adds** | 4 entities, 3 roles, 1 menu with 4 entries, 1 create shortcut, 1 assistant guide; no reports or print formats yet |
| **Depends on** | Ferrum platform `>=0.3.2, <0.4`; Ferrum Drive `>=0.12` (installing Digital Assets installs Drive) |
| **Works with** | Drive (the files), the platform's personal-data requests |
| **Languages** | English, Arabic, Hindi |
| **Tests** | 12 Rust tests against a real PostgreSQL and a Drive; interface checks in CI |
| **Licence** | AGPL-3.0-only |

## 📸 Screenshots

<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/dam-home-dark.webp"><img alt="Digital assets — home" src="docs/assets/screens/dam-home-light.webp" width="100%"></picture>

<p align="center"><b>🏠 Home</b><br><sub>See what waits for review, which rights end soon, and the approved library with thumbnails.</sub></p>

<table>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/dam-media-asset-list-dark.webp"><img alt="Assets — list" src="docs/assets/screens/dam-media-asset-list-light.webp"></picture>
<p><b>🗃️ Assets</b><br><sub>Browse, search and filter assets by kind, status, licence and what the file says.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/dam-asset-collection-list-dark.webp"><img alt="Collections — list" src="docs/assets/screens/dam-asset-collection-list-light.webp"></picture>
<p><b>🗂️ Collections</b><br><sub>Browse the collections that group assets for a campaign or a team.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/dam-media-asset-detail-dark.webp"><img alt="Asset — open record" src="docs/assets/screens/dam-media-asset-detail-light.webp"></picture>
<p><b>📇 An asset&#x27;s record</b><br><sub>Read an asset&#x27;s rights, approval, renditions and file metadata, with its history and related records.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/dam-asset-view-dark.webp"><img alt="Digital assets — an asset" src="docs/assets/screens/dam-asset-view-light.webp"></picture>
<p><b>👁️ An asset seen whole</b><br><sub>Preview an asset with its licence, usage and sharing.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/dam-collection-list-dark.webp"><img alt="Digital assets — collections" src="docs/assets/screens/dam-collection-list-light.webp"></picture>
<p><b>📚 Collections page</b><br><sub>Group assets into collections for a campaign or a team.</sub></p>
</td>
<td width="50%"></td>
</tr>
</table>

<table>
<tr>
<td align="center" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/dam-home-phone-dark.webp"><img alt="Digital assets — home" src="docs/assets/screens/dam-home-phone-light.webp" width="260"></picture>
<p><b>📱 Home (phone)</b><br><sub>See what waits for review, which rights end soon, and the approved library with thumbnails.</sub></p>
</td>
<td align="center" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/dam-media-asset-list-phone-dark.webp"><img alt="Assets — list" src="docs/assets/screens/dam-media-asset-list-phone-light.webp" width="260"></picture>
<p><b>📱 Assets (phone)</b><br><sub>Browse, search and filter assets by kind, status, licence and what the file says.</sub></p>
</td>
</tr>
</table>

## ✨ Features

### 📁 Assets and collections

- **An asset is a Drive file with its rights.** Title, the Drive file, kind, content type, size and SHA-256 (read from the file), description, tags, collection.
- **Five statuses.** Draft, In Review, Approved, Expired, Archived.
- **Collections.** An Asset Collection has a name, an owner and a description, and groups assets for a campaign or a team.
- **Who sees what.** A Media User sees approved assets only and their own usages. A Media Contributor adds assets, submits them and records usages. A Brand Manager decides, shares and sees everything.

### ⚖️ Rights

- **Licence kind.** Owned, Licensed, Royalty Free, Creative Commons or Editorial Only.
- **Licensor and last day of use.** A licensed asset names its licensor and the last day it may be used; a Creative Commons one names whom it credits.
- **Territories** where the asset may be used.
- **Editorial only.** Flagged, and never used for advertising or social.
- **Releases held.** An asset whose file names the people it shows cannot be approved until their releases are held.
- **Expiry.** Past its last day, an approved asset expires. Home lists approved assets whose rights end within 30 days.

### ✅ Approval and usage

- **Submit for Review.** A contributor sends a draft to review.
- **Approve It.** A Brand Manager approves an asset; the approval is of the file's version at that moment, and who approved it is kept.
- **Send It Back.** Rejecting needs a note saying why; the asset returns to Draft with the note.
- **A new version goes back to review.** When the Drive file gets a new version, upkeep sends the asset back for review. An approved asset keeps its file and rights.
- **Archive It** to retire an asset.
- **Usage within the rights.** A usage records where an asset was used: channel (Web, Social, Print, Advertising, Email, Internal), day, placement and who. It must be of an approved asset, of the version approved, on a day no later than its last day. An editorial-only asset is not used to sell.

### 🖼️ Renditions and what the file says

- **Thumbnails and previews.** Each image gets a thumbnail (320 pixels on its longest side) and a preview (1280 at most), turned the way the camera was held. They are JPEG, or PNG when the image has transparency, and are seen by whoever can see the asset. Upkeep looks at up to 10 changed assets per round and removes the renditions of the version before.
- **PDFs and office documents.** A PDF's first page is drawn on white as its renditions (pure-Rust rasteriser). An office document that carries a picture of its first page gets its renditions from it (OpenDocument always does; Word, Excel and PowerPoint files when saved with one).
- **Limits are said, not hidden.** A file that is not an image, an image over 40 megapixels or 50 MB, an encrypted PDF, and a file that cannot be read each keep a note saying why there are no renditions. A video has no frame still: the server carries no video decoder.
- **EXIF.** Size in pixels, when it was taken (camera time), the camera, and the creator and copyright it names.
- **XMP and IPTC.** From any file up to 50 MB, XMP gives title, caption, creators, credit line, copyright and keywords; a JPEG's IPTC record fills what XMP leaves unsaid. The asset keeps Title, Caption and Keywords In The File, and the keywords become its tags when it has none.
- **Place, people and licence in the file.** The place shown (not where it was taken), the people shown, usage terms and a licence web address.
- **Location is never kept.** Where a photo was taken is not stored; the asset only says the file records it, and the library marks such photos, so the place can be removed before the file goes out.
- **Read as text, never run,** and made again for each new version.

### 👁️ Seen whole

- **A page for each asset** at `/dam/view/{asset}`: a video or sound plays, a PDF reads and a picture shows, from the file as it stands, with its licence, usage and sharing beside it.
- **A preview route** (`GET /api/app/dam/preview/{asset}`) answers the current version's file to whoever may see the asset, with their own rights. It serves PDF, MP4, WebM, Ogg and QuickTime video, common audio, and PNG, JPEG, GIF, WebP and AVIF pictures. Never HTML or SVG, which could run in the workspace's pages. It is shown inline and never cached.

### 🔗 Sharing outside the workspace

- **Share It Outside.** A Brand Manager shares an approved asset with a named recipient (a newspaper, a printer, an agency) for a channel within its rights, until a day no later than its last, for up to 1,000 downloads (20 unless set).
- **Link shown once.** At `/api/site/{workspace}/dam/s/{token}`; only the token's digest is kept. The share is recorded as a usage.
- **It works only while** it is not stopped, its day has not passed, its downloads last (counted under a lock, so two at once never pass them), and the asset stays approved with that version. Otherwise it answers as an unknown link does.
- **Stop It** ends a link.
- **Watermark.** A link may carry words (60 letters at most, such as Preview or the recipient's name) drawn across a picture it hands out, on a slant over the whole of it. A picture shared so never goes out unmarked, and one that cannot be read is refused. Its EXIF, including where it was taken, goes no further. Other files go as they are.

### 🏠 The library home

- **Waiting for review.** The queue a Brand Manager works through.
- **Rights ending in 30 days.** Approved assets about to expire.
- **The library.** Approved assets with thumbnails, Editorial only marked, and a mark on photos that record where they were taken.
- **New asset** from Drive.

### 🔐 Roles, personal data and assistant

- **Personal-data requests** reach who keeps a collection, approves, uses and shares an asset. Erasure takes the address off; the records stay. A request that gives a person's name lists the files showing them as possibly theirs, for a person to judge. A name alone never erases anything.
- **Assistant.** It knows an asset's states and rights and suggests "Whose rights expire this month?", "Which assets are waiting for review?" and "Where is this asset used?".

## 🧭 How it works

### From file to approved use

```mermaid
flowchart LR
    DR["File in Drive"] --> AS["Asset: rights and file facts"]
    AS -->|submit| RV["In Review"]
    RV -->|"Brand Manager approves"| AP["Approved (version kept)"]
    RV -->|"sent back with a note"| DF["Draft"]
    AP -->|"usage within rights"| US["Usage recorded"]
    AP -->|"share outside"| SH["Link, end date, downloads"]
    AP -->|"last day passes"| EX["Expired"]
    AP -->|"new file version"| RV
    classDef app fill:#c2410c,color:#fff,stroke:#c2410c
    class AS,AP app
```

An asset is made from a Drive file and carries its rights. A Brand Manager approves a version. Uses and shares are checked against the rights, and a new version or an end date takes the approval away.

### How it sits in Ferrum

```mermaid
flowchart LR
    UI["Digital Assets interface at /dam/"] --> API["REST, MCP, live updates"]
    API --> K["Kernel: permissions, row-level security, audit"]
    K --> DB[("PostgreSQL")]
    K <--> APP["Digital Assets: 4 entities, 3 roles"]
    APP --> DRV["Drive: files and versions"]
    APP --> REND["Renditions: image, PDF, office first page"]
    classDef app fill:#c2410c,color:#fff,stroke:#c2410c
    class APP app
```

## 🧱 Data model

| Entity | What it is | Key fields or states |
|---|---|---|
| MediaAsset | A Drive file with what may be done with it | Status (Draft, In Review, Approved, Expired, Archived), licence, licensor, last day, territories, releases held, approved version, renditions, file facts |
| AssetCollection | A group of assets | Name, owner, description |
| AssetUsage | Where an asset was used | Channel (Web, Social, Print, Advertising, Email, Internal), day, placement, used by |
| AssetShare | An approved asset handed outside by a link | Recipient, channel, last day, download limit, downloads, watermark, stopped |

### 👥 Roles

| Role | What it can do |
|---|---|
| Brand Manager | Approves assets, sends them back, archives them, shares outside, and sees where they went |
| Media Contributor | Adds assets from Drive with their rights, submits them, and records where they are used |
| Media User | Finds approved assets and records where they are used (sees approved assets and their own usages only) |

## 🔌 Integrations

| With | How | Needs |
|---|---|---|
| Ferrum Drive | The files, their versions and trash are Drive's | Drive 0.12 or later (installed with it) |
| Personal-data requests | Names and addresses are found and erased as declared | Platform |
| Outside people | Share links, with a watermark on pictures | None |
| REST, MCP, webhooks | Every record and action, and the preview route | API keys |

Libraries used for renditions: `image` and `kamadak-exif` for images and EXIF, `hayro` to draw a PDF's first page, `zip` to read an office document's first-page picture. See [docs/PROVENANCE.md](docs/PROVENANCE.md).

## 📊 How it compares

Compared with Bynder, Adobe Experience Manager Assets, Canto, Brandfolder and, as an open-source option, ResourceSpace.

| Capability | Ferrum Digital Assets | Bynder | Adobe AEM Assets | Canto | Brandfolder | ResourceSpace |
|---|:-:|:-:|:-:|:-:|:-:|:-:|
| Central library of images, video and documents | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Rights, licence and last day of use | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Use refused outside the rights | ✅ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 |
| Approval before use | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Approval of a file version, back to review on a new one | ✅ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 |
| Several approval steps or approvers per collection | ❌ | ✅ | ✅ | 🟡 | 🟡 | 🟡 |
| Releases held before a file showing people is approved | ✅ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 |
| Thumbnails and previews of images, PDFs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Video frame, transcodes, subtitles | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Embedded metadata read (EXIF, XMP, IPTC) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Hierarchical categories and custom field sets | 🟡 [1] | ✅ | ✅ | ✅ | ✅ | ✅ |
| Full-text search, saved searches and facets | 🟡 [2] | ✅ | ✅ | ✅ | ✅ | ✅ |
| Share an asset by link with an end date | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Watermark on shared pictures | ✅ | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Share a whole collection or a lightbox | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Brand portal for agencies and partners | ❌ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Download in a chosen size or format | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Crops, focal points, on-the-fly transforms | ❌ | ✅ | ✅ | 🟡 | 🟡 | 🟡 |
| Bulk upload with metadata mapping | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Duplicate detection | ❌ | ✅ | ✅ | ✅ | 🟡 | ✅ |
| AI tagging and similar-image search | ❌ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Brand guidelines hub | ❌ | ✅ | 🟡 | 🟡 | ✅ | ❌ |
| Plugins for Adobe, Figma, Canva, Office | ❌ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Usage counted where an asset is embedded | 🟡 [3] | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Rights-expiry reminders by mail | 🟡 [4] | ✅ | ✅ | ✅ | ✅ | ✅ |
| Analytics on views and downloads | 🟡 [5] | ✅ | ✅ | ✅ | ✅ | ✅ |
| Files live in the same system as documents and mail | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Self-hosted, open source | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| No per-seat or per-storage licence fee from the vendor | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |

Notes. 🟡 for a leader means a higher plan, an add-on, a workflow you configure, or only part of the feature.

1. Tags are one free-text field and a collection is one flat link; extra fields can be added in Studio.
2. Filters on the main fields, including file facts, and the platform's record search; no saved facets and no text search inside files.
3. A usage is recorded by a person (or a connected tool), not counted from where an asset is embedded.
4. Home lists rights ending within 30 days. A scheduled automation in Studio can mail an owner; nothing mails by default.
5. A share counts its downloads; recorded usages are counted. There is no report of views, downloads or unused assets yet.

### 🏆 Where Ferrum Digital Assets stands out

- **Rights enforced at the point of use.** A usage or a share is refused outside the rights, the approved version or the last day, and the channel is recorded.
- **Approval that follows the file.** A new version of the Drive file takes the approval away.
- **Careful with what files say.** Metadata is read as text and never run; location is never kept; releases are required when people are shown; a data request finds files showing a named person.
- **Safe outside links.** Shown once, digest only, limited by day and downloads, watermarked pictures never go out unmarked.
- **Open and self-hosted,** on the same Drive, login and audit trail as the rest of the business.

### 🚧 Where the leaders are still ahead

- **A full brand library:** categories, controlled vocabularies, saved searches, bulk upload, duplicate detection, collection sharing and brand portals.
- **Media processing:** download presets by size and format, crops, transforms, and video frames, transcodes and subtitles.
- **Connected work:** plugins for creative tools, and usage counted where an asset is embedded.
- **AI:** auto-tagging and similar-image search.
- **Analytics** on views, downloads and unused assets.

> [!NOTE]
> Leader columns are from public product information as of November 2026 and may change. Plans and add-ons differ; check each vendor.

## 📍 Roadmap

### ✅ Recently shipped

- [x] **0.8.9** — Menu icons chosen so none repeats more than three times; a lighter first page; cleaner print.
- [x] **0.8.8** — The redesigned interface on the upgraded design system; the home on the work-home pattern.
- [x] **0.8.7** — A personal-data request by name lists the files showing that person as possibly theirs.
- [x] **0.8.0** — A shared link's watermark drawn across the pictures it hands out.
- [x] **0.7.0** — A PDF's first page drawn as its renditions.
- [x] **0.6.0** — The place and people a file shows and its licence terms, read from XMP and IPTC; approval waits for releases.

### 🔜 Next up

**Library**
- [ ] Download or share in a chosen size and format (presets for web, print, social).
- [ ] Bulk upload with metadata mapping, bulk edit and bulk download as a zip.
- [ ] Share a whole collection or a lightbox by one link.
- [ ] Duplicate warning using the stored SHA-256.
- [ ] Hierarchical categories and custom field sets by asset type.

**Connected use**
- [ ] Pick assets from Social, Campaigns, Docs and Slides, and record the usage there automatically (a post or email cannot use an expired or editorial-only asset).
- [ ] A rights-expiry reminder mailed before the last day, by default.

**Insight**
- [ ] A report of downloads, usages and unused assets.

### 🔭 Later

- [ ] A video frame still, transcodes and subtitles (needs an external video tool).
- [ ] Crops, focal points and on-the-fly transforms.
- [ ] Full-text search, saved searches and facets; OCR.
- [ ] A brand guidelines hub: logos, colours, fonts, do and don't pages.
- [ ] A brand portal for agencies and partners.
- [ ] Several approval steps, and approvers per collection.
- [ ] AI tagging and similar-image search.
- [ ] Plugins for Adobe, Figma and Canva.
- [ ] EXIF beyond the basics (exposure, lens) and PLUS licensing fields.
- [ ] Renditions for images over 40 megapixels or 50 MB.
- [ ] A watermark on files other than pictures.
- [ ] Licence kinds and channels as editable lists.

## 🚀 Install

Digital Assets is compiled into the Ferrum server image and installed from **Apps** (`/store/`) by a workspace administrator; installing it installs Ferrum Drive (`>=0.12`). Self-hosters who build their own image enable the `dam` feature of the server. It needs Ferrum `>=0.3.2, <0.4`.

After installing:

1. Give people the roles **Brand Manager**, **Media Contributor** and **Media User**.
2. Upload files to Drive, then add them as assets (**New asset**) with their licence, licensor and last day.
3. Contributors submit assets; a Brand Manager approves them.
4. Make collections for campaigns or teams.
5. Optional: add a scheduled automation in Studio that mails an owner before an asset's last day.

## 🔧 Develop

Clone it next to the platform:

```text
work/
├── ferrum/            https://github.com/CBFerrum/Ferrum (with Ferrum-Frontend cloned into ferrum/web)
└── ferrum-app-dam/
```

Point the platform dependencies at your checkout (this file is git-ignored):

```toml
# .cargo/config.toml
[patch."https://github.com/CBFerrum/Ferrum"]
ferrum-apps = { path = "../ferrum/crates/apps" }
ferrum-domain = { path = "../ferrum/crates/domain" }
ferrum-kernel = { path = "../ferrum/crates/kernel" }
ferrum-meta = { path = "../ferrum/crates/meta" }
```

Its tests keep their files in [ferrum-app-drive](https://github.com/CBFerrum/ferrum-app-drive), which Cargo takes from GitHub. To work on Drive alongside, clone it into the same folder and add a `[patch."https://github.com/CBFerrum/ferrum-app-drive"]` entry pointing at `../ferrum-app-drive`.

The tests need PostgreSQL with an ordinary (non-superuser) role, so row-level security applies:

```bash
createuser ferrum --pwprompt && createdb -O ferrum ferrum_dam_test
FERRUM_DAM_TEST_DATABASE_URL=postgres://ferrum:ferrum@localhost:5432/ferrum_dam_test cargo test
```

The interface is in `ui/` and builds with Ferrum-Frontend's packages (checked out at `../ferrum/web`):

```bash
cd ui
npm run check
npm run typecheck
npm test
npm run build      # ui/dist
```

CI (`.github/workflows/ci.yml`) runs `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on PostgreSQL, then the interface check, typecheck and tests on Node 22.

`tests/dam_it.rs` walks a brand's library with Drive installed: files made assets, rights refused when incomplete, review, rejection and approval, usages within and outside the rights, a new version back to review, expiry, renditions and EXIF (a photo held on its side, a transparent mark, a PDF), XMP, a PDF's first page, an office document's first-page picture, previews and who may see them, and who sees and does what.

## 📂 Project layout

```text
app.yaml          the manifest: id, version, requirements, what it adds to Ferrum
packages/dam/     entities, roles, menu and personal-data declarations
src/              Rust: approval and usage hooks, sharing, renditions, preview route, watermark, upkeep
ui/               its own interface (React and Vite), built into ui/dist
tests/            integration tests against a real PostgreSQL and Drive
docs/             provenance of the libraries it uses, and screenshots
```

## 📜 Changelog

Every version is in [CHANGELOG.md](CHANGELOG.md). The latest is **0.8.9**.

## 🤝 Contributing

Issues and pull requests are welcome at [github.com/CBFerrum/ferrum-app-dam](https://github.com/CBFerrum/ferrum-app-dam/issues). Run the checks in the Develop section before you open one. Rebuild `ui/dist/` if you change the screens.

## 📄 License

[AGPL-3.0-only](LICENSE), like the platform.

<div align="center">

<sub>Part of <a href="https://github.com/CBFerrum/Ferrum">Ferrum</a>, the open business platform · Made with 🦀 Rust and 🐘 PostgreSQL</sub>

</div>
