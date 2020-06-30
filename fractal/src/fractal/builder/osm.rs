use super::{TilePos, TileType};
use crate::fractal::queue::QueueHandle;
use crate::fractal::queue::TileResponse;
use crate::fractal::TileContent;

pub fn worker(handle: QueueHandle) {
    loop {
        let h = match handle.tiles.upgrade() {
            Some(h) => h,
            None => break,
        };

        let mut h = h.lock();

        if h.params.kind != TileType::Map {
            drop(h);
            handle.wait();
            continue;
        }

        let next = match h.recv() {
            None => {
                drop(h);
                handle.wait();
                continue;
            },
            Some(next) => next,
        };

        let ver = h.params_version;
        let sz = h.params.resolution;

        // make sure the lock is freed
        drop(h);

        let tile = TileContent::new(build(next, sz as usize));

        let ret = handle.send(TileResponse {
            pos: next,
            version: ver,
            content: tile,
        });

        if ret.is_err() {
            break;
        }
    }
}

fn build(pos: TilePos, texture_size: usize) -> Vec<u8> {
    let mut pixels = vec![0; texture_size * texture_size * 4];
    if pos.z > 19 {
        return pixels;
    }
    if pos.x < 0 || pos.y < 0 {
        return pixels;
    }
    let d = pos.z;
    let s = 1 << d;
    let x = pos.x;
    let y = pos.y;

    if x > s {
        return pixels;
    }
    if y > s {
        return pixels;
    }

    let y = s - y - 1;

    let url = format!("https://tile.openstreetmap.org/{}/{}/{}.png", d, x, y);
    dbg!(&url);
    let url = reqwest::blocking::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:77.0) Gecko/20100101 Firefox/77.0")
        .build()
        .unwrap()
        .get(&url)
        .send();
    let url = match url {
        Ok(x) => x,
        Err(_) => {
            return pixels;
        },
    };
    let url = url.bytes();
    let url = match url {
        Ok(x) => x,
        Err(_) => {
            return pixels;
        },
    };
    let url = url.to_vec();
    let mut url = url.as_slice();

    let decoder = png::Decoder::new(&mut url);
    let (info, mut reader) = match decoder.read_info() {
        Ok(x) => x,
        Err(_) => {
            return pixels;
        },
    };
    dbg!(&info);
    let mut buf = vec![0; texture_size * texture_size * 3];
    assert_eq!(info.width, texture_size as u32);
    assert_eq!(info.height, texture_size as u32);
    assert_eq!(info.buffer_size(), buf.len());
    reader.next_frame(&mut buf).unwrap();
    pixels.clear();
    for p in buf.chunks(3) {
        pixels.push(p[0]);
        pixels.push(p[1]);
        pixels.push(p[2]);
        pixels.push(255);
    }
    pixels
}
