# Digital Assets

Digital asset management for [Ferrum](https://github.com/CBFerrum/Ferrum), on Drive:
- assets kept as Drive files, with what may be done with them;
- approval by a Brand Manager, of the file's version then;
- collections, and usage recorded within the rights.

| | |
|---|---|
| 🖼️ App | `dam`, native, in the server |
| 🧩 Package | `dam` (needs Drive) |
| 🖥️ Interface | its own, at `/dam/`: assets waiting for review, rights ending soon, and the approved library |
| 🔐 Roles | Brand Manager, Asset Contributor, Asset User |
| 📜 Licence | AGPL-3.0-only |

## Highlights

- **From Drive.** An asset is a file in Drive. Its type, content type, size and digest are read from
  the file, never typed in. A file in the trash is no asset.
- **Rights.** A licensed asset names its licensor and the last day it may be used. A Creative
  Commons one names whom it credits. Editorial-only assets are flagged, and releases for anyone
  recognisable are recorded.
- **Approval by version.** A Brand Manager approves an asset, or sends it back saying why; the
  approval is of the file's version then. When the file gets a new version, upkeep sends the asset
  back for review. An approved asset keeps its file and rights.
- **Usage within the rights.** A usage is of an approved asset, of the version approved, on a day
  no later than its last. An editorial-only asset is not used to sell (advertising or social).
- **Expiry.** Past its last day, an approved asset expires.
- **Who sees what.** Asset Users see approved assets only, and their own usages.

## Develop

```bash
cargo test                       # needs FERRUM_DAM_TEST_DATABASE_URL's database
cd ui && npm run build && npm run check
```

`tests/dam_it.rs` walks a brand's library with Drive installed:
- files uploaded to Drive made assets;
- rights refused when incomplete;
- review, a rejection, and approval;
- usages within and outside the rights;
- a new version back to review, and expiry;
- who sees and does what.

## Not yet

- Renditions (thumbnails, crops, other formats), and previews beyond Drive's.
- Metadata read from inside the file (EXIF, XMP), and finding by it.
- Sharing a collection outside the workspace, with watermarks.
- Usage counted from where an asset is embedded, rather than recorded.

## Changelog

| Version | Highlights |
|---|---|
| **0.1.0** | Assets kept in Drive with their rights, approved by version, collections, and usage kept within the rights (FERRUM-ECOSYSTEM-012). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
