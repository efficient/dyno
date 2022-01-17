use libc::Elf64_Ehdr;
use libc::Elf64_Shdr;
use libc::Elf64_Sym;
use libc::Elf64_Word;
use memmap2::MmapMut;
use regex::Regex;
use std::borrow::Cow;
use std::env::args;
use std::ffi::CStr;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::mem::size_of;
use std::os::raw::c_uchar;
use std::slice;

const SHT_SYMTAB: Elf64_Word = 2;
const STV_DEFAULT: c_uchar = 0;
const STV_HIDDEN: c_uchar = 2;

fn main() -> Result<(), String> {
	let mut expr = Cow::from(".*");
	let filename = filename(&mut expr)?;
	let expr = Regex::new(&format!("^{}$", expr)).map_err(or_string)?;
	let file = OpenOptions::new().read(true).write(true).open(filename).map_err(or_string)?;
	let mut file = unsafe {
		MmapMut::map_mut(&file)
	}.map_err(or_string)?;
	let file = &mut file[..];
	let ehdr: &Elf64_Ehdr = unsafe {
		&*file.as_ptr().cast()
	};
	assert!(&ehdr.e_ident[..4] == b"\x7fELF");

	let shdr: &[Elf64_Shdr] = unsafe {
		slice::from_raw_parts(
			file.as_ptr().add(size(ehdr.e_shoff)).cast(),
			ehdr.e_shnum.into(),
		)
	};
	let shdr_string: usize = ehdr.e_shstrndx.into();
	let shdr_string = &shdr[shdr_string];
	let string = &file[
		size(shdr_string.sh_offset)..size(shdr_string.sh_offset + shdr_string.sh_size)
	];
	let shdr_sym = shdr.into_iter().find(|section|
		section.sh_type == SHT_SYMTAB
	).ok_or("No symbol table!")?;
	let sym: &mut [Elf64_Sym] = unsafe {
		slice::from_raw_parts_mut(
			file.as_ptr().add(size(shdr_sym.sh_offset)) as *mut _,
			size(shdr_sym.sh_size) / size_of::<Elf64_Sym>(),
		)
	};
	for sym in sym {
		if sym.st_other & STV_HIDDEN != 0 {
			let name = &string[size(sym.st_name)..];
			let name = &name[..name.into_iter().position(|character|
				*character == b'\0'
			).unwrap() + 1];
			let name = CStr::from_bytes_with_nul(name).unwrap().to_string_lossy();
			if expr.is_match(&name) {
				println!("{}", name);
				sym.st_other = STV_DEFAULT;
			}
		}
	}

	Ok(())
}

fn filename(regex: &mut Cow<str>) -> Result<String, String> {
	let mut args = args();
	let prog = args.next().unwrap();
	let filename = args.next().ok_or(format!("USAGE: {} <filename> [regex]", prog));
	if let Some(expr) = args.next() {
		*regex = Cow::from(expr);
	}
	filename
}

fn or_string<T: ToString>(message: T) -> String {
	message.to_string()
}

fn size<T: TryInto<usize>>(size: T) -> usize
where <T as TryInto<usize>>::Error: Debug {
	size.try_into().unwrap()
}
