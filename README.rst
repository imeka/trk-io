trk-io
======

The `trk-io` crate provides a `TrackVis`__  (.trk) reader and writer.

__ http://www.trackvis.org/docs/?subsect=fileformat

|build_status|_ |crates|_

.. |build_status| image:: https://travis-ci.org/imeka/trk-io.svg?branch=master
.. _build_status: https://travis-ci.org/imeka/trk-io

.. |crates| image:: https://img.shields.io/badge/crates.io-0.3.4-orange.svg
.. _crates: https://crates.io/crates/trk-io

Highlights
----------

- Can read and write `TrackVis` files. Handles affine transformation as
  ``nibabel.streamlines`` and `MI-Brain`__ would.
- Reading and writing is tested as much as in ``nibabel.streamlines``.
- ``Reader`` can read all streamlines at once or can be used as a generator.
- Write all at once or streamline per streamline.
- Follows ``nibabel.streamlines`` architecture (all 3D points are in a single
  ``Vec![Point3D]``). Currently, this is only useful for performance, but it may
  lead to easier changes when and if we support BLAS.
- Handles endianness.
  
  __ https://www.imeka.ca/mi-brain

TODOs
-----

- Add badges (build passing, crates.io)
- Do actual releases, not just publish
- Better handling of scalars and properties. There's currently a way to access
  them but I wouldn't call it conveniant. They are in the header so you need to
  zip over them yourself.
- Think about removing ``Streamlines::lengths`` as it seems only ``offsets`` is
  required. This is transparent to the user and not really important but it
  will same some memory.
- Create some binary tools using this lib, e.g. show_affine, count_tracks,
  pruning, strip_info, color, etc.
- Write documentation
- Support for ``ops.Range``, e.g. ``streamlines[0..10]``
