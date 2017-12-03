`trk-io` implements a `TrackVis` (.trk) reader and writer.

Highlights
----------

- Can read and write `TrackVis` files. Handles affine transformation as
  ``nibabel.streamlines`` and ``MI-Brain`` would.
- Reading and writing is tested as much as in ``nibabel.streamlines``.
- ``Reader`` can read all streamlines at once or can be used as a generator.
  Handles endianness.
- Write all at once or streamline per streamline.
- Follows ``nibabel.streamlines`` architecture (all 3D points are in a single
  ``Vec![Point3D]``). Currently, this is only useful for performance, but it may
  lead to easier changes when and if we support BLAS.
