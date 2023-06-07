use wasmiter::component;

macro_rules! bytes {
    ($($e:expr,)*) => {{
        let mut bytes = std::vec::Vec::<u8>::new();
        $(
            bytes.extend_from_slice(AsRef::<[u8]>::as_ref(&$e));
        )*
        bytes
    }};
}

#[test]
fn type_section() {
    let bytes = [
        1u8,  // count
        0x60, // func
        1,    // parameter count
        0x7F, // i32
        1,    // result count
        0x7E, // i64
    ];

    insta::assert_display_snapshot!(component::TypesComponent::new(0, bytes.as_slice()).unwrap());
}

#[test]
fn export_section() {
    let bytes = bytes! {
        [
            1, // count
            0xC, // name length
        ],
        b"myExportName",
        [
            0, // export func
            0, // funcidx
        ],
    };

    insta::assert_display_snapshot!(component::ExportsComponent::new(0, bytes.as_slice()).unwrap());
}

#[test]
fn import_section() {
    let bytes = bytes! {
        [
            4, // count
            3, // module name length
        ],
        b"env",
        [0xB], // name length
        b"doSomeStuff",
        [
            0, // import func
            0, // typeidx
            3, // module name length
        ],
        b"env",
        [6], // name length
        b"memory",
        [
            2, // import memory
            0, // limit w/o maximum
            0x10, // limit minimum
            2, // module name length
        ],
        b"rt",
        [0xA], // name length
        b"references",
        [
            1, // import table,
            0x6F, // externref
            0, 0, // limits
            2, // module name length
        ],
        b"rt",
        [8], // name length
        b"stackptr",
        [
            3, // import global
            0x7F, // i32
            1, // mutable
        ],
    };

    insta::assert_display_snapshot!(component::ImportsComponent::new(0, bytes.as_slice()).unwrap());
}
