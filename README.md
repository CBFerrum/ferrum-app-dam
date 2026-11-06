# Digital Assets

Digital asset management for [Ferrum](https://github.com/CBFerrum/Ferrum), on Drive:
- assets kept as Drive files, with what may be done with them;
- thumbnails and previews of images, and what their files say of them;
- approval by a Brand Manager, of the file's version then;
- collections, and usage recorded within the rights.

| | |
|---|---|
| 🖼️ App | `dam`, native, in the server |
| 🧩 Package | `dam` (needs Drive) |
| 🖥️ Interface | its own, at `/dam/`: assets waiting for review, rights ending soon, and the approved library |
| 🔐 Roles | Brand Manager, Media Contributor, Media User |
| 📜 Licence | AGPL-3.0-only |

## Highlights

- **From Drive.** An asset is a file in Drive. Its type, content type, size and digest are read from
  the file, never typed in. A file in the trash is no asset.
- **Renditions.** Each image gets a thumbnail (320 pixels on its longest side) and a preview
  (1280 at most), turned the way the camera was held. They are JPEG, or PNG when the image has
  transparency, and they are seen by whoever can see the asset. A new version gets new ones, and
  the old ones are removed.
- **What the file says.** An image's size in pixels, when it was taken, the camera, and the
  creator and copyright its EXIF names are kept on the asset, and assets can be found by them.
  Where a photo was taken is never kept: the asset only says that the file records it, so the
  place can be removed before the file goes out.
- **Rights.** A licensed asset names its licensor and the last day it may be used. A Creative
  Commons one names whom it credits. Editorial-only assets are flagged, and releases for anyone
  recognisable are recorded.
- **Approval by version.** A Brand Manager approves an asset, or sends it back saying why; the
  approval is of the file's version then. When the file gets a new version, upkeep sends the asset
  back for review. An approved asset keeps its file and rights.
- **Usage within the rights.** A usage is of an approved asset, of the version approved, on a day
  no later than its last. An editorial-only asset is not used to sell (advertising or social).
- **Expiry.** Past its last day, an approved asset expires.
- **Shared outside.** A Brand Manager shares an approved asset with someone (a newspaper, a printer,
  an agency) for a use within its rights, until a day no later than its last, for a number of
  downloads (20 unless set). The link is shown once; only a digest of it is kept, and the share is
  recorded as a use. Opened by anyone holding it, it gives the version approved, as the stored file.
  It works only while the link is not stopped, its day has not passed and its downloads last, and
  while the asset stays approved with that version. Otherwise it answers as an unknown link does.
- **Personal data.** Requests reach who keeps a collection, approves, uses and shares an asset.
- **Who sees what.** Media Users see approved assets only, and their own usages.

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
- renditions and EXIF of a photo held on its side, a transparent mark, and a PDF without any, made
  again for a new version and seen only by those who can see the asset;
- who sees and does what.

## Not yet

- Renditions of video, PDFs and documents, crops, and other formats on request.
- XMP and IPTC metadata, and EXIF beyond its basics (exposure, lens).
- Images over 40 megapixels or 50 MB get no renditions.
- Sharing a whole collection by one link, and watermarks.
- Usage counted from where an asset is embedded, rather than recorded.

## Changelog

| Version | Highlights |
|---|---|
| **0.3.1** | Rebuilt with Ferrum-Frontend 24c3951: times on the workspace's clock; drafts headed as drafts. |
| **0.3.0** | Approved assets shared outside by links shown once, giving the version approved while the link and the asset allow; personal data declared. |
| **0.2.1** | Rebuilt with Ferrum-Frontend bb60115: empty numbers read as the server reads them; settings open as their record. |
| **0.2.0** | Thumbnails and previews of images, turned as the camera was held; their size, when and with what they were taken, and the creator and copyright their EXIF names. |
| **0.1.4** | Rebuilt with Ferrum-Frontend 00dea07: a workflow state's kept fields are read-only, its needed ones required. |
| **0.1.3** | Rebuilt with Ferrum-Frontend daeadda: actions say what they did. |
| **0.1.2** | Rebuilt with Ferrum-Frontend 80ff179: labels name their fields, and linked titles show their titles. |
| **0.1.1** | Its roles are Media Contributor and Media User, apart from fixed assets' Asset User. |
| **0.1.0** | Assets kept in Drive with their rights, approved by version, collections, and usage kept within the rights (FERRUM-ECOSYSTEM-012). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
