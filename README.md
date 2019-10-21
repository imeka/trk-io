# trk-io &emsp; [![Latest Version](https://img.shields.io/badge/crates.io-0.12.0-orange.svg)](https://crates.io/crates/trk-io) [![Build Status](https://travis-ci.org/imeka/trk-io.svg?branch=master)](https://travis-ci.org/imeka/trk-io) [![dependency status](https://deps.rs/repo/github/imeka/trk-io/status.svg)](https://deps.rs/repo/github/imeka/trk-io)

`trk-io` implements a `TrackVis` (.trk) reader and writer.

## Highlights

- Can read and write `TrackVis` files. Handles affine transformation as
  ``nibabel.streamlines`` and ``MI-Brain`` would.
- Reading and writing is tested as much as in ``nibabel.streamlines``.
- Can optionally use the ``nifti-rs`` crate, which can then be used to create a
  trk header from a ``NiftiHeader``, like you would do in ``nibabel``
- ``Reader`` can read all streamlines at once or can be used as a generator.
- Scalars and properties are supported when reading and writing trk. You can
  find some examples in ``trk_color.rs``.
- Write all at once or streamline per streamline.
- Follows ``nibabel.streamlines`` architecture (all 3D points are in a single
  ``Vec![Point3D]``). Currently, this is only useful for performance, but it may
  lead to easier changes when and if we support BLAS.
- Handles endianness.
- Some useful tools are coded in `examples/*.rs`. It's a good way to learn how
  to use this library.

## Examples

```rust
// Read complete streamlines to memory
let tractogram = Reader::new("bundle.trk").unwrap().read_all();
for streamline in &tractogram.streamlines {
    println!("Nb points: {}", streamline.len());
    for point in streamline {
        println!("{}", point);
    }
}
```
```rust
// Simple read/write. Using a generator, so it will load only
// one streamline in memory.
let reader = Reader::new("full_brain.trk").unwrap();
let mut writer = Writer::new(
    "copy.trk", Some(reader.header.clone()));
for tractogram_item in reader.into_iter() {
    // tractogram_item is a TractogramItem, which is a tuple of
    // (streamline, scalars, properties).
    writer.write(tractogram_item);
}
// The new file will be completed only at the end of the scope. The
// 'n_count' field is written in the destructor because we don't
// know how many streamlines the user will write.
```

## Roadmap

There's still a lot of work to do but it should work perfectly for simple use cases. In particular, future versions should be able to:

- Support TCK reading/writing
- Create some binary tools using this lib, e.g. show_affine, count_tracks, pruning, strip_info, etc.
- Support for `ops.Range`, e.g. `streamlines[0..10]`

Your help is much appreciated. Consider filing an [issue](https://github.com/imeka/trk-io/issues) in case something is missing for your use case to work. Pull requests are also welcome.
