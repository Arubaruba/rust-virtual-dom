#[macro_export]
macro_rules! template {
    ($($inner:tt)*) => ({
        let mut el = $crate::VirtualElement::new();
        // "+" is disallowed at the top level, so no sibling elements will be returned
        let _ = inner_template!(top_level, $($inner)*)(&mut el);
        el
    });
}

#[macro_export]
macro_rules! inner_template {
    ($tl:ident, ) => (|_: &mut $crate::VirtualElement| Vec::<$crate::VirtualNode>::new());
    ($tl:ident, [$($key:ident=$val:expr)*]$($inner:tt)*) => (|el: &mut $crate::VirtualElement| {
        $(el.attributes.insert(stringify!($key).to_string(), $val.to_string());)*
        inner_template!($tl, $($inner)*)(el)
    });
    ($tl:ident, >($($inner_parens:tt)*)$($inner:tt)*) => (|el: &mut $crate::VirtualElement| {
        let mut el_parens = $crate::VirtualElement::new();
        let mut el_parens_siblings = inner_template!(not_top_level, $($inner_parens)*)(&mut el_parens);
        el.child_nodes.push($crate::VirtualNode::Element(el_parens));
        el.child_nodes.append(&mut el_parens_siblings);

        let mut el_remaining_siblings = inner_template!(not_top_level, $($inner)*)(el);
        el.child_nodes.append(&mut el_remaining_siblings);

        Vec::<$crate::VirtualNode>::new()
    });
    ($tl:ident, >$($inner:tt)*) => (|el: &mut $crate::VirtualElement| {
        let mut el_remaining = $crate::VirtualElement::new();
        let mut el_remaining_siblings = inner_template!(not_top_level, $($inner)*)(&mut el_remaining);
        el.child_nodes.push($crate::VirtualNode::Element(el_remaining));
        el.child_nodes.append(&mut el_remaining_siblings);

        Vec::<$crate::VirtualNode>::new()
    });
    (not_top_level, +($($inner_parens:tt)*)$($inner:tt)*) => (|el: &mut $crate::VirtualElement| {
        let mut el_parens = $crate::VirtualElement::new();
        let mut el_parens_siblings = inner_template!(not_top_level, $($inner)*)(&mut el_parens);

        let mut el_remaining_siblings = inner_template!(not_top_level, $($inner)*)(el);

        let mut els = Vec::new();

        els.push($crate::VirtualNode::Element(el_parens));
        els.append(&mut el_parens_siblings);
        els.append(&mut el_remaining_siblings);
        els
    });
    (not_top_level, +$($inner:tt)*) => (|_: &mut $crate::VirtualElement| {
        let mut el_remaining = $crate::VirtualElement::new();
        let mut el_remaining_siblings =
            inner_template!(not_top_level, $($inner)*)(&mut el_remaining);

        let mut els = Vec::new();

        els.push($crate::VirtualNode::Element(el_remaining));
        els.append(&mut el_remaining_siblings);
        els
    });
    ($tl:ident, {$bind:expr}$($inner:tt)*) => (|el: &mut $crate::VirtualElement| {
        el.child_nodes.append(&mut $crate::VirtualDom::from($bind).0);
        inner_template!($tl, $($inner)*)(el)
    });
    ($tl:ident, .$classes:ident$($inner:tt)*) => (|el: &mut $crate::VirtualElement| {
        let classes = if let Some(existing_classes) = el.attributes.get("class") {
            existing_classes.to_string() + " " + stringify!($classes)
        } else {
            stringify!($classes).to_string()
        };
        el.attributes.insert("class".to_string(), classes);
        inner_template!($tl, $($inner)*)(el)
    });
    ($tl:ident, #$id:ident$($inner:tt)*) => (|el: &mut $crate::VirtualElement| {
        el.attributes.insert("id".to_string(), stringify!($id).to_string());
        inner_template!($tl, $($inner)*)(el)
    });
    ($tl:ident, $name:ident$($inner:tt)*) => (|el: &mut $crate::VirtualElement| {
        el.name = stringify!($name).to_string();
        inner_template!($tl, $($inner)*)(el)
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn template_name_class_id() {
        let mut el = ::VirtualElement::new();
        assert_eq!(el, template!(div));

        el.name = "a".to_string();
        assert_eq!(el, template!(a));

        el.attributes.insert("class".into(), "active red".into());
        assert_eq!(el, template!(a.active.red));

        el.attributes.insert("id".into(), "main".into());
        assert_eq!(el, template!(a#main.active.red));
    }

    #[test]
    fn template_bind_inner_value() {
        let mut el = ::VirtualElement::new();
        el.child_nodes.push(::VirtualNode::Text("some inner text".into()));
        el.child_nodes.push(::VirtualNode::Text("4".into()));
        assert_eq!(el, template!(div{"some inner text"}{1 + 3}));
    }

    #[test]
    fn template_bind_attribute() {
        let mut el = ::VirtualElement::new();
        el.attributes.insert("width".into(), "44".into());
        assert_eq!(el, template!(div[width={40 + 4}]));
    }

    #[test]
    fn template_child_nodes () {
        let mut el = ::VirtualElement::new();
        el.child_nodes.push(::VirtualNode::Element(::VirtualElement::new()));
        assert_eq!(el, template!(div>div));
    }

    #[test]
    fn template_sibling_nodes () {
        let mut el = ::VirtualElement::new();
        el.child_nodes.push(::VirtualNode::Element(::VirtualElement::new()));
        el.child_nodes.push(::VirtualNode::Element(::VirtualElement::new()));
        assert_eq!(el, template!(div>div+div));
    }

    #[test]
    fn template_grouping () {
        let mut el = ::VirtualElement::new();
        let mut group_el = ::VirtualElement::new();

        group_el.child_nodes.push(::VirtualNode::Element(::VirtualElement::new()));
        el.child_nodes.push(::VirtualNode::Element(group_el));
        el.child_nodes.push(::VirtualNode::Element(::VirtualElement::new()));

        assert_eq!(el, template!(div>(div>div)+(div)));
    }
}
