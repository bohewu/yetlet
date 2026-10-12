//! Fixed flat scene input for YT-023; no component tree or DOM integration.

pub const WIDTH: u32 = 1024;
pub const HEIGHT: u32 = 1024;
pub const INSTANCE_BYTES: u64 = 32;
pub const IMAGE_BYTES: u64 = WIDTH as u64 * HEIGHT as u64 * 4;
pub const WORKLOADS: [u32; 3] = [10_000, 50_000, 100_000];

#[derive(Debug, PartialEq)]
pub struct Node {
    pub position_extent: [f32; 4],
    pub color: [f32; 4],
    pub center_pixel: [u32; 2],
}

/// Deterministic row-major grid with pixel-centered rectangles and RGB cycling.
pub fn scene(count: u32) -> Vec<Node> {
    assert!(WORKLOADS.contains(&count), "only the declared workloads");
    let columns = (count as f64).sqrt().ceil() as u32;
    let rows = count.div_ceil(columns);
    (0..count)
        .map(|i| {
            let x = (((i % columns) as f64 + 0.5) * WIDTH as f64 / columns as f64).floor() as u32;
            let y = (((i / columns) as f64 + 0.5) * HEIGHT as f64 / rows as f64).floor() as u32;
            let mut color = [0.0, 0.0, 0.0, 1.0];
            color[(i % 3) as usize] = 1.0;
            Node {
                position_extent: [
                    2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0,
                    1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32,
                    0.7 / columns as f32,
                    0.7 / rows as f32,
                ],
                color,
                center_pixel: [x, y],
            }
        })
        .collect()
}

pub fn instance_data(nodes: &[Node]) -> Vec<u8> {
    nodes
        .iter()
        .flat_map(|node| node.position_extent.iter().chain(node.color.iter()))
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}

/// Checks the actual readback center of every instance and an untouched corner.
pub fn verify_pixels(nodes: &[Node], rgba: &[u8]) -> Result<(), String> {
    if rgba.len() != IMAGE_BYTES as usize {
        return Err("unexpected image length".into());
    }
    if rgba[rgba.len() - 4..] != [0, 0, 0, 255] {
        return Err("background corner was not black".into());
    }
    for (i, node) in nodes.iter().enumerate() {
        let offset = ((node.center_pixel[1] * WIDTH + node.center_pixel[0]) * 4) as usize;
        let expected = node.color.map(|value| (value * 255.0) as u8);
        if rgba[offset..offset + 4] != expected {
            return Err(format!("instance {i} center pixel mismatch"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_workloads_have_distinct_in_bounds_instances_and_exact_buffer_sizes() {
        for count in WORKLOADS {
            let nodes = scene(count);
            assert_eq!(nodes.len(), count as usize);
            let centers: HashSet<_> = nodes.iter().map(|node| node.center_pixel).collect();
            assert_eq!(centers.len(), count as usize);
            assert!(centers.iter().all(|[x, y]| *x < WIDTH && *y < HEIGHT));
            assert_eq!(
                instance_data(&nodes).len() as u64,
                count as u64 * INSTANCE_BYTES
            );
            assert!(
                nodes
                    .iter()
                    .all(|node| node.position_extent.iter().all(|v| v.is_finite()))
            );
        }
    }

    #[test]
    fn readback_check_rejects_clear_only_and_wrong_size_images() {
        let nodes = scene(WORKLOADS[0]);
        assert!(verify_pixels(&nodes, &[]).is_err());
        let mut rgba = [0, 0, 0, 255].repeat((WIDTH * HEIGHT) as usize);
        assert!(verify_pixels(&nodes, &rgba).is_err());
        for node in &nodes {
            let offset = ((node.center_pixel[1] * WIDTH + node.center_pixel[0]) * 4) as usize;
            rgba[offset..offset + 4].copy_from_slice(&node.color.map(|v| (v * 255.0) as u8));
        }
        assert_eq!(verify_pixels(&nodes, &rgba), Ok(()));
        let offset =
            ((nodes[9999].center_pixel[1] * WIDTH + nodes[9999].center_pixel[0]) * 4) as usize;
        rgba[offset] ^= 255;
        assert!(verify_pixels(&nodes, &rgba).is_err());
    }
}
