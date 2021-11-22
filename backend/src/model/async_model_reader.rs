use crate::model::model::{Model, Vector2D, Field, FieldType, Record};
use std::collections::{HashMap, HashSet};
use crate::model::datatypes::reference::REFERENCE_TYPE;
use crate::image::ImageView;
use std::collections::hash_map::Entry;
use crate::model::datatypes::DEFAULT_TYPE;
use tokio::sync::mpsc::{UnboundedSender};
use crate::io::image_io::load_image;
use crate::db::DBMessage;
use std::sync::{Arc, Mutex};
use log::info;
use std::time::SystemTime;

#[derive(Clone, Copy, Debug)]
struct Block {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
    block_id: usize,
}

impl Block {
    fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x1 && y >= self.y1 && x <= self.x2 && y <= self.y2
    }
}

// 1024x1024 pixels blocks
struct BlocksMap {
    map: HashMap<(u32, u32), Vec<Block>>,
}

impl BlocksMap {
    fn new() -> Self {
        Self { map: HashMap::new() }
    }

    fn add(&mut self, block: Block) {
        let x1 = block.x1 / 1024;
        let x2 = block.x2 / 1024;
        let y1 = block.y1 / 1024;
        let y2 = block.y2 / 1024;
        for x in x1..=x2 {
            for y in y1..=y2 {
                match self.map.entry((x, y)) {
                    Entry::Occupied(mut vector) => {
                        vector.get_mut().push(block.clone());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![block.clone()]);
                    }
                };
            }
        }
    }

    fn get_block(&self, x: u32, y: u32) -> Option<&Block> {
        match self.map.get(&(x / 1024, y / 1024)) {
            Some(vec) => vec.iter().find(|b| b.contains(x, y)),
            None => None
        }
    }

    fn get_blocks(&self) -> Vec<&Block> {
        let mut v: Vec<&Block> = self.map.values()
            .flatten()
            .collect();

        v.sort_by(|b1, b2| b1.block_id.cmp(&b2.block_id));
        v
    }
}


pub fn do_load_async(path: &str, tx: UnboundedSender<DBMessage>, progress: Arc<Mutex<f32>>) {
    let path = path.to_string();
    tokio::spawn(async move {
        let mut image = load_image(path.as_str());
        let mut model = Model::new();
        let image_view = ImageView::from(&mut image);
        load_model_into(&mut model, image_view, |p| {
            let mut float = progress.lock().unwrap();
            *float = p;
        });
        tx.send(DBMessage::SetModel { model, image }).unwrap();
    });
}

pub fn load_model_into(model: &mut Model, image: ImageView<'_>, on_progress: impl Fn(f32)) {

    let start_time = SystemTime::now();
    let mut next_block_id = 0;
    let mut connection_map: HashMap<usize, usize> = HashMap::new();
    let mut fields: Vec<Record> = vec![];
    let mut blocks_map = BlocksMap::new();

    let is_blank = |x, y| {
        (x >= image.width || y >= image.height) || image.get_pixel(x, y).is_blank()
    };

    let is_meta = |x, y| {
        (x < image.width && y < image.height) && image.get_pixel(x, y).is_meta()
    };

    let if_not_blank = |x, y, value| {
        if is_blank(x, y) { 0 } else { value }
    };

    let default_type =
        if_not_blank(0, 0, 0b100_000_000)
            | if_not_blank(1, 0, 0b010_000_000)
            | if_not_blank(2, 0, 0b001_000_000)
            | if_not_blank(0, 1, 0b000_100_000)
            | if_not_blank(1, 1, 0b000_010_000)
            | if_not_blank(2, 1, 0b000_001_000)
            | if_not_blank(0, 2, 0b000_000_100)
            | if_not_blank(1, 2, 0b000_000_010)
            | if_not_blank(2, 2, 0b000_000_001);

    // find blocks
    for y in 1..image.height {
        for x in 1..image.width {
            if is_meta(x, y) && is_blank(x, y - 1) && is_blank(x - 1, y - 1) &&
                is_meta(x + 1, y) && is_meta(x, y + 1) && is_blank(x + 1, y - 1) &&
                is_blank(x - 1, y) && is_blank(x - 1, y + 1) &&
                blocks_map.get_block(x, y).is_none()  {
                let top_left = Vector2D { x, y };

                let mut right_bottom = top_left.clone();
                while is_meta(right_bottom.x + 1, top_left.y) {
                    right_bottom.x += 1
                }
                while is_meta(top_left.x, right_bottom.y + 1) {
                    right_bottom.y += 1
                }

                let mut data_top_left = top_left.clone();
                while is_meta(data_top_left.x, data_top_left.y) && data_top_left.x < right_bottom.x && data_top_left.y < right_bottom.y {
                    data_top_left.x += 1;
                    data_top_left.y += 1;
                }
                let mut data_right_bottom = right_bottom.clone();
                while is_meta(data_right_bottom.x, data_right_bottom.y) && data_right_bottom.x > data_top_left.x && data_right_bottom.y > data_top_left.y {
                    data_right_bottom.x -= 1;
                    data_right_bottom.y -= 1;
                }
                assert!(data_right_bottom.x >= data_top_left.x, "failed: {} > {}", data_right_bottom.x, data_top_left.x);
                assert!(data_right_bottom.y >= data_top_left.y);

                let type_start_point = Vector2D {
                    x: right_bottom.x - 2,
                    y: top_left.y - 3,
                };
                let mut ftype =
                    if_not_blank(type_start_point.x + 0, type_start_point.y + 0, 0b100_000_000)
                        | if_not_blank(type_start_point.x + 1, type_start_point.y + 0, 0b010_000_000)
                        | if_not_blank(type_start_point.x + 2, type_start_point.y + 0, 0b001_000_000)
                        | if_not_blank(type_start_point.x + 0, type_start_point.y + 1, 0b000_100_000)
                        | if_not_blank(type_start_point.x + 1, type_start_point.y + 1, 0b000_010_000)
                        | if_not_blank(type_start_point.x + 2, type_start_point.y + 1, 0b000_001_000)
                        | if_not_blank(type_start_point.x + 0, type_start_point.y + 2, 0b000_000_100)
                        | if_not_blank(type_start_point.x + 1, type_start_point.y + 2, 0b000_000_010)
                        | if_not_blank(type_start_point.x + 2, type_start_point.y + 2, 0b000_000_001);

                if ftype == DEFAULT_TYPE.0 {
                    ftype = default_type
                }

                let column_pix = image.get_pixel(top_left.x, 0);
                fields.push(Record {
                    position: top_left,
                    column: column_pix.to_hex_color(),
                    rb_position: right_bottom,
                    fields: vec![Field {
                        field_type: FieldType(ftype),
                        data_start: data_top_left,
                        data_end: data_right_bottom,
                        type_start: type_start_point,
                        ref_to_record: None,
                    }],
                });
                blocks_map.add(Block {
                    block_id: next_block_id,
                    x1: top_left.x,
                    y1: top_left.y,
                    x2: right_bottom.x,
                    y2: right_bottom.y,
                });

                next_block_id += 1;
                //println!("found block {:?}", top_left);
            }
        }
        image.optimize();
        on_progress(0.33 * (y as f32) / (image.height as f32));
    }

    // find connections
    //debug!("find connections");
    let mut points_investigated: HashSet<Vector2D> = HashSet::new();
    let blocks_count = blocks_map.get_blocks().len();
    let mut block_idx = 0;
    for block in blocks_map.get_blocks() {
        //ok, here we go. start flood fill
        let mut points_to_investigate: Vec<Vector2D> = vec![];

        const NOT_CONNECTED: usize = usize::max_value();
        let connect_from_id = block.block_id;
        let mut connect_to_id = NOT_CONNECTED;
        for x in block.x1 - 1..=block.x2 + 1 {
            points_to_investigate.push(Vector2D::new(x, block.y1 - 1));
            points_to_investigate.push(Vector2D::new(x, block.y2 + 1));
        }
        for y in block.y1..=block.y2 {
            points_to_investigate.push(Vector2D::new(block.x1 - 1, y));
            points_to_investigate.push(Vector2D::new(block.x2 + 1, y));
        }
        //println!("Check for {} {} {}", block.block_id, block.x1, block.y1);

        'loop1: while !points_to_investigate.is_empty() {
            let p = points_to_investigate.pop().unwrap();
            if points_investigated.contains(&p) { continue; }
            points_investigated.insert(p);
            if !is_meta(p.x, p.y) { continue; }
            if block.contains(p.x, p.y) { continue; }
            //println!("check {:?}", p);
            match blocks_map.get_block(p.x, p.y) {
                Some(block) => {
                    if  block.block_id < connect_from_id {
                        continue 'loop1;
                    } else if connect_from_id != block.block_id {
                        connect_to_id = block.block_id;
                        break 'loop1;
                    } else if connect_from_id != NOT_CONNECTED && connect_to_id != NOT_CONNECTED && block.block_id != connect_to_id && block.block_id != connect_from_id {
                        info!("WARNING: we have {} -> {}, but also found {}", connect_from_id, connect_to_id, block.block_id);
                    }
                }
                None => {
                    points_to_investigate.push(Vector2D { x: p.x + 1, y: p.y });
                    points_to_investigate.push(Vector2D { x: p.x - 1, y: p.y });
                    points_to_investigate.push(Vector2D { x: p.x, y: p.y + 1 });
                    points_to_investigate.push(Vector2D { x: p.x, y: p.y - 1 });
                }
            }
        }
        if connect_to_id != NOT_CONNECTED && connect_from_id != NOT_CONNECTED {
            assert!(connect_to_id > connect_from_id);
            connection_map.insert(connect_to_id, connect_from_id);
        }
        image.optimize();
        block_idx = block_idx + 1;
        on_progress(0.33 + 0.33 * (block_idx as f32) / (blocks_count as f32));
    }

    //println!("references");
// process references
    for idx in 0..fields.len() {
        let rec = &fields[idx];
        let field = &rec.fields[0];
        if field.field_type != REFERENCE_TYPE { continue; }
        let start_point = Vector2D { x: field.type_start.x + 1, y: field.type_start.y + 1 };
        let color = image.get_pixel(start_point.x, start_point.y);
//        println!("  reference from {:?} color={:?} ts={:?}", rec.position, color, field.type_start);
        let mut points_to_process = vec![start_point];
        let mut points_investigated = HashSet::new();

        let from_field = rec.position;

        while !points_to_process.is_empty() {
            let p = points_to_process.pop().unwrap();
            if points_investigated.contains(&p) { continue; }
            points_investigated.insert(p);
            match blocks_map.get_block(p.x, p.y) {
                Some(block) => {
//                    println!("p {:?} {:?}", p, block);
                    let found_rec = fields[block.block_id].clone();
                    if found_rec.position != from_field {
                        let field = &mut fields[idx].fields[0];
                        let found_field = found_rec.fields[0].clone();
                        //                      println!("found reference from {:?} to {:?}", from_field, found_field.data_start);
                        field.field_type = found_field.field_type;
                        field.data_start = found_field.data_start;
                        field.data_end = found_field.data_end;
                        field.ref_to_record = Some(found_rec.position);
                        break;
                    }
                }
                None => {
                    if image.get_pixel(p.x, p.y) == color {
                        for dx in -4..4 as i32 {
                            for dy in -4..4 as i32 {
                                points_to_process.push(Vector2D { x: (p.x as i32 + dx) as u32, y: (p.y as i32 + dy) as u32 });
                            }
                        }
                    }
                }
            }
        }
        image.optimize();
        on_progress(0.67 + 0.16 * (idx as f32) / (fields.len() as f32));
    }


// and now all together

    //println!("together");
    let fields_count = fields.len();
    for idx in (0..fields_count).rev() {
        let mut record = fields.remove(idx as usize);
//println!(" idx = {} fields = {} conn_from = {:?}", idx, record.fields.len(), connection_map.get(&idx) );
        match connection_map.get(&idx) {
            None => model.insert_record(0, &record),
            Some(from_idx) => {
                let mut target_record = &mut fields[*from_idx];
                target_record.rb_position.x = target_record.rb_position.x.max(record.rb_position.x);
                target_record.rb_position.y = target_record.rb_position.y.max(record.rb_position.y);
                target_record.fields.append(&mut record.fields)
            }
        }
        on_progress(0.84 + 0.16 * ((fields_count - idx) as f32) / (fields_count as f32));
    }

    model.loading_time = start_time.elapsed().unwrap();
}