`trk-io` implements a `TrackVis` (.trk) reader and writer.

[![Latest Version](https://img.shields.io/badge/crates.io-0.4.2-orange.svg)](https://crates.io/crates/trk-io) [![Build Status](https://travis-ci.org/imeka/trk-io.svg?branch=master)](https://travis-ci.org/imeka/trk-io)

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
- Handles endianness.

Examples
--------

```rust
// Read complete streamlines to memory
let streamlines = Reader::new("bundle.trk").read_all();
for streamline in &streamlines {
    println!("Nb points: {}", streamline.len());
    for point in streamline {
        println!("{}", point);
    }
}
```
```rust
// Simple read/write. Using a generator (read one streamline at a time)
let reader = Reader::new("full_brain.trk");
let mut writer = Writer::new("copy.trk", Some(reader.header.clone()));
for streamline in reader.into_iter() {
    writer.write(streamline);
}
// The new file will be completed only at the end of the scope. The
// 'n_count' field is written in the destructor because we don't
// know how many streamlines the user will write.
```
