use enum_primitive_derive::Primitive;
use num_traits;

use qmetaobject::QVariant;
pub mod model;
pub mod qthax;

pub use model_macro::TreeModel;
pub use model_macro::TreeNode;

#[derive(Primitive)]
pub enum ItemDataRole {
    DisplayRole = 0,
    DecorationRole = 1,
    EditRole = 2,
    ToolTipRole = 3,
    StatusTipRole = 4,
    WhatsThisRole = 5,

    FontRole = 6,
    TextAlignmentRole = 7,
    BackgroundRole = 8,
    ForegroundRole = 9,
    CheckStateRole = 10,
    AccessibleTextRole = 11,
    AccessibleDescriptionRole = 12,
    SizeTipRole = 13,

    UserRole = 0x0100,
    NodeLabelRole = 0x0101,
    FieldLabelRole = 0x0102,
    FieldCountRole = 0x0103,
    UnknownRole = 0x7fffffff,
}

impl Default for ItemDataRole {
    fn default() -> Self {
        Self::UnknownRole
    }
}

pub trait TreeNode {
    /// Returns a reference to the nth child node, if one exists.
    fn get(&self, n: usize) -> Option<&dyn TreeNode>;

    /// Returns a mutable reference to the nth child node, if one exists.
    fn get_mut(&mut self, n: usize) -> Option<&mut dyn TreeNode>;

    /// Returns the number of children on this node.
    fn len(&self) -> usize;

    /// Returns the names of the columns. Only needed on nodes intended for root usage.
    fn columns(&self) -> &[&'static str];

    /// Returns the nth field's name if it exists.
    fn field(&self, idx: i32) -> QVariant;

    /// Returns the nth field's data if it exists.
    fn data(&self, idx: i32) -> QVariant;

    /// Sets the nth field's data if it exists.
    fn set_data(&mut self, idx: i32, data: QVariant) -> bool;

    /// Returns the user-facing type name of the node.
    fn label(&self) -> &'static str;
}

/// Dummy TreeNode implementation for the unit type. Intended for use in leaf nodes.
impl TreeNode for () {
    fn get(&self, _n: usize) -> Option<&dyn TreeNode> {
        unimplemented!()
    }

    fn get_mut(&mut self, _n: usize) -> Option<&mut dyn TreeNode> {
        unimplemented!()
    }

    fn len(&self) -> usize {
        unimplemented!()
    }

    fn columns(&self) -> &[&'static str] {
        unimplemented!()
    }

    fn field(&self, idx: i32) -> QVariant {
        unimplemented!()
    }

    fn data(&self, _c: i32) -> QVariant {
        unimplemented!()
    }

    fn set_data(&mut self, _c: i32, _data: QVariant) -> bool {
        unimplemented!()
    }

    fn label(&self) -> &'static str {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn recursive_trait() {
        pub trait Recurse<T>
        where
            T: Recurse<T>,
        {
            fn child(&mut self) -> Option<&mut Vec<T>>;
            fn print(&self);
        }

        struct Bar;
        struct Foo {
            children: Vec<Bar>,
        }

        impl Recurse<Bar> for Foo {
            fn child(&mut self) -> Option<&mut Vec<Bar>> {
                Some(&mut self.children)
            }

            fn print(&self) {
                println!("foo");
            }
        }

        impl Recurse<()> for Bar {
            fn child(&mut self) -> Option<&mut Vec<()>> {
                None
            }

            fn print(&self) {
                println!("bar");
            }
        }

        impl Recurse<()> for () {
            fn child(&mut self) -> Option<&mut Vec<()>> {
                unimplemented!()
            }

            fn print(&self) {
                unimplemented!()
            }
        }

        let mut foo = Foo {
            children: vec![Bar {}],
        };
        foo.print();
        foo.child().unwrap()[0].print();
    }
}
