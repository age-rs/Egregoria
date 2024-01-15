use common::TransparentMap;
use mlua::{FromLua, Table};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

mod load;
mod prototypes;
mod tests;
mod types;
mod validation;

pub use load::*;
pub use prototypes::*;
pub use types::*;

crate::gen_prototypes!(
    companies: GoodsCompanyID => GoodsCompanyPrototype,
    items:     ItemID         => ItemPrototype,
    solar:     SolarPanelID   => SolarPanelPrototype extends GoodsCompanyID,
);

/// A prototype is a collection of data that is dynamically loaded with Lua and defines a type of object
pub trait Prototype: 'static + Sized {
    /// The parent prototype of this prototype (optional). Use NoParent if there is no parent
    type Parent: ConcretePrototype;

    /// The type of the ID of the prototype
    type ID: Copy + Clone + Eq + Ord + Hash + 'static;

    /// The name of the prototype used to parse the prototype from Lua's data table
    const NAME: &'static str;

    /// Parse the prototype from a Lua table
    fn from_lua(table: &Table) -> mlua::Result<Self>;

    /// The ID of the prototype
    fn id(&self) -> Self::ID;

    /// The parent of the prototype
    fn parent(&self) -> Option<&Self::Parent> {
        None
    }

    /// util function to recursively insert the parents of this prototype into the prototypes lists
    fn insert_parents(&self, prototypes: &mut Prototypes) {
        if let Some(p) = self.parent() {
            Self::Parent::storage_mut(prototypes).insert(p.id(), p.clone());
            p.insert_parents(prototypes);
        }
    }
}

/// A concrete prototype is a prototype that has a static storage and ordering (it is not virtual)
pub trait ConcretePrototype: Prototype + Clone {
    fn ordering(prototypes: &Prototypes) -> &[Self::ID];
    fn storage(prototypes: &Prototypes) -> &TransparentMap<Self::ID, Self>;
    fn storage_mut(prototypes: &mut Prototypes) -> &mut TransparentMap<Self::ID, Self>;
}

pub trait PrototypeID: Debug + Copy + Clone + Eq + Ord + Hash + 'static {
    type Prototype: Prototype<ID = Self>;
}

#[derive(Clone)]
pub struct NoParent;

impl Prototype for NoParent {
    type Parent = NoParent;
    type ID = ();
    const NAME: &'static str = "no-parent";

    fn from_lua(_table: &Table) -> mlua::Result<Self> {
        unreachable!()
    }

    fn id(&self) -> Self::ID {
        unreachable!()
    }
}

impl ConcretePrototype for NoParent {
    fn ordering(_prototypes: &Prototypes) -> &[Self::ID] {
        &[]
    }

    fn storage(_prototypes: &Prototypes) -> &TransparentMap<Self::ID, Self> {
        unreachable!()
    }

    fn storage_mut(_prototypes: &mut Prototypes) -> &mut TransparentMap<Self::ID, Self> {
        unreachable!()
    }
}

static mut PROTOTYPES: Option<&'static Prototypes> = None;

#[inline]
pub fn prototypes() -> &'static Prototypes {
    #[cfg(debug_assertions)]
    {
        assert!(unsafe { PROTOTYPES.is_some() });
    }

    // Safety: Please just don't use prototypes before they were loaded... We can allow this footgun
    unsafe { PROTOTYPES.unwrap_unchecked() }
}

pub fn try_prototypes() -> Option<&'static Prototypes> {
    unsafe { PROTOTYPES }
}

#[inline]
pub fn prototype<ID: PrototypeID>(id: ID) -> &'static <ID as PrototypeID>::Prototype
where
    ID::Prototype: ConcretePrototype,
{
    match <ID as PrototypeID>::Prototype::storage(prototypes()).get(&id) {
        Some(v) => v,
        None => panic!("no prototype for id {:?}", id),
    }
}

pub fn try_prototype<ID: PrototypeID>(id: ID) -> Option<&'static <ID as PrototypeID>::Prototype>
where
    ID::Prototype: ConcretePrototype,
{
    <ID as PrototypeID>::Prototype::storage(try_prototypes()?).get(&id)
}

pub fn prototypes_iter<T: ConcretePrototype>() -> impl Iterator<Item = &'static T> {
    let p = prototypes();
    let storage = T::storage(p);
    T::ordering(p).iter().map(move |id| &storage[id])
}

pub fn prototypes_iter_ids<T: ConcretePrototype>() -> impl Iterator<Item = T::ID> {
    T::ordering(prototypes()).iter().copied()
}

#[macro_export]
macro_rules! gen_prototypes {
    ($($name:ident : $id:ident => $t:ident $(extends $parent_id:ident)?,)+) => {
        $(
            prototype_id!($id => $t);
        )+

        $(
            $(
            impl From<$parent_id> for $id {
                fn from(v: $parent_id) -> Self {
                    Self(v.0)
                }
            }

            impl From<$id> for $parent_id {
                fn from(v: $id) -> Self {
                    Self(v.0)
                }
            }
            )?
        )+

        #[derive(Default)]
        struct Orderings {
            $(
                $name: Vec<$id>,
            )+
        }

        #[derive(Default)]
        pub struct Prototypes {
            $(
                $name: TransparentMap<$id, $t>,
            )+
            orderings: Orderings,
        }

        $(
        impl ConcretePrototype for $t {
            fn ordering(prototypes: &Prototypes) -> &[Self::ID] {
                &prototypes.orderings.$name
            }

            fn storage(prototypes: &Prototypes) -> &TransparentMap<Self::ID, Self> {
                &prototypes.$name
            }

            fn storage_mut(prototypes: &mut Prototypes) -> &mut TransparentMap<Self::ID, Self> {
                &mut prototypes.$name
            }
        }

        impl $t {
            pub fn iter() -> impl Iterator<Item = &'static Self> {
                crate::prototypes_iter::<Self>()
            }
            pub fn iter_ids() -> impl Iterator<Item = $id> {
                crate::prototypes_iter_ids::<Self>()
            }
        }
        )+

        impl Prototypes {
            pub(crate) fn print_stats(&self) {
                $(
                    log::info!("loaded {} {}", <$t>::storage(self).len(), <$t>::NAME);
                )+
            }

            pub(crate) fn compute_orderings(&mut self) {
                self.orderings = Orderings {
                    $(
                        $name: {
                            let mut v = <$t>::storage(self).keys().copied().collect::<Vec<_>>();
                            v.sort_by_key(|id| {
                                let proto = &self.$name[id];
                                (&proto.order, proto.id)
                            });
                            v
                        },
                    )+
                }
            }
        }

        fn parse_prototype(table: Table, prototypes: &mut Prototypes) -> Result<(), PrototypeLoadError> {
            let _type = table.get::<_, String>("type")?;
            let _type_str = _type.as_str();
            match _type_str {
                $(
                    <$t>::NAME => {
                        let proto: $t = Prototype::from_lua(&table).map_err(|e| {
                              PrototypeLoadError::PrototypeLuaError(_type_str.to_string(), table.get::<_, String>("name").unwrap(), e)
                        })?;

                        proto.insert_parents(prototypes);

                        if let Some(v) = prototypes.$name.insert((&proto.name).into(), proto) {
                            log::warn!("duplicate {} with name: {}", <$t>::NAME, v.name);
                        }
                    }
                ),+
                _ => {
                    if let Ok(s) = table.get::<_, String>("type") {
                        log::warn!("unknown prototype: {}", s)
                    }
                }
            }

            Ok(())
        }
    };
}

#[macro_export]
macro_rules! prototype_id {
    ($id:ident => $proto:ty) => {
        #[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $id(pub(crate) u64);

        egui_inspect::debug_inspect_impl!($id);

        impl Debug for $id {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                if let Some(v) = crate::try_prototype(*self) {
                    return write!(f, "{}({:?})", stringify!($id), v.name);
                }
                write!(f, "{}({})", stringify!($id), self.0)
            }
        }

        impl $id {
            #[inline]
            pub fn new(v: &str) -> $id {
                Self(common::hash_u64(v))
            }

            #[inline]
            pub fn prototype(self) -> &'static $proto {
                crate::prototype(self)
            }

            #[inline]
            pub fn hash(&self) -> u64 {
                self.0
            }
        }

        impl<'a> From<&'a str> for $id {
            fn from(v: &'a str) -> Self {
                Self(common::hash_u64(v))
            }
        }

        impl<'a> From<&'a String> for $id {
            fn from(v: &'a String) -> Self {
                Self(common::hash_u64(&*v))
            }
        }

        impl<'a> mlua::FromLua<'a> for $id {
            fn from_lua(v: mlua::Value<'a>, _: &'a mlua::Lua) -> mlua::Result<Self> {
                match v {
                    mlua::Value::String(s) => {
                        let Ok(v) = s.to_str() else {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: "string",
                                to: stringify!($id),
                                message: Some("expected utf-8 string".into()),
                            });
                        };
                        Ok(Self(common::hash_u64(v)))
                    }
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: v.type_name(),
                        to: stringify!($id),
                        message: Some("expected string".into()),
                    }),
                }
            }
        }

        impl crate::PrototypeID for $id {
            type Prototype = $proto;
        }
    };
}

fn get_with_err<'a, T: FromLua<'a>>(t: &Table<'a>, field: &'static str) -> mlua::Result<T> {
    t.get::<_, T>(field)
        .map_err(|e| mlua::Error::external(format!("field {}: {}", field, e)))
}
