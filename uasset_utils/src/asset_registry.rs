use std::io::{Read, Seek, Write};

use anyhow::{anyhow, Context, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, BE, LE};
use unreal_asset::{exports::ExportBaseTrait, types::PackageIndex};
use unreal_asset::{flags::EObjectFlags, reader::ArchiveTrait};

use crate::paths::pak_path_to_game_path;

pub trait Readable<R> {
    fn read(reader: &mut R) -> Result<Self>
    where
        Self: Sized;
}
pub trait Writable<W> {
    fn write(&self, writer: &mut W) -> Result<()>;
}

type Guid = [u8; 16];
impl<R: Read> Readable<R> for Guid {
    fn read(reader: &mut R) -> Result<Self> {
        let mut buf = [0; 16];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}
impl<W: Write> Writable<W> for Guid {
    fn write(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self[..])?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NameIndex(u32);
impl<R: Read> Readable<R> for NameIndex {
    fn read(reader: &mut R) -> Result<Self> {
        Ok(NameIndex(reader.read_u32::<LE>()?))
    }
}
impl<W: Write> Writable<W> for NameIndex {
    fn write(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LE>(self.0)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NameIndexFlagged(pub u32, pub Option<u32>);
impl<R: Read> Readable<R> for NameIndexFlagged {
    fn read(reader: &mut R) -> Result<Self> {
        let n = reader.read_u32::<LE>()?;
        Ok(if n & 0x80000000 != 0 {
            let flag = Some(reader.read_u32::<LE>()?);
            NameIndexFlagged(n << 1 >> 1, flag)
        } else {
            NameIndexFlagged(n, None)
        })
    }
}
impl<W: Write> Writable<W> for NameIndexFlagged {
    fn write(&self, writer: &mut W) -> Result<()> {
        if let Some(flag) = self.1 {
            writer.write_u32::<LE>(self.0 | 0x80000000)?;
            writer.write_u32::<LE>(flag)?;
        } else {
            writer.write_u32::<LE>(self.0)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct ExportPath {
    pub object_path: NameIndexFlagged,
    pub package_path: NameIndexFlagged,
    pub asset_class: NameIndexFlagged,
}
impl<R: Read> Readable<R> for ExportPath {
    fn read(reader: &mut R) -> Result<Self> {
        Ok(ExportPath {
            object_path: NameIndexFlagged::read(reader)?,
            package_path: NameIndexFlagged::read(reader)?,
            asset_class: NameIndexFlagged::read(reader)?,
        })
    }
}
impl<W: Write> Writable<W> for ExportPath {
    fn write(&self, writer: &mut W) -> Result<()> {
        self.object_path.write(writer)?;
        self.package_path.write(writer)?;
        self.asset_class.write(writer)?;
        Ok(())
    }
}

fn read_array<R, T, E, F>(length: u32, reader: &mut R, mut f: F) -> Result<Vec<T>, E>
where
    F: FnMut(&mut R) -> Result<T, E>,
{
    (0..length).map(|_| f(reader)).collect()
}
fn write_array<W, T, F>(writer: &mut W, array: impl IntoIterator<Item = T>, mut f: F) -> Result<()>
where
    F: FnMut(&mut W, T) -> Result<()>,
{
    for item in array {
        f(writer, item)?;
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct Pair {
    pub name: NameIndex,
    pub type_: Type,
    pub index: u32,
}

impl<R: Read> Readable<R> for Pair {
    fn read(reader: &mut R) -> Result<Self> {
        let name = NameIndex::read(reader)?;
        let n = reader.read_u32::<LE>()?;
        let type_: Type = (n << 29 >> 29).try_into()?;
        let index = n >> 3;
        Ok(Pair { name, type_, index })
    }
}
impl<W: Write> Writable<W> for Pair {
    fn write(&self, writer: &mut W) -> Result<()> {
        self.name.write(writer)?;
        writer.write_u32::<LE>((self.type_ as u32) | self.index << 3)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct MapHandle {
    pub has_numberless_keys: bool,
    pub num: u16,
    pub pair_begin: u32,
}
impl<R: Read> Readable<R> for MapHandle {
    fn read(reader: &mut R) -> Result<Self> {
        let n = reader.read_u64::<LE>()?;
        Ok(Self {
            has_numberless_keys: (n >> 63) != 0,
            num: (n >> 32) as u16,
            pair_begin: n as u32,
        })
    }
}
impl<W: Write> Writable<W> for MapHandle {
    fn write(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LE>(
            (self.has_numberless_keys as u64) << 63
                | ((self.num as u64) << 32)
                | (self.pair_begin as u64),
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    AnsiString = 0,
    WideString = 1,
    NumberlessName = 2,
    Name = 3,
    NumberlessExportPath = 4,
    ExportPath = 5,
    LocalizedText = 6,
}
impl TryFrom<u32> for Type {
    type Error = anyhow::Error;

    fn try_from(v: u32) -> Result<Self> {
        match v {
            x if x == Type::AnsiString as u32 => Ok(Type::AnsiString),
            x if x == Type::WideString as u32 => Ok(Type::WideString),
            x if x == Type::NumberlessName as u32 => Ok(Type::NumberlessName),
            x if x == Type::Name as u32 => Ok(Type::Name),
            x if x == Type::NumberlessExportPath as u32 => Ok(Type::NumberlessExportPath),
            x if x == Type::ExportPath as u32 => Ok(Type::ExportPath),
            x if x == Type::LocalizedText as u32 => Ok(Type::LocalizedText),
            _ => Err(anyhow!("invalid AssetRegistry type: {v}")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AssetData {
    pub object_path: NameIndexFlagged,
    pub package_path: NameIndexFlagged,
    pub asset_class: NameIndexFlagged,
    pub package_name: NameIndexFlagged,
    pub asset_name: NameIndexFlagged,
    pub tags: MapHandle,
    pub bundle_count: u32,
    pub chunk_ids: Vec<u32>,
    pub flags: u32,
}
impl<R: Read> Readable<R> for AssetData {
    fn read(reader: &mut R) -> Result<Self> {
        Ok(AssetData {
            object_path: NameIndexFlagged::read(reader)?,
            package_path: NameIndexFlagged::read(reader)?,
            asset_class: NameIndexFlagged::read(reader)?,
            package_name: NameIndexFlagged::read(reader)?,
            asset_name: NameIndexFlagged::read(reader)?,
            tags: MapHandle::read(reader)?,
            bundle_count: reader.read_u32::<LE>()?,
            chunk_ids: read_array(reader.read_u32::<LE>()?, reader, R::read_u32::<LE>)?,
            flags: reader.read_u32::<LE>()?,
        })
    }
}
impl<W: Write> Writable<W> for AssetData {
    fn write(&self, writer: &mut W) -> Result<()> {
        self.object_path.write(writer)?;
        self.package_path.write(writer)?;
        self.asset_class.write(writer)?;
        self.package_name.write(writer)?;
        self.asset_name.write(writer)?;
        self.tags.write(writer)?;
        writer.write_u32::<LE>(self.bundle_count)?;
        writer.write_u32::<LE>(self.chunk_ids.len() as u32)?;
        for c in self.chunk_ids.iter() {
            writer.write_u32::<LE>(*c)?;
        }
        writer.write_u32::<LE>(self.flags)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Dependencies {
    pub dependencies_size: u64,
    pub dependencies: Vec<u32>,
    pub package_data_buffer_size: u32,
}
impl<R: Read> Readable<R> for Dependencies {
    fn read(reader: &mut R) -> Result<Self> {
        Ok(Dependencies {
            dependencies_size: reader.read_u64::<LE>()?,
            dependencies: read_array(reader.read_u32::<LE>()?, reader, R::read_u32::<LE>)?,
            package_data_buffer_size: reader.read_u32::<LE>()?,
        })
    }
}
impl<W: Write> Writable<W> for Dependencies {
    fn write(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LE>(self.dependencies_size)?;
        writer.write_u32::<LE>(self.dependencies.len() as u32)?;
        write_array(
            writer,
            &self.dependencies,
            |w, i| Ok(w.write_u32::<LE>(*i)?),
        )?;
        writer.write_u32::<LE>(self.package_data_buffer_size)?;
        Ok(())
    }
}

const MAGIC_START: u32 = 0x12345679;
const MAGIC_END: u32 = 0x87654321;

#[derive(Debug, PartialEq)]
pub struct Store {
    pub pair_count: u32,
    pub texts: Vec<String>,
    pub nbl_names: Vec<NameIndexFlagged>,
    pub names: Vec<NameIndexFlagged>,
    pub nbl_export_paths: Vec<ExportPath>,
    pub export_paths: Vec<ExportPath>,
    pub ansi_strings: Vec<String>,
    pub wide_strings: Vec<String>,
    pub pairs: Vec<Pair>,
}
impl<R: Read> Readable<R> for Store {
    fn read(reader: &mut R) -> Result<Self> {
        assert_eq!(MAGIC_START, reader.read_u32::<LE>()?);

        let nbl_names_count = reader.read_u32::<LE>()?;
        let names_count = reader.read_u32::<LE>()?;
        let nbl_export_path_count = reader.read_u32::<LE>()?;
        let export_path_count = reader.read_u32::<LE>()?;
        let texts_count = reader.read_u32::<LE>()?;
        let ansi_strings_count = reader.read_u32::<LE>()?;
        let wide_strings_count = reader.read_u32::<LE>()?;
        let _ansi_string_bytes = reader.read_u32::<LE>()?;
        let _wide_string_bytes = reader.read_u32::<LE>()?;
        let nbl_pair_count = reader.read_u32::<LE>()?;
        let pair_count = reader.read_u32::<LE>()?;

        let _text_bytes = reader.read_u32::<LE>()?;
        let texts = read_array(texts_count, reader, |r| -> Result<String> {
            let mut chars = vec![0; r.read_u32::<LE>()? as usize - 1];
            r.read_exact(&mut chars)?;
            r.read_u8()?;
            Ok(String::from_utf8_lossy(&chars).into_owned())
        })?;

        let nbl_names = read_array(nbl_names_count, reader, NameIndexFlagged::read)?;
        let names = read_array(names_count, reader, NameIndexFlagged::read)?;

        let nbl_export_paths = read_array(nbl_export_path_count, reader, ExportPath::read)?;
        let export_paths = read_array(export_path_count, reader, ExportPath::read)?;

        let _ansi_string_offsets = read_array(ansi_strings_count, reader, R::read_u32::<LE>)?;
        let _wide_string_offets = read_array(wide_strings_count, reader, R::read_u32::<LE>)?;

        let ansi_strings = read_array(ansi_strings_count, reader, |r| -> Result<String> {
            let mut chars = vec![];
            loop {
                let next = r.read_u8()?;
                if next == 0 {
                    break;
                }
                chars.push(next);
            }
            Ok(String::from_utf8_lossy(&chars).into_owned())
        })?;

        let wide_strings = read_array(wide_strings_count, reader, |reader| -> Result<String> {
            let mut chars = vec![];
            loop {
                let next = reader.read_u16::<LE>()?;
                if next == 0 {
                    break;
                }
                chars.push(char::from_u32(next.into()).unwrap_or(char::REPLACEMENT_CHARACTER));
            }
            Ok(chars.iter().collect::<String>())
        })?;

        let pairs = read_array(nbl_pair_count, reader, Pair::read)?;

        assert_eq!(reader.read_u32::<LE>()?, MAGIC_END);
        Ok(Self {
            pair_count,
            texts,
            nbl_names,
            names,
            nbl_export_paths,
            export_paths,
            ansi_strings,
            wide_strings,
            pairs,
        })
    }
}
impl<W: Write> Writable<W> for Store {
    fn write(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LE>(MAGIC_START)?;

        writer.write_u32::<LE>(self.nbl_names.len() as u32)?;
        writer.write_u32::<LE>(self.names.len() as u32)?;
        writer.write_u32::<LE>(self.nbl_export_paths.len() as u32)?;
        writer.write_u32::<LE>(self.export_paths.len() as u32)?;
        writer.write_u32::<LE>(self.texts.len() as u32)?;
        writer.write_u32::<LE>(self.ansi_strings.len() as u32)?;
        writer.write_u32::<LE>(self.wide_strings.len() as u32)?;

        writer.write_u32::<LE>(
            self.ansi_strings
                .iter()
                .map(|n| n.as_bytes().len() as u32 + 1)
                .sum(),
        )?;
        writer.write_u32::<LE>(
            self.wide_strings
                .iter()
                .map(|n| n.chars().count() as u32 + 1)
                .sum(),
        )?;

        writer.write_u32::<LE>(self.pairs.len() as u32)?;
        writer.write_u32::<LE>(self.pair_count)?;
        writer.write_u32::<LE>(
            self.texts
                .iter()
                .map(|n| n.as_bytes().len() as u32 + 1 + 4)
                .sum(),
        )?;

        write_array(writer, &self.texts, |w, i| {
            w.write_u32::<LE>(i.as_bytes().len() as u32 + 1)?;
            w.write_all(i.as_bytes())?;
            w.write_u8(0)?;
            Ok(())
        })?;

        write_array(writer, &self.nbl_names, |w, i| i.write(w))?;
        write_array(writer, &self.names, |w, i| i.write(w))?;
        write_array(writer, &self.nbl_export_paths, |w, i| i.write(w))?;
        write_array(writer, &self.export_paths, |w, i| i.write(w))?;

        let mut offset = 0;
        for i in &self.ansi_strings {
            writer.write_u32::<LE>(offset)?;
            offset += i.as_bytes().len() as u32 + 1;
        }

        let mut offset = 0;
        for i in &self.wide_strings {
            writer.write_u32::<LE>(offset)?;
            offset += i.chars().count() as u32 + 1;
        }

        write_array(writer, &self.ansi_strings, |w, i| {
            w.write_all(i.as_bytes())?;
            w.write_u8(0)?;
            Ok(())
        })?;

        write_array(writer, &self.wide_strings, |w, i| {
            for c in i.chars() {
                w.write_u16::<LE>(c as u16)?;
            }
            w.write_u16::<LE>(0)?;
            Ok(())
        })?;

        write_array(writer, &self.pairs, |w, i| i.write(w))?;

        writer.write_u32::<LE>(MAGIC_END)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Names(pub indexmap::IndexSet<String>);
impl std::ops::Index<NameIndexFlagged> for Names {
    type Output = String;
    fn index(&self, index: NameIndexFlagged) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}
impl std::ops::Index<NameIndex> for Names {
    type Output = String;
    fn index(&self, index: NameIndex) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

#[derive(Debug, PartialEq)]
pub struct AssetRegistry {
    pub version: Guid,
    pub version_int: u32,
    pub hash_version: u64,
    pub names: Names,
    pub store: Store,
    pub asset_data: Vec<AssetData>,
    pub dependencies: Dependencies,
}
impl<R: Read> Readable<R> for AssetRegistry {
    fn read(reader: &mut R) -> Result<Self> {
        let version = Guid::read(reader)?;

        let version_int = reader.read_u32::<LE>()?;
        let name_count = reader.read_u32::<LE>()?;
        let _num_string_bytes = reader.read_u32::<LE>()?;
        let hash_version = reader.read_u64::<LE>()?;

        let _lowercase_hashes = read_array(name_count, reader, R::read_u64::<LE>)?;
        let name_lengths = read_array(name_count, reader, R::read_u16::<BE>)?;

        let names = Names(
            name_lengths
                .into_iter()
                .map(|l| -> Result<String> {
                    let mut chars = vec![0; l as usize];
                    reader.read_exact(&mut chars)?;
                    Ok(String::from_utf8_lossy(&chars).into_owned())
                })
                .collect::<Result<_>>()?,
        );

        let store = Store::read(reader)?;

        let asset_data = read_array(reader.read_u32::<LE>()?, reader, AssetData::read)?;

        let dependencies = Dependencies::read(reader)?;
        Ok(AssetRegistry {
            version,
            version_int,
            hash_version,
            names,
            store,
            asset_data,
            dependencies,
        })
    }
}
impl<W: Write> Writable<W> for AssetRegistry {
    fn write(&self, writer: &mut W) -> Result<()> {
        self.version.write(writer)?;

        writer.write_u32::<LE>(self.version_int)?;
        writer.write_u32::<LE>(self.names.0.len() as u32)?;
        writer.write_u32::<LE>(self.names.0.iter().map(|n| n.bytes().len() as u32).sum())?;
        writer.write_u64::<LE>(self.hash_version)?;

        write_array(writer, &self.names.0, |w, i| {
            let hash = cityhasher::hash(i.to_ascii_lowercase().as_bytes());
            Ok(w.write_u64::<LE>(hash)?)
        })?;
        write_array(writer, &self.names.0, |w, i| {
            Ok(w.write_u16::<BE>(i.as_bytes().len() as u16)?)
        })?;

        write_array(writer, &self.names.0, |w, i| {
            w.write_all(i.as_bytes())?;
            Ok(())
        })?;

        self.store.write(writer)?;

        writer.write_u32::<LE>(self.asset_data.len() as u32)?;
        write_array(writer, &self.asset_data, |w, i| i.write(w))?;

        self.dependencies.write(writer)?;

        Ok(())
    }
}

pub mod dbg {
    use super::*;

    use std::fmt::{Debug, Formatter, Result};

    #[derive(PartialEq)]
    pub struct Dbg<'reg, 'data, D> {
        reg: &'reg AssetRegistry,
        data: &'data D,
    }
    impl<'reg, 'data, D> Dbg<'reg, 'data, D> {
        pub fn new(reg: &'reg AssetRegistry, data: &'data D) -> Self {
            Self { reg, data }
        }
    }

    impl Debug for Dbg<'_, '_, AssetData> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_struct("AssetData")
                .field("object_path", &self.reg.names[self.data.object_path])
                .field("package_path", &self.reg.names[self.data.package_path])
                .field("asset_class", &self.reg.names[self.data.asset_class])
                .field("package_name", &self.reg.names[self.data.package_name])
                .field("asset_name", &self.reg.names[self.data.asset_name])
                .field("tags", &Dbg::new(self.reg, &self.data.tags))
                .field("bundle_count", &self.data.bundle_count)
                .field("chunk_ids", &self.data.chunk_ids)
                .field("flags", &self.data.flags)
                .finish()
        }
    }
    impl Debug for Dbg<'_, '_, ExportPath> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_struct("Asset")
                .field("object_path", &self.reg.names[self.data.object_path])
                .field("package_path", &self.reg.names[self.data.package_path])
                .field("asset_class", &self.reg.names[self.data.asset_class])
                .finish()
        }
    }
    impl Debug for Dbg<'_, '_, MapHandle> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            let mut dbg = f.debug_list();
            let start = self.data.pair_begin as usize;
            let end = start + self.data.num as usize;
            for i in start..end {
                dbg.entry(&Dbg::new(self.reg, &self.reg.store.pairs[i]));
            }
            dbg.finish()
        }
    }
    impl Debug for Dbg<'_, '_, Pair> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            let mut dbg = f.debug_struct("Pair");
            dbg.field("name", &self.reg.names[self.data.name]);
            let s = &self.reg.store;
            let i = self.data.index as usize;
            match self.data.type_ {
                Type::AnsiString => {
                    dbg.field("value", &s.ansi_strings[i]);
                }
                Type::WideString => {
                    dbg.field("value", &s.wide_strings[i]);
                }
                Type::NumberlessName => {
                    dbg.field("value", &self.reg.names[s.nbl_names[i]]);
                }
                Type::Name => {
                    dbg.field("value", &self.reg.names[s.names[i]]);
                }
                Type::NumberlessExportPath => {
                    dbg.field("value", &Dbg::new(self.reg, &s.nbl_export_paths[i]));
                }
                Type::ExportPath => {
                    dbg.field("value", &Dbg::new(self.reg, &s.export_paths[i]));
                }
                Type::LocalizedText => {
                    dbg.field("value", &s.texts[i]);
                }
            }
            dbg.finish()
        }
    }
}

pub fn get_root_export<C: Read + Seek>(
    asset: &unreal_asset::asset::Asset<C>,
) -> Option<PackageIndex> {
    for (i, e) in asset.asset_data.exports.iter().enumerate() {
        let base = e.get_base_export();
        if base.outer_index.index == 0 && base.object_flags.contains(EObjectFlags::RF_PUBLIC) {
            return Some(PackageIndex::from_export(i as i32).unwrap());
        }
    }
    None
}

impl AssetRegistry {
    pub fn get_name(&mut self, name: &str) -> NameIndexFlagged {
        if let Some(i) = self.names.0.get_index_of(name) {
            NameIndexFlagged(i as u32, None)
        } else {
            self.names.0.insert(name.to_string());
            NameIndexFlagged(self.names.0.len() as u32 - 1, None)
        }
    }
    pub fn populate<C: Read + Seek>(
        &mut self,
        path: &str,
        asset: &unreal_asset::Asset<C>,
    ) -> Result<()> {
        let game_path = crate::paths::PakPathBuf::from(
            pak_path_to_game_path(path).context("failed to get game path")?,
        );

        let root = get_root_export(asset).context("no root export")?;
        let root = asset.get_export(root).unwrap();

        let asset_name_str = root.get_base_export().object_name.get_owned_content();
        let package_path_str = game_path.parent().context("no path parent")?.as_str();
        let package_name_str = game_path.as_str();
        let object_path_str = format!("{game_path}.{asset_name_str}");
        let asset_class_str = asset
            .get_import(root.get_base_export().class_index)
            .context("bad import ref")?
            .object_name
            .get_owned_content();

        // skip existing
        if self
            .asset_data
            .iter()
            .find(|a| self.names[a.object_path] == object_path_str)
            .is_some()
        {
            return Ok(());
        }

        let object_path = self.get_name(&object_path_str);
        let package_path = self.get_name(package_path_str);
        let asset_class = self.get_name(&asset_class_str);
        let package_name = self.get_name(package_name_str);
        let asset_name = self.get_name(&asset_name_str);

        let new = AssetData {
            object_path,
            package_path,
            asset_class,
            package_name,
            asset_name,
            tags: MapHandle {
                has_numberless_keys: true,
                num: 0,
                pair_begin: 0,
            },
            bundle_count: 0,
            chunk_ids: vec![],
            flags: 0,
        };
        self.asset_data.push(new);

        if let (Some(asset_name_str), Some(object_path_str), Some(asset_class_str)) = (
            asset_name_str.strip_suffix("_C"),
            object_path_str.strip_suffix("_C"),
            asset_class_str.strip_suffix("GeneratedClass"),
        ) {
            let new = AssetData {
                object_path: self.get_name(object_path_str),
                package_path,
                asset_class: self.get_name(asset_class_str),
                package_name,
                asset_name: self.get_name(asset_name_str),
                tags: MapHandle {
                    has_numberless_keys: true,
                    num: 0,
                    pair_begin: 0,
                },
                bundle_count: 0,
                chunk_ids: vec![],
                flags: 0,
            };
            self.asset_data.push(new);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn hash() {
        assert_eq!(cityhasher::hash::<u64>(b"Timestamp"), 0x62701ea6363a9b97);
    }

    /*
    use super::*;

    fn dump_ar(ar: &AssetRegistry) -> String {
        use std::fmt::Write;

        let mut out = String::new();
        let mut asset_data: Vec<_> = ar.asset_data.iter().collect();
        asset_data.sort_by_key(|a| &ar.names[a.object_path]);

        for asset in &asset_data {
            let dbg = dbg::Dbg::new(&ar, *asset);
            writeln!(&mut out, "{:#?}", dbg).unwrap();
        }
        out
    }

    #[test]
    fn read_test_pak() {
        use std::io::*;

        let mut reader =
            BufReader::new(std::fs::File::open("../MinimalUProject2-LinuxNoEditor.pak").unwrap());
        let pak = repak::PakBuilder::new().reader(&mut reader).unwrap();
        let mut ar = AssetRegistry::read(&mut Cursor::new(
            pak.get("MinimalUProject2/AssetRegistry.bin", &mut reader)
                .unwrap(),
        ))
        .unwrap();
        std::fs::write("../input.txt", dump_ar(&ar)).unwrap();

        ar.asset_data.clear();
        ar.names.0.clear();

        for file in pak.files() {
            let path = crate::paths::PakPath::new(&file);
            match path.extension() {
                Some("uasset" | "umap") => {}
                _ => continue,
            };
            let uasset = Cursor::new(pak.get(path.as_str(), &mut reader).unwrap());
            let uexp = Cursor::new(
                pak.get(path.with_extension("uexp").as_str(), &mut reader)
                    .unwrap(),
            );
            let asset = unreal_asset::AssetBuilder::new(
                uasset,
                unreal_asset::engine_version::EngineVersion::VER_UE4_27,
            )
            .bulk(uexp)
            .skip_data(true)
            .build()
            .unwrap();
            ar.populate(path.with_extension("").as_str(), &asset)
                .unwrap();
        }
        std::fs::write("../output.txt", dump_ar(&ar)).unwrap();
    }
    */
}
