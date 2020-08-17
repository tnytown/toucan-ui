#![recursion_limit = "256"]

use prelude::TreeStoreExtManual;
use std::path::Path;
use vgtk::ext::*;
use vgtk::lib::gio::ApplicationFlags;
use vgtk::lib::{glib, gtk::*};
use vgtk::{gtk, run, Component, UpdateAction, VNode};

mod model;

#[derive(Debug)]
struct Root {
    model: model::Model,
}

impl Default for Root {
    fn default() -> Self {
        Root {
            model: model::Model::new_with_backing(Path::new("data/switchboard.yaml")).unwrap(),
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    Exit,
}

use model::Packet;
impl Component for Root {
    type Message = Message;
    type Properties = ();

    fn update(&mut self, msg: Self::Message) -> UpdateAction<Self> {
        match msg {
            Message::Exit => {
                vgtk::quit();
                UpdateAction::None
            }
        }
    }

    fn view(&self) -> VNode<Root> {
        let p = &self.model.0[0].1.packets[0];

        #[rustfmt::skip]
        gtk! {
            <Application::new_unwrap(Some("me.unown.toucan-ui"), ApplicationFlags::empty())>
                <Window border_width=20 on destroy=|_| Message::Exit>
                    <@FieldView packet=p />
				</Window>
            </Application>
        }
    }
}

#[derive(Default, Clone)]
pub struct FieldView {
    packet: model::Packet,
}

// TODO(aptny): look into implementing TreeModelExt for model::Packet, then casting
impl Component for FieldView {
    type Message = ();
    type Properties = Self;

    fn view(&self) -> VNode<Self> {
        /* https://gtk-rs.org/docs/gtk/trait.TreeStoreExt.html
         * https://gtk-rs.org/docs/gtk/prelude/trait.TreeStoreExtManual.html
         * https://github.com/gtk-rs/examples/blob/master/src/bin/list_store.rs */
        fn tree_store_setup(view: &TreeView) {
            struct Column<'a, T: CellRendererExt> {
                renderer: &'a T,
                name: &'a str,
            };
            let cols = [
                Column {
                    renderer: &CellRendererText::new(),
                    name: "name",
                },
                Column {
                    renderer: &CellRendererText::new(),
                    name: "type",
                },
                Column {
                    renderer: &CellRendererText::new(),
                    name: "unit",
                },
            ];
            view.set_headers_visible(true);
            for (i, d) in cols.iter().enumerate() {
                let col = TreeViewColumn::new();
                col.pack_start(d.renderer, true);
                col.set_title(d.name);
                col.set_sort_column_id(i as i32);
                col.set_expand(true);
                col.add_attribute(d.renderer, "text", i as i32);
                col.set_resizable(true);
                view.append_column(&col);
            }
        }

        let str = TreeStore::new(&[glib::Type::String, glib::Type::String, glib::Type::String]);

        for f in &self.packet.data {
            match f {
                model::Field::Bits { name, bits } => {
                    let root = str.insert_with_values(
                        None,
                        None,
                        &[0, 1],
                        &[name, &"bitfield".to_string()],
                    );

                    for x in bits {
                        str.insert_with_values(
                            Some(&root),
                            None,
                            &[0, 1],
                            &[&x.name, &"field".to_string()],
                        );
                    }
                }
                model::Field::Plain { name, typ, units } => {
                    str.insert_with_values(None, None, &[0, 1, 2], &[name, typ, units]);
                }
            }
        }

        #[rustfmt::skip]
        gtk! {
            <TreeView headers_clickable=true
             model=Some(str.upcast::<TreeModel>())
             on show=|view| tree_store_setup(view) />
        }
    }

    fn update(&mut self, _msg: Self::Message) -> UpdateAction<Self> {
        UpdateAction::Render
    }

    fn create(p: Self::Properties) -> Self {
        p
    }

    fn change(&mut self, _props: Self::Properties) -> UpdateAction<Self> {
        unimplemented!("add a Component::change() implementation")
    }

    fn mounted(&mut self) {}
    fn unmounted(&mut self) {}
}

fn main() {
    pretty_env_logger::init();
    std::process::exit(run::<Root>());
}
