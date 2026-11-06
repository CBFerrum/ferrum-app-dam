# Provenance

This app's design and code are its own. No code was copied from another project.

- Its shape follows common digital-asset-management practice: an asset with its rights and
  licence, approval before use, collections, usage records, and expiry of rights.
- The files are Ferrum Drive's, read through the platform's records; their versions are Drive's.
- Renditions use the `image` crate (MIT or Apache-2.0) to read, turn, scale and write images, and
  EXIF is read with `kamadak-exif` (BSD-2-Clause). Both are dependencies; no code of theirs was
  copied.
- A PDF's first page is drawn by `hayro` (Apache-2.0 or MIT), a dependency, with its
  `embed-fonts` feature: the standard PDF fonts it carries are PDFium's (BSD-3-Clause, their
  licence in the crate's `assets/LICENSE_FOXIT`). No code of theirs was copied.
