#[cfg(test)]

#[allow(unused_imports)]
use super::generator::chunk::{ChunkCoord, ChunkLocalCoord, WorldCoord, CHUNK_SIZE};

#[test]
fn positive_negative_test() {
    assert_eq!(1 / 16, 0);
    assert_eq!(-1 / 16, 0);
    assert_eq!(-17 / 16, -1);
    assert_eq!(17 / 16, 1);
    assert_eq!(16 / 16, 1);
    assert_eq!(-16 / 16, -1);
    assert_eq!(-16 % 16, 0);
    assert_eq!(-15 % 16, -15);
}

#[test]
fn coord_test() {
    let test_batches = [
        (
            WorldCoord {
                x: CHUNK_SIZE as i32,
                y: CHUNK_SIZE as i32,
                z: CHUNK_SIZE as i32,
            },
            ChunkCoord { x: 1, y: 1, z: 1},
            ChunkLocalCoord { x: 0, y: 0, z: 0},
        ),
        (
            WorldCoord {
                x: -1,
                y: CHUNK_SIZE as i32,
                z: CHUNK_SIZE as i32,
            },
            ChunkCoord { x: -1, y: 1, z: 1},
            ChunkLocalCoord { x: CHUNK_SIZE - 1, y: 0, z: 0},
        ),
        (
            WorldCoord {
                x: -(CHUNK_SIZE as i32),
                y: CHUNK_SIZE as i32,
                z: CHUNK_SIZE as i32,
            },
            ChunkCoord { x: -1, y: 1, z: 1},
            ChunkLocalCoord { x: 0, y: 0, z: 0},
        ),
    ];

    for (world, expected_chunk, expected_local) in test_batches {
        let chunk_coord: ChunkCoord = world.into();
        let local_coord: ChunkLocalCoord = world.into();

        assert_eq!(chunk_coord, expected_chunk);
        assert_eq!(local_coord, expected_local);
    }
}
