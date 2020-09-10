//#![feature(trace_macros)]
//trace_macros!(true);

use cstr::cstr;
use qmetaobject::*;
use std::{cell::RefCell, fs};

use common::model;
use common::TreeModel;
//mod model;

#[derive(Default, QObject)]
struct WtfType {
    base: qt_base_class!(trait QObject),
    value: qt_property!(i32),
}

#[derive(QObject, Default)]
struct AppState {
    model: model::Model,
    engine: Option<QmlEngine>,
    base: qt_base_class!(trait QObject),
    yeet: qt_method!(
        fn yeet(&mut self) -> QJSValue {
            self.engine.as_mut().map_or(false.into(), |e| {
                e.new_qobject::<WtfType>(WtfType {
                    value: 42,
                    ..Default::default()
                })
            })
        }
    ),
    open_directory: qt_method!(fn(&self, path: QString) -> QJSValue),
    //nth_field_model: qt_method!(fn(&self, set: usize, n: usize) -> QJSValue),
}

impl AppState {
    fn js_expose<T: QObject>(&mut self, obj: T) -> QJSValue {
        self.engine
            .as_mut()
            .map_or(false.into(), |e| e.new_qobject(obj))
    }

    fn open_directory(&mut self, path: QString) -> QJSValue {
        let mut model = model::Model::new();
        let err: Result<(), anyhow::Error> = fs::read_dir::<String>(path.clone().into())
            .unwrap()
            .map(|x| x.unwrap())
            .filter(|x| x.file_type().unwrap().is_file())
            .filter(|x| !x.file_name().to_str().unwrap().contains("ignore"))
            .map(|x| model.add_backing(&x.path()))
            .collect();
        err.unwrap(); // TODO(aptny): Qt Error/Result type bridging
        self.js_expose(PacketListModel {
            list: model,
            base: Default::default(),
            __pather_data: RefCell::new(vec![0]),
        })
    }

    /*fn nth_field_model(&mut self, set: usize, n: usize) -> QJSValue {
        println!("Got {} {} from JS", set, n);
        self.js_expose(FieldListModel {
            packet: self.model.sets[set].packets[n].clone(),
            base: Default::default(),
            __pather_data: RefCell::new(vec![0]),
        })
    }*/
}

#[derive(QObject, TreeModel)]
struct PacketListModel {
    #[root]
    list: model::Model,

    base: qt_base_class!(trait QAbstractItemModel),
    //on_clicked: qt_method!(fn(&self, idx: QModelIndex)),
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

qrc!(init_rscs, "app/qml" {
   "root.qml", "prelude.js"
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

    engine.add_import_path(
        format!("{}/qml", std::env::current_dir().unwrap().to_string_lossy()).into(),
    );

    engine.load_file("qrc:/app/qml/root.qml".into());
    println!("{}/qml", std::env::current_dir().unwrap().to_string_lossy());
    app.borrow_mut().engine = Some(engine);

    app.borrow().engine.as_ref().unwrap().exec(); // TODO(aptny): why does the borrow here work?
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
