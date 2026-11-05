# Changelog

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
