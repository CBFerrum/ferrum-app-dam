# Changelog

## 0.8.7

People shown in a file come up in personal-data requests by name (Ferrum's closure review, FEO-016). A media asset's People Shown In The File (from its XMP) is declared as a name field, so a request that gives a person's name lists the files showing them under "possibly theirs", for a person to judge: a name alone never erases anything. Needs Ferrum 937a494 or later, which matches a name within a list.

## 0.8.6

Rebuilt with Ferrum-Frontend c59c9a2: the names of its models, fields and manifest in Arabic and Hindi (123 each in ui/src/locales/names, which the platform serves to every page of the workspace, so a record's labels, the launcher and the bar read in the person's language), and the sentences its pages still showed in English (split around a link or a figure, built in code from a template or word by word) whole and translated (18 words each in ui/src/locales). Translated by the model, to be reviewed by native speakers.

## 0.8.5

Rebuilt with Ferrum-Frontend 155518c: the sentences its pages built in code (counts, dates, what a toast or a screen reader says) go through t() too, as whole sentences where code chose a word, and carry built-in Arabic and Hindi (18 words each in ui/src/locales, translated by the model and to be reviewed by native speakers).

## 0.8.4

The assistant knows what its records mean in its pages (contributes.shell.assistant): their states, what counts and where figures come from, with questions it suggests asking there.

## 0.8.3

Rebuilt with Ferrum-Frontend 5b22e7b: its own words carry built-in Arabic and Hindi (18 words each, ui/src/locales, translated by the model and to be reviewed by native speakers), fetched for the page's language only. The labels it keeps in objects, its dialogs, its page titles and the words it keeps in variables go through t() as well, and the names and sizes it shows as they are keep their order right to left. Its pages are drawn once their words are in, and its entry carries only the parts of the kit it uses.

## 0.8.2

Rebuilt with Ferrum-Frontend 6537d65: an action's link, code or token shown once is kept in a dialog to copy, and on a page that reads right to left, English left untranslated keeps its own order. Its own words go through the kit's t() (17 places), so a workspace's own words for its language reach them, and Admin, Interface words lists them under it (dist/words.json).

## 0.8.1

Rebuilt with Ferrum-Frontend d5d98f2: a workspace's own words for its interface (Admin, Interface words) reach its pages as they load; an attachment field that holds a picture shows it; and a value another record already has is said so, where the form said the record had been changed by someone else.

## 0.8.0

A shared link may carry a watermark: words drawn across a picture it downloads, such as Preview or
the name it is for (Share It Outside → Watermark, 60 letters at most). They are white with a dark
edge, partly see-through, repeated on a slant over the whole picture, set in Helvetica by hayro
(already here for PDFs' first pages). A picture shared so never goes out unmarked: one that cannot
be read is refused. It is turned the way the camera was held first, and its EXIF goes no further,
where it was taken among it. Other files go as they are; letters outside Latin-1 show as `?`.

## 0.7.0

A PDF's renditions are its first page. The page is drawn on white with hayro, a PDF rasterizer in
pure Rust (Apache-2.0 or MIT; its standard fonts are PDFium's, BSD-3-Clause). The thumbnail is 320
pixels on the page's longest side and the preview 1280, made again for each new version and seen
by whoever sees the asset. An encrypted PDF, or one that cannot be read, keeps a note saying so.
A video's frame still has none: the server carries no video decoder.

## 0.6.4

Rebuilt with Ferrum-Frontend 63280d3: right to left, its pages mirror as they read (icons that point along the line, margins and alignment by the line's start and end, arrow keys along it), and the records pages' words are in Arabic and Hindi.

## 0.6.3

Rebuilt with Ferrum-Frontend a64416a: a new record's box whose default is 1 shows ticked, as it is saved.

## 0.6.2

Rebuilt with Ferrum-Frontend 922fcfb: an action's dialog says every refusal, the errors on what the action made (another record's rows) among them, and an action with parameters and a confirmation asks once.

## 0.6.1

Rebuilt with Ferrum-Frontend a76295e: its pages show times on a person's own clock when they chose one on their Account page, while today and due dates stay on the workspace's day. Its pages count today on the workspace's day, not UTC's, so the day's figures are right in the evening west of Greenwich and the early morning east of it.

## 0.6.0

The people and the place a file shows, and its licence (FERRUM-ECOSYSTEM-012).

- **Place Shown In The File:** from XMP's IPTC Extension Location Shown (sub-location, city,
  province or state, country), or else IPTC Core's location fields, or a JPEG's IPTC record (92, 90,
  95, 101). The location created, where it was taken, is never read.
- **People Shown In The File:** from XMP's IPTC Extension Person In Image. An asset whose file names
  people it shows is approved only once their releases are held (Releases Held).
- **Usage Terms** and **Licence In The File:** from XMP rights (`xmpRights:UsageTerms`, and
  `xmpRights:WebStatement` when it is a web address).
- Read as before: as text, never run, made again for each new version.

## 0.5.0

Files seen whole, and documents' first pages (FERRUM-ECOSYSTEM-012).

- **Seen whole.** Each asset opens on a page of its own, at `/dam/view/{asset}`: a video or a sound
  plays, a PDF reads and a picture shows, from the file as it stands. The library's titles open it,
  and it links to the record and what may be done with it.
- **The preview route.** `GET /api/app/dam/preview/{asset}` answers the current version's file to
  whoever may see the asset, read with their own rights, for the kinds a page shows by itself: PDF,
  MP4, WebM, Ogg and QuickTime video, common audio, and PNG, JPEG, GIF, WebP and AVIF pictures.
  Never HTML or SVG, which could run in the workspace's pages. It is shown inline, never cached,
  and read as its type only.
- **Documents' first pages.** An office document that carries a picture of its first page (ODF
  always does; Word, Excel and PowerPoint files when saved with one) gets its thumbnail and preview
  from it. Its size in pixels is not claimed: the picture is not the document.

## 0.4.0

What files say of themselves beyond EXIF (FERRUM-ECOSYSTEM-012).

- **XMP** from any file, whether a photo, a PDF or an office document: its title (the default
  language's), caption, creators, credit line, copyright and keywords.
- **IPTC** from a JPEG's Photoshop segment: the title, caption, by-line, credit, copyright notice and
  keywords, read as UTF-8 or else Latin-1. XMP is read first, and IPTC says only what it leaves
  unsaid.
- The asset keeps Title, Caption and Keywords In The File. Its creator and copyright come from EXIF
  first, then XMP or IPTC. Its keywords become its tags when it has none.
- Every file up to 50 MB is read for this, not only images; renditions are still made of images
  alone. Read as text and never run, with no new dependency.

## 0.3.1

Rebuilt with Ferrum-Frontend 24c3951: its pages show times on the workspace's clock, naming the zone when it is not the browser's, a draft numbered on submit is headed as a draft, and a record of an app that is not installed says so.

## 0.3.0

Shared outside the workspace (FERRUM-ECOSYSTEM-012).

- **Share It Outside:** a Brand Manager shares an approved asset with someone, for a use within its
  rights (an editorial-only asset never to sell), until a day no later than its last, for up to
  1,000 downloads (20 unless set). The link is shown once, and only its digest is kept. The share
  is recorded as a use of the asset.
- **The link:** at `/api/site/{workspace}/dam/s/{token}`, it gives the version approved, as the
  stored file, with nothing to cache or index. It works only while it is not stopped, its day has
  not passed and its downloads last (counted under a lock, so two at once never pass them), and
  the asset stays approved with that version and within its days. Otherwise it answers as an
  unknown link does. Stop It ends a link.
- **Personal data:** requests reach who keeps a collection, approves, uses and shares an asset.

## 0.2.1

Rebuilt with Ferrum-Frontend bb60115: a record page reads an empty number or check as the server does, so an action whose condition needs one shows, and a workspace's settings open as their one record.

## 0.2.0

Renditions, and what a file says of itself (FERRUM-ECOSYSTEM-012).

- Each round, upkeep looks at up to 10 assets whose file changed since they were last looked at
  (new assets and new versions). An image gets a thumbnail (320 pixels on its longest side) and a
  preview (1280 at most), turned the way the camera was held. They are JPEG, or PNG when the image
  has transparency. They are stored with the asset, so whoever can see the asset can see them.
  The renditions of the version before are removed.
- Its size in pixels, when it was taken (in the camera's own time), the camera, and the creator
  and copyright its EXIF names are kept on the asset, and assets can be found by them. Where it was
  taken is never kept; the asset says only that the file records it.
- A file that is not an image, an image over 40 megapixels or 50 MB, and a file that cannot be
  read as an image each keep a note saying why there are no renditions.
- The library shows thumbnails, and marks approved photos that record where they were taken.
- Rebuilt with Ferrum-Frontend 4a3681c.

## 0.1.4

Rebuilt with Ferrum-Frontend 00dea07: in a workflow state that names what may change, the other fields show read-only, and the fields the state needs show required.

## 0.1.3

Rebuilt with Ferrum-Frontend daeadda: after an action runs, a record page shows what the action said it did rather than only that it is done.

## 0.1.2

Rebuilt with Ferrum-Frontend 80ff179: a field's label names its control for assistive technology, and a record whose title is a link shows that record's title, not its id.

## 0.1.1

- Its roles are Media Contributor and Media User: Asset User was also a fixed-assets role, and roles
  of one name share their rights.

## 0.1.0

Digital asset management on Drive (FERRUM-ECOSYSTEM-012).

- Assets as Drive files, their type, size and digest read from the file.
- Rights: licences with their licensor and last day, Creative Commons credits, editorial-only,
  releases and territories.
- Approval by a Brand Manager of the file's version; a new version back to review.
- Usages only of the approved version, within the rights; editorial-only assets not used to sell.
- Expiry past the last day; collections.
- Its own interface at `/dam/`: the library.
