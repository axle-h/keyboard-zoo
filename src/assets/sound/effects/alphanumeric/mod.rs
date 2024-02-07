use std::collections::HashMap;
use sdl2::mixer::Chunk;
use crate::assets::sound::LoadSound;
use crate::config::AudioConfig;

const ASSETS: [(char, &'static [u8]); 36] = [
    ('a', include_bytes!("a.ogg")),
    ('b', include_bytes!("b.ogg")),
    ('c', include_bytes!("c.ogg")),
    ('d', include_bytes!("d.ogg")),
    ('e', include_bytes!("e.ogg")),
    ('f', include_bytes!("f.ogg")),
    ('g', include_bytes!("g.ogg")),
    ('h', include_bytes!("h.ogg")),
    ('i', include_bytes!("i.ogg")),
    ('j', include_bytes!("j.ogg")),
    ('k', include_bytes!("k.ogg")),
    ('l', include_bytes!("l.ogg")),
    ('m', include_bytes!("m.ogg")),
    ('n', include_bytes!("n.ogg")),
    ('o', include_bytes!("o.ogg")),
    ('p', include_bytes!("p.ogg")),
    ('q', include_bytes!("q.ogg")),
    ('r', include_bytes!("r.ogg")),
    ('s', include_bytes!("s.ogg")),
    ('t', include_bytes!("t.ogg")),
    ('u', include_bytes!("u.ogg")),
    ('v', include_bytes!("v.ogg")),
    ('w', include_bytes!("w.ogg")),
    ('x', include_bytes!("x.ogg")),
    ('y', include_bytes!("y.ogg")),
    ('z', include_bytes!("z.ogg")),
    ('0', include_bytes!("0.ogg")),
    ('1', include_bytes!("1.ogg")),
    ('2', include_bytes!("2.ogg")),
    ('3', include_bytes!("3.ogg")),
    ('4', include_bytes!("4.ogg")),
    ('5', include_bytes!("5.ogg")),
    ('6', include_bytes!("6.ogg")),
    ('7', include_bytes!("7.ogg")),
    ('8', include_bytes!("8.ogg")),
    ('9', include_bytes!("9.ogg"))
];

pub fn alphanumeric_sounds(config: &AudioConfig) -> HashMap<char, Chunk> {
    let assets = ASSETS.map(|(ch, bytes)| (ch, config.load_chunk(bytes).unwrap()));
    HashMap::from(assets)
}
