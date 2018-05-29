extern crate trk_io;

use trk_io::{ArraySequence, Point};

fn get_toy_streamlines() -> ArraySequence<Point> {
    ArraySequence::new(
        vec![2, 3, 3],
        vec![Point::new(1.0, 0.0, 0.0), // 1
             Point::new(2.0, 0.0, 0.0), // 1
             Point::new(0.0, 1.0, 0.0), // 2
             Point::new(0.0, 2.0, 0.0), // 2
             Point::new(0.0, 3.0, 0.0), // 2
             Point::new(0.0, 0.0, 1.0), // 3
             Point::new(0.0, 0.0, 2.0), // 3
             Point::new(0.0, 0.0, 3.0)] // 3
    )
}

#[test]
fn test_integers() {
    let arr = ArraySequence::new(
        vec![2, 3, 2, 1],
        vec![4, 5, 6, 7, 8, 9, 10, 11]);
    assert_eq!(arr.len(), 4);
    assert_eq!(arr.offsets, vec![0, 2, 5, 7, 8]);
}

#[test]
fn test_construction() {
    let streamlines = get_toy_streamlines();
    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines.offsets, vec![0, 2, 5, 8]);
}

#[test]
#[should_panic]
fn test_new_not_enough() {
    ArraySequence::new(vec![2], vec![Point::new(1.0, 0.0, 0.0)]);
}

#[test]
#[should_panic]
fn test_new_too_much() {
    ArraySequence::new(vec![2], vec![Point::new(1.0, 0.0, 0.0),
                                     Point::new(1.0, 0.0, 0.0),
                                     Point::new(1.0, 0.0, 0.0)]);
}

#[test]
fn test_empty() {
    let mut arr = ArraySequence::empty();
    assert_eq!(arr.is_empty(), true);
    assert_eq!(arr.len(), 0);

    for _ in 0..2 {
        arr.push(1);
        assert_eq!(arr.is_empty(), false);
        assert_eq!(arr.len(), 0);
    }

    arr.end_push();
    assert_eq!(arr.is_empty(), false);
    assert_eq!(arr.len(), 1);
}

#[test]
fn test_iterator() {
    let streamlines = get_toy_streamlines();
    let mut iter = streamlines.into_iter();
    assert_eq!(iter.next().unwrap(), [Point::new(1.0, 0.0, 0.0),
                                      Point::new(2.0, 0.0, 0.0)]);
    assert_eq!(iter.next().unwrap(), [Point::new(0.0, 1.0, 0.0),
                                      Point::new(0.0, 2.0, 0.0),
                                      Point::new(0.0, 3.0, 0.0)]);
    assert_eq!(iter.next().unwrap(), [Point::new(0.0, 0.0, 1.0),
                                      Point::new(0.0, 0.0, 2.0),
                                      Point::new(0.0, 0.0, 3.0)]);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_reverse_iterator() {
    let streamlines = get_toy_streamlines();
    let lengths = streamlines.iter().rev().map(
        |streamline| streamline.len()
    ).collect::<Vec<_>>();
    assert_eq!(lengths, vec![3, 3, 2]);
}

#[test]
fn test_iterator_mut() {
    let p0 = Point::origin();

    let mut streamlines = get_toy_streamlines();
    for (i, streamline) in streamlines.iter_mut().enumerate() {
        for p in streamline {
            if i % 2 == 0 {
                *p = p0;
            }
        }
    }

    let mut iter = streamlines.into_iter();
    assert_eq!(iter.next().unwrap(), [p0, p0]);
    assert_eq!(iter.next().unwrap(), [Point::new(0.0, 1.0, 0.0),
                                      Point::new(0.0, 2.0, 0.0),
                                      Point::new(0.0, 3.0, 0.0)]);
    assert_eq!(iter.next().unwrap(), [p0, p0, p0]);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_dynamic() {
    let mut arr = ArraySequence::empty();
    for i in 0..10 {
        assert_eq!(arr.nb_push_done(), i);
        arr.push(i);
        assert_eq!(arr.nb_push_done(), i + 1);
    }
    arr.end_push();
    assert_eq!(arr.nb_push_done(), 0);

    assert_eq!(arr.len(), 1);
    assert_eq!(arr.length_of_array(0), 10);
    assert_eq!(arr[0].len(), 10);
    assert_eq!(arr.offsets, vec![0, 10]);

    arr.extend(vec![11, 12, 13, 14, 15]);
    assert_eq!(arr.len(), 2);
    assert_eq!(arr.length_of_array(0), 10);
    assert_eq!(arr[0].len(), 10);
    assert_eq!(arr.length_of_array(1), 5);
    assert_eq!(arr[1].len(), 5);
    assert_eq!(arr.offsets, vec![0, 10, 15]);

    arr.extend_from_slice(&[20, 21, 22, 23]);
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[2].len(), 4);
    assert_eq!(arr.offsets, vec![0, 10, 15, 19]);
}

#[test]
fn test_empty_push() {
    let mut arr = ArraySequence::<f64>::empty();
    assert_eq!(arr.len(), 0);
    assert_eq!(arr.offsets, vec![0]);

    // An `end_push` without any `push` should do nothing
    arr.end_push();
    arr.end_push();

    assert_eq!(arr.len(), 0);
    assert_eq!(arr.offsets, vec![0]);
}

#[test]
fn test_filter() {
    let p = Point::new(1.0, 1.0, 1.0);
    let arr = ArraySequence::new(
        vec![2, 3, 2, 3],
        vec![p * 1.0, p * 2.0,
             p * 2.0, p * 3.0, p * 4.0,
             p * 3.0, p * 4.0,
             p * 4.0, p * 5.0, p * 6.0]);
    let filtered = arr.filter(&mut |arr: &[Point]| arr.len() == 3);
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0], [p * 2.0, p * 3.0, p * 4.0]);
    assert_eq!(filtered[1], [p * 4.0, p * 5.0, p * 6.0]);

    // Ensure that arr is still usable
    assert_eq!(arr.len(), 4);
}
