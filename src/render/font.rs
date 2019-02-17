use nalgebra::Matrix;
use nalgebra::Dim;
use nalgebra::Scalar;
use gl;
use std::collections::HashMap;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::hinting::HintingOptions;
use crate::resources::Resources;
use euclid::Point2D;
use slog::trace;
use slog::debug;
use linked_list::LinkedList;
use linked_list::Cursor;
use std::iter::FromIterator;

pub struct Font {
    font: Vec<font_kit::font::Font>,
    gl: gl::Gl,
    atlas: Atlas,
}

impl Font {
    pub fn from_resource(gl: &gl::Gl, resources: &Resources, path: &str, font_size: f32, log: &slog::Logger) -> Font {
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890:;'\",<.>/?!@#$%^&*()_-+=[]{}\\|`~";
        let fonts = resources.load_font(path).unwrap();
        let hinting_options = HintingOptions::Full(font_size);
        let canvas_format = Format::Rgb24;
        let rasterization_options = RasterizationOptions::SubpixelAa;
        let mut atlas = Atlas::new(512, 512, 1);
        for font in fonts {
            trace!(log, "Font loaded: {:?}", font);
            for character in chars.chars() {
                let glyph_id = font.glyph_for_char(character).unwrap();
                let raster_rect = font.raster_bounds(glyph_id,
                                                    font_size,
                                                    &Point2D::zero(),
                                                    hinting_options,
                                                    rasterization_options)
                                    .unwrap();

                let mut canvas = Canvas::new(&raster_rect.size.to_u32(), canvas_format);

                let origin = Point2D::new(-raster_rect.origin.x, -raster_rect.origin.y).to_f32();
                font.rasterize_glyph(&mut canvas,
                                    glyph_id,
                                    font_size,
                                    &origin,
                                    hinting_options,
                                    rasterization_options);

                atlas.add_glyph(character, &canvas, log);
            }
        }
        Font {
            font: resources.load_font(path).unwrap(),
            gl: gl.clone(),
            atlas: atlas,
        }
    }
}

type AtlasRegion = nalgebra::Vector4<u32>;
type SkylineNode = nalgebra::Vector3<u32>;

struct Atlas {
    nodes: LinkedList<SkylineNode>,
    pixel_width: u32,
    pixel_height: u32,
    pixel_depth: u32,
    data_length: usize,
    data_used: usize,
    data: Vec<u8>,
    glyph_map: HashMap<char, AtlasRegion>,
}

impl Atlas {
    pub fn new(width: i32, height: i32, depth: i32) -> Atlas {
        let size = (width * height * depth) as usize;
        Atlas {
            nodes: LinkedList::from_iter(vec!(SkylineNode::new(1, 1, width as u32 - 2)).into_iter()),
            pixel_width: width as u32,
            pixel_height: height as u32,
            pixel_depth: depth as u32,
            data_length: size,
            data_used: 0,
            data: vec![0; size * 3],
            glyph_map: HashMap::new(),
        }
    }

    pub fn add_glyph(&mut self, character: char, canvas: &Canvas, log: &slog::Logger) -> Result<(), failure::Error> {
        let width = canvas.size.width;
        let height = canvas.size.height;
        let required_width = width + 1;
        let required_height = height + 1;
        let mut best_height = std::u32::MAX;
        let mut best_width = std::u32::MAX;

        debug!(
            log,
            "Adding glyph for character '{character}' to texture atlas",
            character = character
        );

        trace!(
            log,
            "'{character}' requires {w}x{h} space",
            character = character,
            w = required_width,
            h = required_height
        );

        // The SKYLINE Bin-pack algorithm
        let mut cit = self.nodes.cursor();
        let mut best_node: Option<SkylineNode> = None;
        let mut region = AtlasRegion::new(0, 0, required_width, required_height);
        // Current node is our candidate-test node, the one we want to stack our new Glyph on top of.
        'node_loop: loop {
            let current_node = match cit.next() {
                Some(t) => t.clone(),
                None => break,
            };

            trace!(
                log,
                "current node {:?}",
                current_node,
            );

            let mut remaining_nodes = cit.split();
            // Imagine the candidate node is 10px wide, the new one is 15px wide, and there is only 2px of space
            // to the right of our candidate node. If we place our new node on top of it, the right side will
            // extend past our pixel width boundary.
            if required_width + current_node[0] as u32 > self.pixel_width {
                trace!(
                    log,
                    "not enough width after node: {:?}, skipping it",
                    current_node,
                );
                continue;
            }
            let mut it = remaining_nodes.iter();
            let mut max_y_found = 0;
            let mut width_left = required_width;
            let mut next_node = current_node;
            // Now we need to check for possible nodes to the right of our new node that we might
            // be intersecting and also that there is enough height to fit our node.
            // Imagine there are two nodes side by side that are both 10px wide, the first one is 10px tall and the second one is 20px tall
            // If we try to place our new node on the first node then it will overhang it by 5px(remember it is 15px wide).required_width
            // That overhang will actually intersect with the second node so we can't use this candidate node
            while width_left > 0 {
                max_y_found = std::cmp::max(max_y_found, next_node[1]);
                if max_y_found + required_height > self.pixel_height {
                    trace!(
                        log,
                        "not enough height above node: {:?}, skipping it",
                        next_node,
                    );
                    break;
                }
                let overhang_width: i32 = width_left as i32 - next_node[2] as i32;
                let max_x = next_node[0] + next_node[2];
                width_left = if overhang_width < 0 {
                    0
                } else {
                    overhang_width as u32
                };
                // I don't think this check is actually needed
                // We aren't changing the x position of our new node here
                // and we know it already fits because of the first check in the 'node_loop'
                if max_x as i32 + overhang_width + 1 > self.pixel_width as i32 {
                    break;
                }
                match it.next() {
                    Some(node) => { next_node = node.clone()}
                    _ => { break; }
                }
            }
            if width_left > 0 {
                cit.splice(&mut remaining_nodes);
                continue;
            }

            // Let's check to see if this current node is the 'best' one, when considering only the ones
            // we have checked in the past.
            let start_height = max_y_found;
            let max_height = start_height + required_height;
            if max_height < best_height || (max_height == best_height && current_node[2] < best_width) {
                best_node = Some(current_node.clone());
                best_height = max_height;
                best_width = current_node[2];
                region.x = current_node[0];
                region.y = start_height;
            }

            cit.splice(&mut remaining_nodes);
        }

        match best_node {
            None => {
                panic!("No possible candidate found to stack on top of");
            }
            Some(_) => {
                let new_node = SkylineNode::new(region[0], region[1] + region[3], region[2]);
                cit.insert(new_node);
                self.add_bitmap_to_texture_data(&canvas, &region, width, height);
            }
        }

        Ok(())
    }
    fn add_bitmap_to_texture_data(&mut self,
                                  data: &Canvas,
                                  region: &AtlasRegion,
                                  width: u32,
                                  height: u32) {
        for i in 0..height {
            let input_row_offset = data.stride as u32 * i;
            let output_row = region[1] + i * self.pixel_width;
            let output_col = region[0];
            for ri in 0..width {
                let output_idx = (output_row + output_col) as usize;
                let input_idx = (input_row_offset + ri) as usize;
                self.data[output_idx] = data.pixels[input_idx];
            }
        }
    }
}