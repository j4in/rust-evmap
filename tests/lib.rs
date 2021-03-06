extern crate evmap;

#[test]
fn it_works() {
    let x = ('x', 42);

    let (r, mut w) = evmap::new();

    // the map is uninitialized, so all lookups should return None
    assert_eq!(r.get_and(&x.0, |rs| rs.len()), None);

    w.refresh();

    // after the first refresh, it is empty, but ready
    assert_eq!(r.get_and(&x.0, |rs| rs.len()), None);
    // since we're not using `meta`, we get ()
    assert_eq!(r.meta_get_and(&x.0, |rs| rs.len()), Some((0, ())));

    w.insert(x.0, x);

    // it is empty even after an add (we haven't refresh yet)
    assert_eq!(r.get_and(&x.0, |rs| rs.len()), None);
    assert_eq!(r.meta_get_and(&x.0, |rs| rs.len()), Some((0, ())));

    w.refresh();

    // but after the swap, the record is there!
    assert_eq!(r.meta_get_and(&x.0, |rs| rs.len()), Some((1, ())));
    assert_eq!(r.get_and(&x.0, |rs| rs.len()), Some(1));
    assert_eq!(r.get_and(&x.0, |rs| rs.iter().any(|v| v.0 == x.0 && v.1 == x.1)),
               Some(true));
}

#[test]
fn busybusybusy() {
    use std::thread;

    let n = 10000;
    let (r, mut w) = evmap::new();
    thread::spawn(move || for i in 0..n {
        w.insert(i, true);
        w.refresh();
    });

    for i in 0..n {
        let i = i.into();
        loop {
            match r.get_and(&i, |rs| rs.len()) {
                Some(0) => continue,
                Some(1) => break,
                Some(i) => assert_ne!(i, 1),
                None => continue,
            }
        }
    }
}

#[test]
fn minimal_query() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.refresh();
    w.insert(1, "b");

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(1));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"a")).unwrap());
}

#[test]
fn non_copy_values() {
    let (r, mut w) = evmap::new();
    w.insert(1, format!("a"));
    w.refresh();
    w.insert(1, format!("b"));

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(1));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"a")).unwrap());
}

#[test]
fn non_minimal_query() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.refresh();
    w.insert(1, "c");

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(2));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"a")).unwrap());
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"b")).unwrap());
}

#[test]
fn absorb_negative_immediate() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.remove(1, "a");
    w.refresh();

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(1));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"b")).unwrap());
}

#[test]
fn absorb_negative_later() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.refresh();
    w.remove(1, "a");
    w.refresh();

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(1));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"b")).unwrap());
}

#[test]
fn absorb_multi() {
    let (r, mut w) = evmap::new();
    w.extend(vec![(1, "a"), (1, "b")]);
    w.refresh();

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(2));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"a")).unwrap());
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"b")).unwrap());

    w.remove(1, "a");
    w.insert(1, "c");
    w.remove(1, "c");
    w.refresh();

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(1));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"b")).unwrap());
}

#[test]
fn empty() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.insert(2, "c");
    w.empty(1);
    w.refresh();

    assert_eq!(r.get_and(&1, |rs| rs.len()), None);
    assert_eq!(r.get_and(&2, |rs| rs.len()), Some(1));
    assert!(r.get_and(&2, |rs| rs.iter().any(|r| r == &"c")).unwrap());
}

#[test]
fn empty_post_refresh() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.insert(2, "c");
    w.refresh();
    w.empty(1);
    w.refresh();

    assert_eq!(r.get_and(&1, |rs| rs.len()), None);
    assert_eq!(r.get_and(&2, |rs| rs.len()), Some(1));
    assert!(r.get_and(&2, |rs| rs.iter().any(|r| r == &"c")).unwrap());
}

#[test]
fn replace() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.insert(2, "c");
    w.update(1, "x");
    w.refresh();

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(1));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"x")).unwrap());
    assert_eq!(r.get_and(&2, |rs| rs.len()), Some(1));
    assert!(r.get_and(&2, |rs| rs.iter().any(|r| r == &"c")).unwrap());
}

#[test]
fn replace_post_refresh() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.insert(2, "c");
    w.refresh();
    w.update(1, "x");
    w.refresh();

    assert_eq!(r.get_and(&1, |rs| rs.len()), Some(1));
    assert!(r.get_and(&1, |rs| rs.iter().any(|r| r == &"x")).unwrap());
    assert_eq!(r.get_and(&2, |rs| rs.len()), Some(1));
    assert!(r.get_and(&2, |rs| rs.iter().any(|r| r == &"c")).unwrap());
}

#[test]
fn with_meta() {
    let (r, mut w) = evmap::with_meta::<usize, usize, _>(42);
    assert_eq!(r.meta_get_and(&1, |rs| rs.len()), None);
    w.refresh();
    assert_eq!(r.meta_get_and(&1, |rs| rs.len()), Some((0, 42)));
    w.set_meta(43);
    assert_eq!(r.meta_get_and(&1, |rs| rs.len()), Some((0, 42)));
    w.refresh();
    assert_eq!(r.meta_get_and(&1, |rs| rs.len()), Some((0, 43)));
}

#[test]
fn map_into() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.insert(2, "c");
    w.refresh();
    w.insert(1, "x");

    use std::collections::HashMap;
    let copy: HashMap<_, Vec<_>> = r.map_into(|k, vs| (k.clone(), vs.iter().cloned().collect()));

    assert_eq!(copy.len(), 2);
    assert!(copy.contains_key(&1));
    assert!(copy.contains_key(&2));
    assert_eq!(copy[&1].len(), 2);
    assert_eq!(copy[&2].len(), 1);
    assert_eq!(copy[&1], vec!["a", "b"]);
    assert_eq!(copy[&2], vec!["c"]);
}

#[test]
fn foreach() {
    let (r, mut w) = evmap::new();
    w.insert(1, "a");
    w.insert(1, "b");
    w.insert(2, "c");
    w.refresh();
    w.insert(1, "x");

    r.for_each(|&k, vs| match k {
        1 => assert_eq!(vs, &*vec!["a", "b"]),
        2 => assert_eq!(vs, &*vec!["c"]),
        _ => unreachable!(),
    });
}
