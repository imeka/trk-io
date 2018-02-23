# trk-io &emsp; [![Latest Version](https://img.shields.io/badge/crates.io-0.4.5-orange.svg)](https://crates.io/crates/trk-io) [![Build Status](https://travis-ci.org/imeka/trk-io.svg?branch=master)](https://travis-ci.org/imeka/trk-io) [![dependency status](https://deps.rs/repo/github/imeka/trk-io/status.svg)](https://deps.rs/repo/github/imeka/trk-io)

`trk-io` implements a `TrackVis` (.trk) reader and writer.

## Highlights

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

## Examples

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

## Roadmap

There's still a lot of work to do but it should work perfectly for simple use cases. In particular, future versions should be able to:

- Support TCK reading/writing
- Better handling of scalars and properties. There's currently a way to access them but I wouldn't call it conveniant. They are in the header so you need to zip over them yourself.
- Create some binary tools using this lib, e.g. show_affine, count_tracks, pruning, strip_info, color, etc.
- Support for `ops.Range`, e.g. `streamlines[0..10]`

Your help is much appreciated. Consider filing an [issue](https://github.com/imeka/trk-io/issues) in case something is missing for your use case to work. Pull requests are also welcome.
