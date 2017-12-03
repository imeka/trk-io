trk-io
======

The `trk-io` crate provides a `TrackVis`__  (.trk) reader and writer.

__ http://www.trackvis.org/docs/?subsect=fileformat

Highlights
----------

- Can read and write `TrackVis` files. Handles affine transformation as
  ``nibabel.streamlines`` and `MI-Brain`__ would.
- Reading and writing is tested as much as in ``nibabel.streamlines``.
- ``Reader`` can read all streamlines at once or can be used as a generator.
  Handles endianness.
- Write all at once or streamline per streamline.
- Follows ``nibabel.streamlines`` architecture (all 3D points are in a single
  ``Vec![Point3D]``). Currently, this is only useful for performance, but it may
  lead to easier changes when and if we support BLAS.
  
  __ https://www.imeka.ca/mi-brain

TODOs
-----

- Still missing a writing test for Endianness.
- Handle colors. Can read a colored file but will ignore the color information.
  Idem for all scalars and properties.
- Open trk file only once.
- Remove ``Streamlines::lengths`` as it seems only ``offsets`` is required.
- Create some binary tools using this lib, e.g. show_affine, count_tracks,
  pruning, strip_info, color, etc.
- Support for ``ops.Range``, e.g. ``streamlines[0..10]``
