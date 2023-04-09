#![doc = include_str!("../README.md")]

use std::mem;

use proc_macro::TokenStream;
use quote::quote;
use syn::token::Pub;
use syn::*;

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

macro_rules! bail {
    ($span: expr, $($arg:tt)*) => {
        return Err(syn::Error::new_spanned($span, format!($($arg)*)))
    }
}

/// Returns `Ok(true)` if the attributes list contains a `#[fully_pub(exclude)]` attribute,
/// then remove it from the list.
///
/// If the attribute is ill-formatted or present more than once, returns an `Err`.
fn is_exclude(attrs: &mut Vec<Attribute>) -> Result<bool> {
    let mut is_exclude = false;

    for attr in mem::take(attrs) {
        if attr.path().is_ident(CRATE_NAME) {
            let arg = attr.parse_args::<Ident>()?;

            if arg != "exclude" {
                bail!(&arg, "unknown {CRATE_NAME} attribute `{arg}`");
            }

            if is_exclude {
                bail!(attr, "duplicate {CRATE_NAME} attribute `exclude`");
            }

            is_exclude = true;
        } else {
            attrs.push(attr);
        }
    }

    Ok(is_exclude)
}

/// Sets this visibility to public.
fn make_pub(vis: &mut Visibility) {
    *vis = Visibility::Public(Pub::default());
}

/// Explore the item `recursively` (or not), making it's fields
/// public.
fn explore_item(item: &mut Item, recursive: bool) -> Result<()> {
    match item {
        Item::Const(ItemConst { vis, attrs, .. })
        | Item::Enum(ItemEnum { vis, attrs, .. })
        | Item::Fn(ItemFn { vis, attrs, .. })
        | Item::Static(ItemStatic { vis, attrs, .. })
        | Item::Trait(ItemTrait { vis, attrs, .. })
        | Item::TraitAlias(ItemTraitAlias { vis, attrs, .. })
        | Item::Type(ItemType { vis, attrs, .. }) => {
            if !is_exclude(attrs)? {
                make_pub(vis);
            }
        }
        Item::ExternCrate(_) | Item::Macro(_) | Item::Use(_) => (),
        Item::ForeignMod(ItemForeignMod { attrs, items, .. }) => {
            if !is_exclude(attrs)? {
                for item in items {
                    match item {
                        ForeignItem::Fn(ForeignItemFn { vis, attrs, .. })
                        | ForeignItem::Static(ForeignItemStatic { vis, attrs, .. })
                        | ForeignItem::Type(ForeignItemType { vis, attrs, .. }) => {
                            if !is_exclude(attrs)? {
                                make_pub(vis);
                            }
                        }
                        ForeignItem::Macro(_) => (),
                        _ => (),
                    }
                }
            }
        }
        Item::Impl(ItemImpl {
            attrs,
            trait_,
            items,
            ..
        }) => {
            if trait_.is_none() && !is_exclude(attrs)? {
                for item in items {
                    match item {
                        ImplItem::Const(ImplItemConst { vis, attrs, .. })
                        | ImplItem::Fn(ImplItemFn { vis, attrs, .. })
                        | ImplItem::Type(ImplItemType { vis, attrs, .. }) => {
                            if !is_exclude(attrs)? {
                                make_pub(vis);
                            }
                        }
                        ImplItem::Macro(_) => (),
                        _ => (),
                    }
                }
            }
        }
        Item::Mod(ItemMod {
            vis,
            attrs,
            content: Some((_, content)),
            ..
        }) => {
            if !is_exclude(attrs)? {
                make_pub(vis);

                if recursive {
                    for item in content {
                        explore_item(item, recursive)?;
                    }
                }
            }
        }
        Item::Struct(ItemStruct {
            vis, attrs, fields, ..
        }) => {
            if !is_exclude(attrs)? {
                make_pub(vis);

                match fields {
                    Fields::Named(FieldsNamed { named: fields, .. })
                    | Fields::Unnamed(FieldsUnnamed {
                        unnamed: fields, ..
                    }) => {
                        for Field { vis, attrs, .. } in fields {
                            if !is_exclude(attrs)? {
                                make_pub(vis);
                            }
                        }
                    }
                    Fields::Unit => (),
                }
            }
        }
        Item::Union(ItemUnion {
            vis,
            attrs,
            fields: FieldsNamed { named: fields, .. },
            ..
        }) => {
            if !is_exclude(attrs)? {
                make_pub(vis);

                for Field { vis, attrs, .. } in fields {
                    if !is_exclude(attrs)? {
                        make_pub(vis);
                    }
                }
            }
        }
        _ => (),
    }

    Ok(())
}

/// Parse arguments to attr and then explore the item recursively,
/// making its parts public.
fn make_fully_pub(attr: Option<Ident>, item: &mut Item) -> Result<()> {
    let recursive = match attr {
        Some(ident) if ident == "recursive" => true,
        Some(ident) => bail!(ident, "invalid argument to `{CRATE_NAME}` attribute macro"),
        None => false,
    };

    explore_item(item, recursive)
}

/// Attribute macro that can be applied to any Rust item, and marks
/// all of its content as [`pub`](https://doc.rust-lang.org/std/keyword.pub.html).
///
/// Call it with the argument `recursive` to make it recursive over the content of
/// a nested `mod`: like so `#[fully_pub(recursive)]`.
///
/// Does nothing on `extern crate`, `use` and `mod` statements.
///
/// You can apply the `#[fully_pub(exclude)]` attribute to any content
/// of an item to exclude it from being marked as `pub`, if it would have been
/// otherwise.
/// 
/// # Exact Behaviour
/// 
/// This macro has the following behaviour depending on the kind of items it is applied on:
/// 
/// * `const`, `fn`, `static`, `trait` (and `trait` aliases) and `type` are all simply made `pub`.
/// Nested items in a `fn` are not affected.
/// * `macro_rule`, `extern crate`, `mod` statements and `use` are left as-is.
/// * `extern` modules will see all of their items (`const`, `fn` or `static`) made `pub`.
/// * `impl` blocks (excluding `impl Trait` blocks) get all their items
/// (`const`, `fn` or `static`) marked as `pub`
/// * `mod { /* ... */ }` are marked as `pub`, but their content is left untouched, unless
/// the `(recursive)` argument is passed to the attribute, in which case all of their items will
/// be marked `pub` recursively.
/// * `struct` and `union` get marked `pub` along with all their fields.
/// 
/// # Examples
///
/// ```
/// use fully_pub::fully_pub;
///
/// #[fully_pub]
/// struct User {
///     name: String,
///     age: i32,
///     #[fully_pub(exclude)]
///     secret: String,
/// }
///
/// #[fully_pub]
/// impl User {
///     fn new(name: String, age: i32, secret: String) -> Self {
///         Self { name, age, secret }
///     }
///
///     fn happy_birthday(&mut self) {
///         self.age += 1;
///     }
///
///     #[fully_pub(exclude)]
///     fn get_secret(&mut self) -> &str {
///         &self.secret
///     }
/// }
/// ```
/// 
/// ```
/// use fully_pub::fully_pub;
///
/// #[fully_pub(recursive)]
/// mod nested {
///     fn double(x: f32) -> f32 {
///         2.0 * x
///     }
///
///     fn square(x: f32) -> f32 {
///         x * x
///     }
///
///     mod deep {
///         use super::*;
///     
///         fn double_square(x: f32) -> f32 {
///             double(square(x))
///         }
///
///         #[fully_pub(exclude)]
///         fn square_double(x: f32) -> f32 {
///             square(double(x))
///         }
///     }
/// }
///
/// #[fully_pub] // simply makes the module pub, not its content
/// mod private_content {
///     fn secret_double(x: f32) -> f32 {
///         f32::from_bits(x.to_bits() + 0x800000) // evil floating point bit level hacking
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn fully_pub(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Option<Ident>);
    let mut item = parse_macro_input!(item as Item);

    match make_fully_pub(attr, &mut item) {
        Ok(_) => quote! { #item }.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
