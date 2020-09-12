//#![feature(trace_macros)]
//trace_macros!(true);
#![windows_subsystem = "windows"]

//use cstr::cstr;
use qmetaobject::*;
use std::{cell::RefCell, fs};

use common::model;
use common::{JSExposeExt, TreeModel};
//mod model;

#[derive(Default, QObject)]
struct WtfType {
    base: qt_base_class!(trait QObject),
    value: qt_property!(i32),
}

#[derive(QObject, Default)]
struct AppState {
    _model: model::Model,
    engine: Option<QmlEngine>,
    base: qt_base_class!(trait QObject),
    open_directory: qt_method!(fn(&self, path: QString) -> QJSValue),
}

impl AppState {
    fn open_directory(&mut self, path: QString) -> QJSValue {
        let mut model = model::Model::new();
        let res: Result<(), anyhow::Error> = fs::read_dir::<String>(path.clone().into())
            .map_err(|e| e.into())
            .and_then(|dir| {
                dir.map(|x| x.unwrap())
                    .filter(|x| x.file_type().unwrap().is_file())
                    .filter(|x| !x.file_name().to_str().unwrap().contains("ignore"))
                    .map(|x| model.add_backing(&x.path()))
                    .collect()
            });
        let res = res.map(|_| PacketListModel {
            list: model,
            base: Default::default(),
            __pather_data: RefCell::new(vec![0]),
        });
        self.engine.js_map_or_throw(res)
    }
}

#[derive(QObject, TreeModel)]
struct PacketListModel {
    #[root]
    list: model::Model,

    base: qt_base_class!(trait QAbstractItemModel),
    __pather_data: RefCell<Vec<i32>>,
}

impl Clone for PacketListModel {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Default for PacketListModel {
    fn default() -> Self {
        todo!()
    }
}

qrc!(init_rscs, "app" {
    "qml/root.qml", "qml/prelude.js"
});

fn main() {
    let app = RefCell::new(AppState {
        engine: None,
        ..Default::default()
    });
    let mut engine = QmlEngine::new();

    unsafe {
        engine.set_object_property("app".into(), QObjectPinned::new(&app));
    }

    common::qthax::register_types();
    init_rscs();

    engine.load_file("qrc:/app/qml/root.qml".into());
    app.borrow_mut().engine = Some(engine);

    app.borrow().engine.as_ref().unwrap().exec(); // TODO(aptny): why does the borrow here work?
}
