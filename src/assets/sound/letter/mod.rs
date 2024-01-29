
const A: &[u8] = include_bytes!("a.ogg");
const B: &[u8] = include_bytes!("b.ogg");
const C: &[u8] = include_bytes!("c.ogg");
const D: &[u8] = include_bytes!("d.ogg");
const E: &[u8] = include_bytes!("e.ogg");
const F: &[u8] = include_bytes!("f.ogg");
const G: &[u8] = include_bytes!("g.ogg");
const H: &[u8] = include_bytes!("h.ogg");
const I: &[u8] = include_bytes!("i.ogg");
const J: &[u8] = include_bytes!("j.ogg");
const K: &[u8] = include_bytes!("k.ogg");
const L: &[u8] = include_bytes!("l.ogg");
const M: &[u8] = include_bytes!("m.ogg");
const N: &[u8] = include_bytes!("n.ogg");
const O: &[u8] = include_bytes!("o.ogg");
const P: &[u8] = include_bytes!("p.ogg");
const Q: &[u8] = include_bytes!("q.ogg");
const R: &[u8] = include_bytes!("r.ogg");
const S: &[u8] = include_bytes!("s.ogg");
const T: &[u8] = include_bytes!("t.ogg");
const U: &[u8] = include_bytes!("u.ogg");
const V: &[u8] = include_bytes!("v.ogg");
const W: &[u8] = include_bytes!("w.ogg");
const X: &[u8] = include_bytes!("x.ogg");
const Y: &[u8] = include_bytes!("y.ogg");
const Z: &[u8] = include_bytes!("z.ogg");

pub fn letter_sound(ch: char) -> &'static [u8] {
    match ch {
        'a' => A,
        'b' => B,
        'c' => C,
        'd' => D,
        'e' => E,
        'f' => F,
        'g' => G,
        'h' => H,
        'i' => I,
        'j' => J,
        'k' => K,
        'l' => L,
        'm' => M,
        'n' => N,
        'o' => O,
        'p' => P,
        'q' => Q,
        'r' => R,
        's' => S,
        't' => T,
        'u' => U,
        'v' => V,
        'w' => W,
        'x' => X,
        'y' => Y,
        'z' => Z,
        _ => panic!("unknown character")
    }
}