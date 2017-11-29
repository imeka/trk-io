trk-io
======

The `trk-io` crate provides a `TrackVis`__  (.trk) reader and writer.

__ http://www.trackvis.org/docs/?subsect=fileformat

Highlights
----------

- Can read and write `TrackVis` files. Handles affine transformation as
  ``nibabel.streamlines`` and `MI-Brain`__ would.
- ``Reader`` can read all streamlines at once or can be used as a generator.
- Follows ``nibabel.streamlines`` architecture (all 3D points are in a single
  ``Vec![Point3D]``). Currently, this is only useful for performance, but it may
  lead to easier changes when and if we support BLAS.
  
  __ https://www.imeka.ca/mi-brain

TODOs
-----

- Add reading/writing tests to ensure that our highlights are true!
- Write new streamlines using a reference header
- Writing is not really practical because you need to have the complete
  streamlines before writing it. We need to be able to write the file per
  streamline.
- Handle colors. Can read a colored file but will ignore the color information.
  Idem for all scalars and properties.
- Support endianness.
