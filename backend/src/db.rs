use crate::image::{ImageView, BoxedStorableImage, SyncResponse};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::oneshot;
use crate::model::model::{DataValue, Model, Record, DataError, Vector2D};
use crate::model::async_model_reader::{do_load_async, load_model_into};
use crate::model::datatypes::DataTypes;
use std::fmt::{Debug, Formatter};
use std::sync::{Mutex, Arc};
use log::*;


#[derive(Debug)]
pub struct DataRecord {
    pub id: Vector2D,
    pub column: String,
    pub fields: Vec<DataFieldValue>,
}

#[derive(Debug)]
pub struct DataFieldValue {
    pub value: DataValue,
    pub reference: Option<Vector2D>,
}


#[derive(Debug, Clone)]
pub struct DBQuery {
    offset: Option<u32>,
    limit: Option<u32>,
    column: Option<String>,
    ids: Option<Vec<Vector2D>>
}

impl DBQuery {
    pub fn new() -> Self {
        Self { offset: None, limit: None, column: None, ids: None }
    }

    pub fn offset(&mut self, offset: u32) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    pub fn limit(&mut self, limit: u32) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    pub fn column(&mut self, column: String) -> &mut Self {
        self.column = Some(column);
        self
    }

    pub fn ids(&mut self, ids: Vec<Vector2D>) -> &mut Self {
        self.ids = Some(ids);
        self
    }


    pub fn build(&self) -> Self {
        Self {
            offset: self.offset,
            limit: self.limit,
            column: self.column.clone(),
            ids: self.ids.clone()
        }
    }
}

pub enum DBMessage {
    GetModel { tx: oneshot::Sender<DBResult<Model>> },
    GetRecords { query: DBQuery, tx: oneshot::Sender<DBResult<Vec<DataRecord>>> },
    SetField { x: u32, y: u32, fi: u32, value: DataValue, tx: oneshot::Sender<DBResult<()>> },
    CloneRecord { x: u32, y: u32, tx: oneshot::Sender<DBResult<DataRecord>> },
    Sync,

    // like "private" ?
    SetModel { model: Model, image: BoxedStorableImage },

    Shutdown
}

impl Debug for DBMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DBMessage::GetModel { .. } => f.debug_struct("DBMessage::GetModel").finish(),
            DBMessage::GetRecords { query, .. } => f.debug_struct("DBMessage::GetRecords").field("query", query).finish(),
            DBMessage::CloneRecord { x, y, .. } => f.debug_struct("DBMessage::CloneRecord").field("x", x).field("y", y).finish(),
            DBMessage::SetField { x, y, fi, value, .. } => f.debug_struct("DBMessage::SetField").field("x", x).field("y", y).field("field_index", fi).field("value", value).finish(),
            DBMessage::Sync => f.debug_struct("DBMessage::Sync").finish(),
            DBMessage::SetModel { .. } => f.debug_struct("DBMessage::SetModel").finish(),
            DBMessage::Shutdown => f.debug_struct("DBMessage::Shutdown").finish(),
        }
    }
}

#[derive(Debug)]
pub enum DBResult<T> {
    Ok(T),
    StillLoading(f32),
    Err(String),
}

impl<T> DBResult<T> where T: Debug {
    pub fn unwrap(self) -> T {
        match self {
            DBResult::Ok(value) => value,
            _ => panic!("Unexpected {:?} instead of value", self)
        }
    }
}

impl<T, E: Into<String>> From<Result<T, E>> for DBResult<T> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(value) => DBResult::Ok(value),
            Err(error) => DBResult::Err(error.into())
        }
    }
}


struct DB {
    path: String,
    image: Option<BoxedStorableImage>,
    model: Option<Model>,
    data_types: DataTypes,

    model_loading_progress: Arc<Mutex<f32>>,
}

impl DB {
    fn new<S>(path: S) -> Self where S: Into<String> {
        Self {
            path: path.into(),
            image: None,
            model: None,
            data_types: DataTypes::new(),
            model_loading_progress: Arc::new(Mutex::new(0.0)),
        }
    }

    async fn handle(&mut self, message: DBMessage) -> () {
        //let message_str = format!("{:?}", message);
        //println!("DB[{}]: start processing {}", self.path, message_str);
        match message {
            DBMessage::Shutdown => {},
            DBMessage::SetModel { model, image } => {
                self.model = Some(model);
                self.image = Some(image);
                *self.model_loading_progress.lock().unwrap() = 1.0;
            }
            DBMessage::CloneRecord { x, y, tx } => {
                let data_types = &self.data_types;
                match &mut self.model {
                    Some(model) => {
                        let mut image = self.image.as_mut().unwrap();
                        let result: Result<DataRecord, DataError> = model.get_by_id(x, y)
                            .map_or(Result::Err(DataError::NotFound), |r| Result::Ok(r.clone()))
                            .and_then(|rec| {

                                let mut x = rec.position.x;

                                let mut y = model.records.iter()
                                    .filter(|r| r.column == rec.column && r.position.x == x)
                                    .map(|r| r.rb_position.y)
                                    .max().unwrap() + 10;

                                while y + rec.rb_position.y - rec.position.y > image.height() {
                                    x = x + rec.rb_position.x - rec.position.x + 10;
                                    y = model.records.iter()
                                        .filter(|r| r.column == rec.column && r.position.x == x)
                                        .map(|r| r.rb_position.y)
                                        .max().unwrap_or(rec.position.y) + 10;
                                }

                                let x_shift = x - rec.position.x;
                                let y_shift = y - rec.position.y;

                                for xx in rec.position.x..=rec.rb_position.x {
                                    for yy in rec.position.y..=rec.rb_position.y {
                                        //todo: not optimal
                                        image.set_pixel(
                                            xx + x_shift,
                                            yy + y_shift,
                                            &image.get_pixel(xx, yy),
                                        )
                                    }
                                }

                                let mut new_record = rec.clone();
                                let shift_vector = Vector2D::new(x_shift, y_shift);
                                new_record.position += shift_vector;
                                new_record.rb_position += shift_vector;
                                for field in &mut new_record.fields {
                                    field.type_start += shift_vector;
                                    field.data_start += shift_vector;
                                    field.data_end += shift_vector;
                                }
                                model.add_record(&new_record);

                                to_data_record(data_types, &new_record, &mut image)
                            });
                        tx.send(result.into()).unwrap();
                    }
                    None => {
                        tx.send(DBResult::StillLoading(*self.model_loading_progress.lock().unwrap())).unwrap();
                    }
                }
            }
            DBMessage::GetRecords { query, tx } => {
                match &self.model {
                    Some(model) => {
                        let mut image = self.image.as_mut().unwrap();
                        let data_types = &self.data_types;
                        let records_to_return: Vec<&Record> = match query.ids  {
                            Some(ids) => ids.iter()
                                .map(|id| model.get_by_id(id.x, id.y))
                                .filter_map(|o| o)
                                .collect(),
                            None => model.records
                                .iter()
                                .filter(|r| match &query.column {
                                    Some(c) => r.column.eq(c),
                                    None => true
                                })
                                .skip(query.offset.unwrap_or(0) as usize)
                                .take(query.limit.unwrap_or(model.records.len() as u32) as usize)
                                .collect()

                        };

                        let records_to_return: Result<Vec<DataRecord>, DataError> = records_to_return.iter()
                            .map(|r| to_data_record(data_types, r, &mut image))
                            .collect();
                        match records_to_return {
                            Ok(records) => tx.send(DBResult::Ok(records)).unwrap(),
                            Err(error) => tx.send(DBResult::Err(error.into())).unwrap()
                        }
                    }
                    None => {
                        tx.send(DBResult::StillLoading(*self.model_loading_progress.lock().unwrap())).unwrap();
                    }
                }
            }
            DBMessage::GetModel { tx } => {
                match &self.model {
                    Some(model) => {
                        tx.send(DBResult::Ok(model.clone())).unwrap();
                    }
                    None => {
                        tx.send(DBResult::StillLoading(*self.model_loading_progress.lock().unwrap())).unwrap();
                    }
                }
            }
            DBMessage::SetField { x, y, fi, value, tx } => {
                let image = self.image.as_mut().unwrap();
                let model = self.model.as_ref();
                match model
                    .and_then(|model| model.get_by_id(x, y))
                    .and_then(|rec| rec.fields.get(fi as usize)) {
                    Some(field) => {
                        let mut view = ImageView::new(image, field.data_start, field.data_end);
                        tx.send(self.data_types.write(&mut view, &field, value).into()).unwrap();
                    }
                    None => {
                        tx.send(DBResult::Err("Not found".to_string())).unwrap()
                    }
                }
            }
            DBMessage::Sync => {
                match self.image.as_mut() {
                    None => {}
                    Some(image) => {
                        if let SyncResponse::Reloaded = image.sync().unwrap() {
                            info!("[{}] Reload model", self.path);
                            let mut model = Model::new();
                            load_model_into(&mut model, ImageView::from(image), |_f| { } );
                            self.model = Some(model);
                            info!("[{}] Reloaded.", self.path);

                        }
                    }
                }
                //info!("Sync completed: {:?}", result)
            }
        };
        //println!("DB[{}]: processed {}", self.path, message_str);
    }
}

fn to_data_record(data_types: &DataTypes, rec: &Record, image: &mut BoxedStorableImage) -> Result<DataRecord, DataError> {
    let mut fields = vec![];

    for field in &rec.fields {
        let view = ImageView::new(image, field.data_start, field.data_end);
        fields.push(DataFieldValue {
            value: data_types.read(&view, field)?,
            reference: field.ref_to_record,
        })
    }

    Ok(DataRecord {
        id: rec.position,
        column: rec.column.clone(),
        fields,
    })
}


#[derive(Clone)]
pub struct DBHandle {
    tx: UnboundedSender<DBMessage>,
}

impl DBHandle {
    pub fn run_in_background(path: &str) -> DBHandle {
        let mut db = DB::new(path);
        let (tx, mut rx) = unbounded_channel();
        let path = path.to_string();
        let async_tx = tx.clone();
        tokio::spawn(async move {
            do_load_async(path.as_str(), async_tx, db.model_loading_progress.clone());
            while let Some(message) = rx.recv().await {
                if let DBMessage::Shutdown = message {
                    break;
                }
                db.handle(message).await
            }
        });
        DBHandle { tx }
    }

    pub async fn get_records(&self, query: DBQuery) -> DBResult<Vec<DataRecord>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(DBMessage::GetRecords { query, tx }).unwrap();
        rx.await.unwrap()
    }

    pub async fn get_model(&self) -> DBResult<Model> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(DBMessage::GetModel { tx }).unwrap();
        rx.await.unwrap()
    }

    pub async fn set_field(&self, x: u32, y: u32, fi: u32, value: DataValue) -> DBResult<()> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(DBMessage::SetField { x, y, fi, value, tx}).unwrap();
        rx.await.unwrap()
    }

    pub async fn clone_record(&self, x: u32, y: u32) -> DBResult<DataRecord> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(DBMessage::CloneRecord { x, y, tx }).unwrap();
        rx.await.unwrap()
    }

    pub async fn sync(&self) {
        self.tx.send(DBMessage::Sync).unwrap();
    }

    pub fn shutdown(&self) {
        self.tx.send(DBMessage::Shutdown).unwrap();
    }

}

