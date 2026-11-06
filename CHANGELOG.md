# Changelog

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
