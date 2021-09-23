use crate::model::model::{Model, IndexedDataImage, Vector, Field, FieldType, Record};
use std::collections::{HashMap};
use crate::model::colors::Color;
use crate::model::datatypes::reference::REFERENCE_TYPE;

pub fn load_model_into(model: &mut Model, image: &dyn IndexedDataImage) {
    let mut meta_info_map = vec![vec![0; image.height() as usize]; image.width() as usize];
    //0 means empty
    //>0 means block
    //<0 means edge
    let mut next_block_id = 1;
    let mut connection_map: HashMap<i32, i32> = HashMap::new();
    let mut fields: Vec<Record> = vec![];

    let is_blank = |x, y| {
        (x < 0 || y < 0) || image[(x, y)].is_blank()
    };

    let is_meta = |x, y| {
        (x >= 0 && y >= 0) && image[(x, y)].is_meta()
    };

    let if_not_blank = |x, y, value| {
        if is_blank(x, y) { 0 } else { value }
    };

    // find blocks

    for y in 0..image.height() {
        for x in 0..image.width() {
            if meta_info_map[x as usize][y as usize] != 0 { continue; }

            if is_blank(x, y) && is_blank(x + 1, y) && is_blank(x + 2, y) &&
                is_blank(x, y + 1) && is_meta(x + 1, y + 1) && is_meta(x + 2, y + 1) &&
                is_blank(x, y + 2) && is_meta(x + 1, y + 2)  {
                let top_left = Vector { x: x + 1, y: y + 1 };
                //println!("check {} {}", x+1, y+1);

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

                let type_start_point = Vector {
                    x: right_bottom.x - 2,
                    y: top_left.y - 3,
                };
                let ftype =
                    if_not_blank(type_start_point.x + 0, type_start_point.y + 0, 0b100_000_000)
                        | if_not_blank(type_start_point.x + 1, type_start_point.y + 0, 0b010_000_000)
                        | if_not_blank(type_start_point.x + 2, type_start_point.y + 0, 0b001_000_000)
                        | if_not_blank(type_start_point.x + 0, type_start_point.y + 1, 0b000_100_000)
                        | if_not_blank(type_start_point.x + 1, type_start_point.y + 1, 0b000_010_000)
                        | if_not_blank(type_start_point.x + 2, type_start_point.y + 1, 0b000_001_000)
                        | if_not_blank(type_start_point.x + 0, type_start_point.y + 2, 0b000_000_100)
                        | if_not_blank(type_start_point.x + 1, type_start_point.y + 2, 0b000_000_010)
                        | if_not_blank(type_start_point.x + 2, type_start_point.y + 2, 0b000_000_001);


                for x in top_left.x..=right_bottom.x {
                    for y in top_left.y..=right_bottom.y {
                        meta_info_map[x as usize][y as usize] = next_block_id
                    }
                }

                let column_pix = image.get_pixel(top_left.x, 0);
                fields.push(Record {
                    position: top_left,
                    //todo: create util fn
                    column: format!("#{:02X?}{:02X?}{:02X?}", column_pix[0], column_pix[1], column_pix[2]),
                    fields: vec![Field {
                        field_type: FieldType(ftype),
                        data_start: data_top_left,
                        data_end: data_right_bottom,
                        type_start: type_start_point,
                        ref_to_record: None,
                    }],
                });

                next_block_id += 1
            }
        }
    }

    // find connections
    let mut next_edge_id = 1;
    for y in 0..image.height() {
        for x in 0..image.width() {
            if meta_info_map[x as usize][y as usize] != 0 { continue; }
            if !is_meta(x, y) { continue; }

            //ok, here we go. start flood fill
            let mut points_to_investigate: Vec<Vector> = vec![];

            let mut connect_from_id = -1;
            let mut connect_to_id = -1;
            points_to_investigate.push(Vector { x, y });

            while !points_to_investigate.is_empty() {
                let p = points_to_investigate.pop().unwrap();
                if !is_meta(p.x, p.y) { continue; }
                let meta: &mut i32 = &mut meta_info_map[p.x as usize][p.y as usize];
                if *meta < 0 { continue; }
                if *meta > 0 {
                    if connect_from_id == -1 {
                        connect_from_id = *meta
                    } else if connect_from_id > *meta {
                        connect_to_id = connect_from_id;
                        connect_from_id = *meta
                    } else if connect_from_id != *meta {
                        connect_to_id = *meta
                    } else if connect_from_id != -1 && connect_to_id != -1 && *meta != connect_to_id && *meta != connect_from_id {
                        println!("WARNING: we have {} -> {}, but also found {}", connect_from_id, connect_to_id, *meta);
                    }
                } else {
                    *meta = -next_edge_id;
                    //println!("{} {} = {}", p.x, p.y, -next_edge_id);

                    points_to_investigate.push(Vector { x: p.x + 1, y: p.y });
                    points_to_investigate.push(Vector { x: p.x - 1, y: p.y });
                    points_to_investigate.push(Vector { x: p.x, y: p.y + 1 });
                    points_to_investigate.push(Vector { x: p.x, y: p.y - 1 });
                }
            }
            if connect_to_id > 0 && connect_from_id > 0 {
                assert!(connect_to_id > connect_from_id);
                connection_map.insert(connect_to_id - 1, connect_from_id - 1); //convert to indexes
            }
            next_edge_id += 1;
        }
    }

    // process references
    for idx in 0..fields.len() {
        let rec = &fields[idx];
        let field = &rec.fields[0];
        if field.field_type != REFERENCE_TYPE { continue }
        //println!("  reference from {:?} ", rec.position);
        let start_point = Vector { x: field.type_start.x + 1, y: field.type_start.y + 1 };
        let color = image.get_pixel(start_point.x, start_point.y);
        let mut points_to_process = vec![start_point];

        let from_field = rec.position;

        while !points_to_process.is_empty() {
            let p = points_to_process.pop().unwrap();
            let meta = meta_info_map[p.x as usize][p.y as usize];
            if meta > 0  {
                let found_rec = fields[(meta - 1) as usize].clone();
                if found_rec.position != from_field {
                    let field = &mut fields[idx].fields[0];
                    let found_field = found_rec.fields[0].clone();
                    //println!("found reference from {:?} to {:?}", from_field, found_field.data_start);
                    field.field_type = found_field.field_type;
                    field.data_start = found_field.data_start;
                    field.data_end = found_field.data_end;
                    field.ref_to_record = Some(found_rec.position);
                    break;
                }
            } else if meta == 0 && image.get_pixel(p.x, p.y) == color {
                meta_info_map[p.x as usize][p.y as usize] = -999;
                for dx in -4..4 as i32 {
                    for dy in -4..4 as i32 {
                        points_to_process.push(Vector { x: (p.x as i32 + dx) as u32, y: (p.y as i32 + dy) as u32 });
                    }
                }
            }
        }
    }


    // and now all together

    for idx in (0..fields.len() as i32).rev() {
        let mut record = fields.remove(idx as usize);
        //println!(" idx = {} fields = {} conn_from = {:?}", idx, record.fields.len(), connection_map.get(&idx) );
        match connection_map.get(&idx) {
            None => model.records.insert(0, record),
            Some(from_idx) => fields[*from_idx as usize].fields.append(&mut record.fields)
        }
    }
}