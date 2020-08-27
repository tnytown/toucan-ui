//#![feature(trace_macros)]
//trace_macros!(true);

use cstr::cstr;
use qmetaobject::*;
use std::{cell::RefCell, fs, rc::Rc};
mod model;

#[allow(dead_code)]
enum ItemDataRole {
    DisplayRole,
    DecorationRole,
    EditRole,
    ToolTipRole,
    StatusTipRole,
    WhatsThisRole,

    FontRole,
    TextAlignmentRole,
    BackgroundRole,
    ForegroundRole,
    CheckStateRole,
    AccessibleTextRole,
    AccessibleDescriptionRole,
    SizeTipRole,

    UserRole = 0x0100,
}

#[derive(QObject)]
struct PacketListModel<'a> {
    base: qt_base_class!(trait QAbstractListModel),
    on_clicked: qt_method!(fn(&self, idx: QModelIndex)),
    list: &'a model::Model,
    field: Rc<RefCell<FieldListModel<'a>>>,
}

impl Clone for PacketListModel<'_> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Default for PacketListModel<'_> {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Default, Clone)]
struct WtfType(i32);
impl QMetaType for WtfType {}

lazy_static! {
    static ref EMPTY_PACKET: model::Packet = model::Packet::default();
}

macro_rules! osu {
    () => {
        panic!("osu!")
    };
}
//impl QMetaType for FieldListModel<'static> {}
impl<'a> PacketListModel<'a> {
    #[allow(non_snake_case)]
    fn on_clicked(&self, idx: QModelIndex) {
        println!("packet idx {}", idx.row());
        if let Ok(mut fm) = self.field.try_borrow_mut() {
            (&*fm as &dyn QAbstractItemModel).begin_reset_model();
            fm.packet = &self.list.0[0].1.packets[idx.row() as usize];
            (&*fm as &dyn QAbstractItemModel).end_reset_model();
        } else {
            osu!()
        }
    }
}

impl QAbstractListModel for PacketListModel<'_> {
    fn row_count(&self) -> i32 {
        self.list.0[0].1.packets.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let pos = index.row() as usize;
        if pos >= self.list.0[0].1.packets.len() {
            return QVariant::default();
        }
        if role != ItemDataRole::DisplayRole as i32 {
            return QVariant::default();
        }

        //println!("osugame {}", self.list.0[0].1.packets[pos].name);
        QString::from(self.list.0[0].1.packets[pos].name.clone()).into()
    }
}

#[derive(QObject)]
struct FieldListModel<'a> {
    base: qt_base_class!(trait QAbstractItemModel),
    packet: &'a model::Packet,
}

impl Clone for FieldListModel<'_> {
    fn clone(&self) -> Self {
        Default::default()
    }
}

impl Default for FieldListModel<'_> {
    fn default() -> Self {
        FieldListModel {
            base: QObjectCppWrapper::default(),
            packet: &EMPTY_PACKET,
        }
    }
}
impl FieldListModel<'_> {
    fn item(&self, idx: QModelIndex) -> Option<&dyn TreeNode> {
        // BitFieldItem
        if idx.id() & (1 << 32) > 0 {
            let r = idx.id() ^ (1 << 32);

            // TODO(aptny): https://github.com/rust-lang/rust/issues/53667
            if let Some(a) = self.packet.data.get(r) {
                if let model::Field::Bits { name: _, bits } = a {
                    if let Some(b) = bits.get(idx.row() as usize) {
                        return Some(b);
                    }
                }
            }
        }

        // Field
        if let Some(a) = self.packet.data.get(idx.row() as usize) {
            return Some(a);
        }

        None
    }
}

/// Implements a tree model (name, type, units) for the fields of a Packet.
impl QAbstractItemModel for FieldListModel<'_> {
    fn index(&self, row: i32, column: i32, parent: QModelIndex) -> QModelIndex {
        let e = self as &dyn QAbstractItemModel;

        // if the parent is a bitfield item, it has no children. return the root.
        if parent.id() & (1 << 32) > 0 {
            return e.create_index(-1, -1, 0);
        }

        // if the parent is the root, return a pristine index w/ the r and c.
        if !parent.is_valid() {
            return e.create_index(row, column, 0);
        }

        // the parent is a bitfield and we need to return a bitfield item.
        e.create_index(row, column, (parent.row() as usize) | (1 << 32))
    }

    fn parent(&self, index: QModelIndex) -> QModelIndex {
        let e = self as &dyn QAbstractItemModel;
        println!(
            "parent() for row={} col={} id={}",
            index.row(),
            index.column(),
            index.id()
        );
        // if this is a bitfield item, return the bitfield.
        if index.id() & (1 << 32) > 0 {
            return e.create_index((index.id() ^ (1 << 32)) as i32, 0, 0);
        }

        e.create_index(-1, -1, 0)
    }

    fn row_count(&self, parent: QModelIndex) -> i32 {
        // parent is a bitfield item
        if (parent.id() & (1 << 32)) > 0 {
            return 0;
        }

        // root item
        if !parent.is_valid() {
            return self.packet.data.len() as i32;
        }

        // parent is a bitfield
        if let model::Field::Bits { name: _, bits: x } = &self.packet.data[parent.row() as usize] {
            println!("bitfield row # = {}", x.len());
            return x.len() as i32;
        }

        0
    }

    fn column_count(&self, _parent: QModelIndex) -> i32 {
        3
    }

    // TODO(aptny): less bad please
    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        /*println!(
            "data() {} {} {} {} {}",
            index.row(),
            index.column(),
            index.id(),
            index.is_valid(),
            role
        );*/
        if role != ItemDataRole::DisplayRole as i32 {
            return QVariant::default();
        }

        match self.item(index) {
            Some(x) => match x.data(index.column() as usize) {
                Some(x) => QString::from(x.clone()).into(),
                None => QVariant::default(),
            },
            None => QVariant::default(),
        }
    }
}

trait TreeNode {
    fn data(&self, col: usize) -> Option<String>;
}

impl TreeNode for model::Field {
    fn data(&self, col: usize) -> Option<String> {
        match self {
            model::Field::Bits { name, bits: _ } => Some(match col {
                0 => name.clone(),
                1 => "bitfield".to_owned(),
                _ => "".to_owned(),
            }),
            model::Field::Plain { name, typ, units } => Some(
                match col {
                    0 => name,
                    1 => typ,
                    _ => units,
                }
                .clone(),
            ),
        }
    }
}

impl TreeNode for model::BitFieldItem {
    fn data(&self, col: usize) -> Option<String> {
        Some(match col {
            0 => self.name.clone(),
            1 => "field".to_owned(),
            _ => "".to_owned(),
        })
    }
}

qrc!(init_rscs, "app/qml" {
   "root.qml",
});

impl QMetaType for PacketListModel<'static> {}
fn main() {
    //qml_register_type::<model::Packet>(cstr!("Packet"), 1, 0, cstr!("Packet"));
    //    FieldListModel::register(Some(cstr!("FieldListModel")));
    PacketListModel::register_type(cstr!("PacketListModel"));
    //    println!("id: {}", FieldListModel::id());
    qml_register_type::<FieldListModel>(cstr!("FieldListModel"), 1, 0, cstr!("FieldListModel"));
    //qml_register_type::<PacketListModel>(cstr!("PacketListModel"), 1, 0, cstr!("PacketListModel"));
    let mut engine = QmlEngine::new();

    let mut list = model::Model::new();
    let err: Result<(), anyhow::Error> = fs::read_dir("data/")
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_file())
        .filter(|x| !x.file_name().to_str().unwrap().contains("ignore"))
        .map(|x| list.add_backing(&x.path()))
        .collect();
    err.unwrap();

    let detail_model = Rc::new(RefCell::new(FieldListModel {
        base: Default::default(),
        packet: &EMPTY_PACKET, //&self.list.0[0].1.packets[idx.row() as usize],
    }));

    let overview_model = RefCell::new(PacketListModel {
        list: &list,
        base: Default::default(),
        on_clicked: Default::default(),
        field: detail_model.clone(),
    });

    unsafe {
        engine.set_object_property("listModel".into(), QObjectPinned::new(&overview_model));
        engine.set_object_property(
            "detailModel".into(),
            QObjectPinned::new(&*Rc::into_raw(detail_model)), // NB: leaking this doesn't actually matter?
        );
    }

    init_rscs();
    engine.load_file("qrc:/app/qml/root.qml".into());
    engine.exec();
}
