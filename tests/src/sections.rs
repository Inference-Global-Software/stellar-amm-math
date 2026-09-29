//! The sections of a WebAssembly module: enough of the binary format to find
//! a contract's code, its function bodies and its interface, the
//! `contractspecv0` custom section.

/// The payload of the module's code section.
///
/// # Errors
///
/// When `module` is not a well-formed sequence of sections, or has no code
/// section or two.
pub fn code(module: &[u8]) -> Result<&[u8], String> {
    the_one(&sections(module)?, |section| (section.id == CODE).then_some(section.payload), "code")
}

/// The bodies of the functions the module defines, in the order of its code
/// section, each its locals and its instructions as bytes. In a module that
/// imports no function, body `i` is function `i`.
///
/// # Errors
///
/// When [`code`] fails, or the code section is not a vector of bodies that
/// ends where the section ends.
pub fn function_bodies(module: &[u8]) -> Result<Vec<&[u8]>, String> {
    let mut rest = code(module)?;
    let count = leb128(&mut rest)?;
    let mut bodies = Vec::new();
    for _ in 0..count {
        let size = leb128(&mut rest)?;
        let (body, tail) = rest.split_at_checked(size).ok_or("a function body runs past the code section")?;
        bodies.push(body);
        rest = tail;
    }
    if !rest.is_empty() {
        return Err("the code section does not end where its bodies do".to_owned());
    }
    Ok(bodies)
}

/// The contract's interface: the payload of the module's `contractspecv0`
/// custom section, a sequence of XDR `ScSpecEntry` values.
///
/// # Errors
///
/// When `module` is not a well-formed sequence of sections, or has no
/// `contractspecv0` section or two.
pub fn interface(module: &[u8]) -> Result<&[u8], String> {
    custom(&sections(module)?, "contractspecv0")
}

/// The id of the code section.
const CODE: u8 = 10;

/// A section: its id and its contents.
struct Section<'a> {
    id: u8,
    payload: &'a [u8],
}

/// The sections of `module`, in order.
fn sections(module: &[u8]) -> Result<Vec<Section<'_>>, String> {
    let mut rest = module.strip_prefix(b"\0asm\x01\0\0\0").ok_or("not a WebAssembly 1.0 module")?;
    let mut sections = Vec::new();
    while let Some((&id, tail)) = rest.split_first() {
        rest = tail;
        let size = leb128(&mut rest)?;
        let (payload, tail) = rest.split_at_checked(size).ok_or("a section runs past the end of the module")?;
        rest = tail;
        sections.push(Section { id, payload });
    }
    Ok(sections)
}

/// The payload of the one custom section called `name`.
fn custom<'a>(sections: &[Section<'a>], name: &str) -> Result<&'a [u8], String> {
    let named = |section: &Section<'a>| {
        if section.id != 0 {
            return None;
        }
        let mut body = section.payload;
        let name_len = leb128(&mut body).ok()?;
        let (section_name, payload) = body.split_at_checked(name_len)?;
        (section_name == name.as_bytes()).then_some(payload)
    };
    the_one(sections, named, name)
}

/// The one section `pick` picks.
fn the_one<'a>(
    sections: &[Section<'a>],
    pick: impl Fn(&Section<'a>) -> Option<&'a [u8]>,
    what: &str,
) -> Result<&'a [u8], String> {
    let mut picked = sections.iter().filter_map(pick);
    let found = picked.next().ok_or_else(|| format!("the module has no {what} section"))?;
    if picked.next().is_some() {
        return Err(format!("the module has two {what} sections"));
    }
    Ok(found)
}

/// An unsigned LEB128 of at most 32 bits, as section sizes are.
fn leb128(bytes: &mut &[u8]) -> Result<usize, String> {
    let mut value = 0_u64;
    for shift in [0, 7, 14, 21, 28] {
        let (&byte, rest) = bytes.split_first().ok_or("a LEB128 runs past the end of the module")?;
        *bytes = rest;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            let value = u32::try_from(value).map_err(|_| "a LEB128 overflows 32 bits")?;
            return usize::try_from(value).map_err(|_| "a size does not fit usize".to_owned());
        }
    }
    Err("a LEB128 longer than five bytes".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A module with a type section, two custom sections and a code section.
    const MODULE: &[u8] = &[
        0, b'a', b's', b'm', 1, 0, 0, 0, // magic, version
        1, 1, 0, // type section, one byte, no types
        0, 18, 14, b'c', b'o', b'n', b't', b'r', b'a', b'c', b't', b's', b'p', b'e', b'c', b'v', b'0', 1, 2, 3, //
        0, 6, 5, b'o', b't', b'h', b'e', b'r', //
        10, 9, 2, // code section, nine bytes, two bodies:
        2, 0, 0x0b, // no locals, `end`
        4, 1, 1, 0x7f, 0x0b, // one `i32` local, `end`
    ];

    fn custom_section(module: &[u8], name: &str) -> Result<Vec<u8>, String> {
        custom(&sections(module)?, name).map(<[u8]>::to_vec)
    }

    /// A module whose only section is a code section with `payload`.
    fn with_code(payload: &[u8]) -> Vec<u8> {
        let mut module = MODULE[..8].to_vec();
        module.extend([CODE, u8::try_from(payload.len()).expect("a short payload")]);
        module.extend_from_slice(payload);
        module
    }

    #[test]
    fn finds_the_named_custom_section_and_the_code() {
        assert_eq!(interface(MODULE), Ok(&[1, 2, 3][..]));
        assert_eq!(custom_section(MODULE, "other"), Ok(vec![]));
        assert!(custom_section(MODULE, "missing").is_err());
        assert_eq!(code(MODULE), Ok(&[2, 2, 0, 0x0b, 4, 1, 1, 0x7f, 0x0b][..]));
    }

    #[test]
    fn reads_the_function_bodies() {
        assert_eq!(function_bodies(MODULE), Ok(vec![&[0, 0x0b][..], &[1, 1, 0x7f, 0x0b][..]]));
        assert_eq!(function_bodies(&with_code(&[0])), Ok(vec![]));
        let malformed: [(&[u8], &str); 4] = [
            (&[1, 3, 0, 0x0b], "runs past the code section"),
            (&[2, 2, 0, 0x0b], "runs past the end"),
            (&[1, 2, 0, 0x0b, 0], "does not end where its bodies do"),
            (&[0, 0], "does not end where its bodies do"),
        ];
        for (payload, expected) in malformed {
            let refused = function_bodies(&with_code(payload)).expect_err("a malformed code section was read");
            assert!(refused.contains(expected), "{payload:?}: {refused}");
        }
        assert!(function_bodies(&MODULE[..39]).unwrap_err().contains("no code"));
    }

    #[test]
    fn refuses_malformed_modules() {
        let mut twice = MODULE.to_vec();
        twice.extend_from_slice(&MODULE[11..31]);
        assert!(interface(&twice).unwrap_err().contains("two"));
        assert!(interface(&MODULE[..20]).is_err());
        assert!(interface(&MODULE[1..]).is_err());
        assert!(code(&MODULE[..39]).unwrap_err().contains("no code"));
    }

    #[test]
    fn reads_leb128() {
        let cases: [(&[u8], usize); 4] =
            [(&[0], 0), (&[0x7f], 127), (&[0x80, 0x01], 128), (&[0xff, 0xff, 0xff, 0xff, 0x0f], u32::MAX as usize)];
        for (bytes, expected) in cases {
            let mut rest = bytes;
            assert_eq!(leb128(&mut rest), Ok(expected), "{bytes:?}");
            assert!(rest.is_empty());
        }
        assert!(leb128(&mut &[0xff, 0xff, 0xff, 0xff, 0x1f][..]).is_err());
        assert!(leb128(&mut &[0x80][..]).is_err());
    }
}
