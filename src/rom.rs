//! Red-label ROM metadata and optional local ROM reporting.
//!
//! The metadata is embedded from `assets/red-label/roms.tsv`; local ROM files
//! are only used by verification/reporting workflows.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomDescriptor {
    pub name: &'static str,
    pub size: u64,
    pub crc32: &'static str,
}

pub fn red_label_roms() -> &'static [RomDescriptor] {
    static ROMS: OnceLock<Vec<RomDescriptor>> = OnceLock::new();
    ROMS.get_or_init(|| {
        parse_rom_descriptors(crate::assets::RED_LABEL_ROMS_TSV)
            .expect("embedded red-label ROM metadata should parse")
    })
    .as_slice()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomReport {
    pub directory: PathBuf,
    pub expected: usize,
    pub found: Vec<String>,
    pub missing: Vec<String>,
    pub unexpected: Vec<String>,
    pub wrong_size: Vec<String>,
    pub wrong_crc: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRomFile {
    pub descriptor: RomDescriptor,
    pub crc32: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRomSet {
    files: Vec<VerifiedRomFile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomRegion {
    pub name: &'static str,
    pub size: usize,
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomView {
    Fixed,
    Banked(u8),
    SoundCpu,
    Prom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomLoad {
    pub name: &'static str,
    pub region: &'static str,
    pub region_offset: usize,
    pub size: usize,
    pub view: RomView,
    pub cpu_start: Option<u16>,
}

impl RomLoad {
    pub fn cpu_end(self) -> Option<u16> {
        let start = self.cpu_start?;
        let width = u16::try_from(self.size.saturating_sub(1)).ok()?;

        Some(start.saturating_add(width))
    }

    fn contains_cpu_address(self, view: RomView, address: u16) -> bool {
        let Some(start) = self.cpu_start else {
            return false;
        };
        let Some(end) = self.cpu_end() else {
            return false;
        };

        self.view == view && (start..=end).contains(&address)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomImageRegion {
    pub region: RomRegion,
    bytes: Vec<Option<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelRomImages {
    regions: Vec<RomImageRegion>,
    loads: Vec<RomLoad>,
}

pub fn red_label_rom_regions() -> &'static [RomRegion] {
    static REGIONS: OnceLock<Vec<RomRegion>> = OnceLock::new();
    REGIONS
        .get_or_init(|| {
            parse_rom_regions(crate::assets::RED_LABEL_ROM_REGIONS_TSV)
                .expect("embedded red-label ROM regions should parse")
        })
        .as_slice()
}

pub fn red_label_main_cpu_region() -> &'static RomRegion {
    red_label_rom_regions()
        .iter()
        .find(|region| region.name == "maincpu")
        .expect("embedded red-label ROM regions should include maincpu")
}

pub fn red_label_rom_map() -> &'static [RomLoad] {
    static LOADS: OnceLock<Vec<RomLoad>> = OnceLock::new();
    LOADS
        .get_or_init(|| {
            parse_rom_map(crate::assets::RED_LABEL_ROM_MAP_TSV)
                .expect("embedded red-label ROM map should parse")
        })
        .as_slice()
}

pub const RED_LABEL_CROM0_ROMMAP_BYTES: usize = 24;
pub const RED_LABEL_CROM0_ROMMAP_SLOT_BYTES: usize = 2;
pub const RED_LABEL_CROM0_ROM_CHUNK_SIZE: usize = 0x0800;
pub const RED_LABEL_CROM0_FIXED_ROM_BLOCK: u8 = 0x0F;
const RED_LABEL_CROM0_ROMMAP_ADDRESS_MASK: u8 = 0x70;
const RED_LABEL_CROM0_ROMMAP_BLOCK_MASK: u8 = 0x0F;
const RED_LABEL_CROM0_ROMMAP_UNUSED_MASK: u8 = 0x80;

pub fn red_label_crom0_rom_map_descriptors() -> Result<[u8; RED_LABEL_CROM0_ROMMAP_BYTES], String> {
    red_label_crom0_rom_map_descriptors_from_loads(red_label_rom_map())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelCrom0RomCheck {
    pub descriptor_index: usize,
    pub rom_number: u8,
    pub half: u8,
    pub descriptor: u8,
    pub view: RomView,
    pub start_address: u16,
    pub checksum: u16,
}

impl RedLabelCrom0RomCheck {
    pub fn failed(self) -> bool {
        self.checksum != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0RomDiagnosticReport {
    checks: Vec<RedLabelCrom0RomCheck>,
    failures: Vec<RedLabelCrom0RomCheck>,
    reported_bad_roms: Vec<u8>,
}

impl RedLabelCrom0RomDiagnosticReport {
    pub fn checks(&self) -> &[RedLabelCrom0RomCheck] {
        &self.checks
    }

    pub fn failures(&self) -> &[RedLabelCrom0RomCheck] {
        &self.failures
    }

    pub fn reported_bad_roms(&self) -> &[u8] {
        &self.reported_bad_roms
    }

    pub fn last_reported_bad_rom(&self) -> Option<u8> {
        self.reported_bad_roms.last().copied()
    }

    pub fn all_roms_ok(&self) -> bool {
        self.failures.is_empty()
    }
}

pub fn red_label_crom0_rom_diagnostics(
    images: &RedLabelRomImages,
) -> Result<RedLabelCrom0RomDiagnosticReport, String> {
    red_label_crom0_rom_diagnostics_from_descriptors(
        images,
        &red_label_crom0_rom_map_descriptors()?,
    )
}

impl VerifiedRomSet {
    pub fn files(&self) -> &[VerifiedRomFile] {
        &self.files
    }

    pub fn file_bytes(&self, name: &str) -> Option<&[u8]> {
        self.files
            .iter()
            .find(|file| file.descriptor.name == name)
            .map(|file| file.bytes.as_slice())
    }

    pub fn total_bytes(&self) -> usize {
        self.files.iter().map(|file| file.bytes.len()).sum()
    }

    #[cfg(test)]
    pub(crate) fn from_files_for_test(files: Vec<VerifiedRomFile>) -> Self {
        Self { files }
    }
}

impl RedLabelRomImages {
    pub fn from_verified_rom_set(rom_set: &VerifiedRomSet) -> Result<Self, String> {
        Self::from_parts(rom_set, red_label_rom_regions(), red_label_rom_map())
    }

    #[cfg(test)]
    pub(crate) fn from_parts_for_test(
        rom_set: &VerifiedRomSet,
        regions: &[RomRegion],
        loads: &[RomLoad],
    ) -> Result<Self, String> {
        Self::from_parts(rom_set, regions, loads)
    }

    pub fn regions(&self) -> &[RomImageRegion] {
        &self.regions
    }

    pub fn loads(&self) -> &[RomLoad] {
        &self.loads
    }

    pub fn region(&self, name: &str) -> Option<RomRegion> {
        self.region_image(name).map(|image| image.region)
    }

    pub fn byte_at_region_offset(&self, region: &str, offset: usize) -> Option<u8> {
        self.region_image(region)?
            .bytes
            .get(offset)
            .copied()
            .flatten()
    }

    pub fn fixed_byte(&self, address: u16) -> Option<u8> {
        self.cpu_byte(RomView::Fixed, address)
    }

    pub fn banked_byte(&self, bank: u8, address: u16) -> Option<u8> {
        self.cpu_byte(RomView::Banked(bank), address)
    }

    pub fn sound_cpu_byte(&self, address: u16) -> Option<u8> {
        self.cpu_byte(RomView::SoundCpu, address)
    }

    pub fn prom_byte(&self, offset: usize) -> Option<u8> {
        self.byte_at_region_offset("proms", offset)
    }

    fn from_parts(
        rom_set: &VerifiedRomSet,
        regions: &[RomRegion],
        loads: &[RomLoad],
    ) -> Result<Self, String> {
        let mut images = regions
            .iter()
            .copied()
            .map(|region| RomImageRegion {
                region,
                bytes: vec![None; region.size],
            })
            .collect::<Vec<_>>();

        for load in loads.iter().copied() {
            let Some(image) = images
                .iter_mut()
                .find(|image| image.region.name == load.region)
            else {
                return Err(format!(
                    "mapped file {} targets missing region {}",
                    load.name, load.region
                ));
            };
            let Some(file_bytes) = rom_set.file_bytes(load.name) else {
                return Err(format!(
                    "verified ROM set is missing mapped file {}",
                    load.name
                ));
            };
            if file_bytes.len() != load.size {
                return Err(format!(
                    "mapped file {} expected {} bytes got {}",
                    load.name,
                    load.size,
                    file_bytes.len()
                ));
            }

            let end = load
                .region_offset
                .checked_add(load.size)
                .ok_or_else(|| format!("mapped file {} offset overflows", load.name))?;
            if end > image.bytes.len() {
                return Err(format!(
                    "mapped file {} ends at 0x{end:05x} beyond {} region size 0x{:05x}",
                    load.name, image.region.name, image.region.size
                ));
            }

            for (index, byte) in file_bytes.iter().copied().enumerate() {
                let offset = load.region_offset + index;
                if image.bytes[offset].replace(byte).is_some() {
                    return Err(format!(
                        "mapped file {} overlaps an existing byte at {}:0x{offset:05x}",
                        load.name, image.region.name
                    ));
                }
            }
        }

        Ok(Self {
            regions: images,
            loads: loads.to_vec(),
        })
    }

    fn cpu_byte(&self, view: RomView, address: u16) -> Option<u8> {
        let load = self
            .loads
            .iter()
            .copied()
            .find(|load| load.contains_cpu_address(view, address))?;
        let start = load.cpu_start?;
        let offset = load.region_offset + usize::from(address - start);

        self.byte_at_region_offset(load.region, offset)
    }

    fn region_image(&self, name: &str) -> Option<&RomImageRegion> {
        self.regions.iter().find(|image| image.region.name == name)
    }
}

impl RomReport {
    pub fn expected_count(&self) -> usize {
        self.expected
    }

    pub fn found_count(&self) -> usize {
        self.found.len()
    }

    pub fn verified_count(&self) -> usize {
        self.found_count().saturating_sub(self.wrong_crc.len())
    }

    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.wrong_size.is_empty() && self.wrong_crc.is_empty()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "ROM set {}: {}/{} expected files present, {}/{} CRCs verified",
            self.directory.display(),
            self.found_count(),
            self.expected_count(),
            self.verified_count(),
            self.found_count()
        )
    }
}

pub fn scan_dir(path: &Path) -> io::Result<RomReport> {
    scan_dir_against(path, red_label_roms())
}

pub fn load_verified_dir(path: &Path) -> io::Result<Result<VerifiedRomSet, RomReport>> {
    load_verified_dir_against(path, red_label_roms())
}

fn scan_dir_against(path: &Path, descriptors: &[RomDescriptor]) -> io::Result<RomReport> {
    let mut found = BTreeSet::new();
    let mut unexpected = BTreeSet::new();
    let mut wrong_size = BTreeSet::new();
    let mut wrong_crc = BTreeSet::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(descriptor) = descriptors
            .iter()
            .find(|descriptor| descriptor.name == name)
        {
            found.insert(name.clone());
            let metadata = entry.metadata()?;
            if metadata.len() != descriptor.size {
                wrong_size.insert(format!(
                    "{} expected {} bytes got {}",
                    descriptor.name,
                    descriptor.size,
                    metadata.len()
                ));
            }

            let actual_crc = crc32_file(&entry.path())?;
            let actual_crc_hex = format!("{actual_crc:08x}");
            if actual_crc_hex != descriptor.crc32 {
                wrong_crc.insert(format!(
                    "{} expected {} got {}",
                    descriptor.name, descriptor.crc32, actual_crc_hex
                ));
            }
        } else {
            unexpected.insert(name);
        }
    }

    let found_vec = descriptors
        .iter()
        .filter(|descriptor| found.contains(descriptor.name))
        .map(|descriptor| descriptor.name.to_string())
        .collect::<Vec<_>>();

    let missing = descriptors
        .iter()
        .filter(|descriptor| !found.contains(descriptor.name))
        .map(|descriptor| descriptor.name.to_string())
        .collect::<Vec<_>>();

    Ok(RomReport {
        directory: path.to_path_buf(),
        expected: descriptors.len(),
        found: found_vec,
        missing,
        unexpected: unexpected.into_iter().collect(),
        wrong_size: wrong_size.into_iter().collect(),
        wrong_crc: wrong_crc.into_iter().collect(),
    })
}

fn load_verified_dir_against(
    path: &Path,
    descriptors: &[RomDescriptor],
) -> io::Result<Result<VerifiedRomSet, RomReport>> {
    let report = scan_dir_against(path, descriptors)?;
    if !report.is_complete() {
        return Ok(Err(report));
    }

    let mut files = Vec::with_capacity(descriptors.len());
    for descriptor in descriptors {
        let bytes = fs::read(path.join(descriptor.name))?;
        let crc32 = crc32(&bytes);
        files.push(VerifiedRomFile {
            descriptor: *descriptor,
            crc32,
            bytes,
        });
    }

    Ok(Ok(VerifiedRomSet { files }))
}

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn crc32_file(path: &Path) -> io::Result<u32> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut crc = 0xFFFF_FFFFu32;
    let mut buffer = [0u8; 8 * 1024];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        crc = crc32_update(crc, &buffer[..read]);
    }

    Ok(!crc)
}

fn crc32_update(mut crc: u32, bytes: &[u8]) -> u32 {
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    crc
}

fn parse_rom_descriptors(text: &'static str) -> Result<Vec<RomDescriptor>, String> {
    let mut lines = text.lines().enumerate();
    let Some((_, header)) = lines.next() else {
        return Err(String::from("ROM metadata asset is empty"));
    };
    if header != "name\tsize\tcrc32" {
        return Err(format!("unexpected ROM metadata header: {header}"));
    }

    let mut descriptors = Vec::new();
    for (line_index, line) in lines {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err(format!(
                "ROM metadata line {line_number} has wrong field count"
            ));
        }

        let size = fields[1].parse::<u64>().map_err(|error| {
            format!("ROM metadata line {line_number} has invalid size: {error}")
        })?;
        if !is_crc32_hex(fields[2]) {
            return Err(format!(
                "ROM metadata line {line_number} has invalid CRC-32: {}",
                fields[2]
            ));
        }

        descriptors.push(RomDescriptor {
            name: fields[0],
            size,
            crc32: fields[2],
        });
    }

    if descriptors.is_empty() {
        return Err(String::from("ROM metadata asset has no ROM descriptors"));
    }

    Ok(descriptors)
}

fn is_crc32_hex(value: &str) -> bool {
    value.len() == 8 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn parse_rom_regions(text: &'static str) -> Result<Vec<RomRegion>, String> {
    let mut lines = text.lines().enumerate();
    let Some((_, header)) = lines.next() else {
        return Err(String::from("ROM region asset is empty"));
    };
    if header != "region\tsize\tsource" {
        return Err(format!("unexpected ROM region header: {header}"));
    }

    let mut regions = Vec::new();
    for (line_index, line) in lines {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err(format!(
                "ROM region line {line_number} has wrong field count"
            ));
        }

        regions.push(RomRegion {
            name: fields[0],
            size: parse_usize_number(fields[1], line_number, "size")?,
            source: fields[2],
        });
    }

    if regions.is_empty() {
        return Err(String::from("ROM region asset has no regions"));
    }

    Ok(regions)
}

fn parse_rom_map(text: &'static str) -> Result<Vec<RomLoad>, String> {
    let mut lines = text.lines().enumerate();
    let Some((_, header)) = lines.next() else {
        return Err(String::from("ROM map asset is empty"));
    };
    if header != "name\tregion\tregion_offset\tsize\tview\tbank\tcpu_start" {
        return Err(format!("unexpected ROM map header: {header}"));
    }

    let mut loads = Vec::new();
    for (line_index, line) in lines {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() != 7 {
            return Err(format!("ROM map line {line_number} has wrong field count"));
        }

        let view = parse_rom_view(fields[4], fields[5], line_number)?;
        let cpu_start = parse_optional_u16_number(fields[6], line_number, "cpu_start")?;
        validate_rom_cpu_start(view, cpu_start, line_number)?;

        loads.push(RomLoad {
            name: fields[0],
            region: fields[1],
            region_offset: parse_usize_number(fields[2], line_number, "region_offset")?,
            size: parse_usize_number(fields[3], line_number, "size")?,
            view,
            cpu_start,
        });
    }

    if loads.is_empty() {
        return Err(String::from("ROM map asset has no loads"));
    }

    Ok(loads)
}

fn red_label_crom0_rom_map_descriptors_from_loads(
    loads: &[RomLoad],
) -> Result<[u8; RED_LABEL_CROM0_ROMMAP_BYTES], String> {
    let mut descriptors = [0; RED_LABEL_CROM0_ROMMAP_BYTES];
    for load in loads.iter().copied() {
        let Some(rom_number) = red_label_defender_rom_number(load.name)? else {
            continue;
        };
        let slot_start = usize::from(rom_number - 1) * RED_LABEL_CROM0_ROMMAP_SLOT_BYTES;
        if slot_start + RED_LABEL_CROM0_ROMMAP_SLOT_BYTES > descriptors.len() {
            return Err(format!(
                "CROM0 ROMMAP has no slot for physical ROM {}",
                load.name
            ));
        }
        if load.size % RED_LABEL_CROM0_ROM_CHUNK_SIZE != 0 {
            return Err(format!(
                "CROM0 ROMMAP load {} size 0x{:04X} is not 2K aligned",
                load.name, load.size
            ));
        }

        let chunks = load.size / RED_LABEL_CROM0_ROM_CHUNK_SIZE;
        if chunks > RED_LABEL_CROM0_ROMMAP_SLOT_BYTES {
            return Err(format!(
                "CROM0 ROMMAP load {} spans {chunks} chunks, expected at most two",
                load.name
            ));
        }

        for chunk_index in 0..chunks {
            let descriptor = red_label_crom0_descriptor_byte(load, chunk_index)?;
            let descriptor_index = slot_start + chunk_index;
            if descriptors[descriptor_index] != 0 {
                return Err(format!(
                    "CROM0 ROMMAP slot {} half {} is already occupied",
                    rom_number,
                    chunk_index + 1
                ));
            }
            descriptors[descriptor_index] = descriptor;
        }
    }

    Ok(descriptors)
}

fn red_label_crom0_rom_diagnostics_from_descriptors(
    images: &RedLabelRomImages,
    descriptors: &[u8; RED_LABEL_CROM0_ROMMAP_BYTES],
) -> Result<RedLabelCrom0RomDiagnosticReport, String> {
    let mut failed_by_index = [false; RED_LABEL_CROM0_ROMMAP_BYTES];
    let mut checks = Vec::new();
    let mut failures = Vec::new();

    for (descriptor_index, descriptor) in descriptors.iter().copied().enumerate() {
        if descriptor == 0 {
            continue;
        }

        let check = red_label_crom0_rom_check(images, descriptor_index, descriptor)?;
        if check.failed() {
            failed_by_index[descriptor_index] = true;
            failures.push(check);
        }
        checks.push(check);
    }

    Ok(RedLabelCrom0RomDiagnosticReport {
        checks,
        failures,
        reported_bad_roms: red_label_crom0_reported_bad_roms(&failed_by_index),
    })
}

fn red_label_crom0_rom_check(
    images: &RedLabelRomImages,
    descriptor_index: usize,
    descriptor: u8,
) -> Result<RedLabelCrom0RomCheck, String> {
    let view = red_label_crom0_descriptor_view(descriptor);
    let start_address = red_label_crom0_descriptor_start_address(descriptor)?;
    let checksum = red_label_crom0_rom_checksum(images, view, start_address, descriptor)?;

    Ok(RedLabelCrom0RomCheck {
        descriptor_index,
        rom_number: red_label_crom0_physical_rom_number(descriptor_index),
        half: red_label_crom0_physical_rom_half(descriptor_index),
        descriptor,
        view,
        start_address,
        checksum,
    })
}

fn red_label_crom0_reported_bad_roms(
    failed_by_index: &[bool; RED_LABEL_CROM0_ROMMAP_BYTES],
) -> Vec<u8> {
    let mut reported = Vec::new();
    let mut descriptor_index = 0;
    while descriptor_index < failed_by_index.len() {
        if failed_by_index[descriptor_index] {
            reported.push(red_label_crom0_physical_rom_number(descriptor_index));
            descriptor_index += if descriptor_index % RED_LABEL_CROM0_ROMMAP_SLOT_BYTES == 0 {
                RED_LABEL_CROM0_ROMMAP_SLOT_BYTES
            } else {
                1
            };
        } else {
            descriptor_index += 1;
        }
    }

    reported
}

fn red_label_crom0_descriptor_view(descriptor: u8) -> RomView {
    let block = descriptor & RED_LABEL_CROM0_ROMMAP_BLOCK_MASK;
    if block == RED_LABEL_CROM0_FIXED_ROM_BLOCK {
        RomView::Fixed
    } else {
        RomView::Banked(block)
    }
}

fn red_label_crom0_descriptor_start_address(descriptor: u8) -> Result<u16, String> {
    if descriptor & RED_LABEL_CROM0_ROMMAP_UNUSED_MASK != 0 {
        return Err(format!(
            "CROM0 ROMMAP descriptor 0x{descriptor:02X} sets unsupported bit 7"
        ));
    }

    let address_code = u16::from((descriptor & RED_LABEL_CROM0_ROMMAP_ADDRESS_MASK) >> 4);
    Ok(0xC000 + address_code * RED_LABEL_CROM0_ROM_CHUNK_SIZE as u16)
}

fn red_label_crom0_rom_checksum(
    images: &RedLabelRomImages,
    view: RomView,
    start_address: u16,
    descriptor: u8,
) -> Result<u16, String> {
    let mut checksum = 0u16;
    let mut address = start_address;
    loop {
        let high = red_label_crom0_rom_checksum_byte(images, view, address, descriptor)?;
        let low =
            red_label_crom0_rom_checksum_byte(images, view, address.wrapping_add(1), descriptor)?;
        checksum = add_end_around_carry(checksum, u16::from_be_bytes([high, low]));

        if address == 0xFFFE {
            break;
        }
        address = address
            .checked_add(2)
            .ok_or_else(|| format!("CROM0 ROM checksum address {address:#06X} overflows"))?;
    }

    Ok(checksum)
}

fn red_label_crom0_rom_checksum_byte(
    images: &RedLabelRomImages,
    view: RomView,
    address: u16,
    descriptor: u8,
) -> Result<u8, String> {
    let byte = if address < 0xD000 {
        match view {
            RomView::Banked(bank) => images.banked_byte(bank, address).or_else(|| {
                if (0xC800..0xD000).contains(&address) {
                    images.banked_byte(bank, address - RED_LABEL_CROM0_ROM_CHUNK_SIZE as u16)
                } else {
                    None
                }
            }),
            RomView::Fixed => images.fixed_byte(address),
            RomView::SoundCpu | RomView::Prom => None,
        }
    } else {
        images.fixed_byte(address)
    };

    byte.ok_or_else(|| {
        format!("CROM0 ROM checksum descriptor 0x{descriptor:02X} cannot read {address:#06X}")
    })
}

fn add_end_around_carry(left: u16, right: u16) -> u16 {
    let sum = u32::from(left) + u32::from(right);
    (sum as u16).wrapping_add((sum >> 16) as u16)
}

fn red_label_crom0_physical_rom_number(descriptor_index: usize) -> u8 {
    (descriptor_index / RED_LABEL_CROM0_ROMMAP_SLOT_BYTES + 1) as u8
}

fn red_label_crom0_physical_rom_half(descriptor_index: usize) -> u8 {
    (descriptor_index % RED_LABEL_CROM0_ROMMAP_SLOT_BYTES + 1) as u8
}

fn red_label_defender_rom_number(name: &str) -> Result<Option<u8>, String> {
    let Some(number) = name
        .strip_prefix("defend.")
        .and_then(|suffix| suffix.parse::<u8>().ok())
    else {
        return Ok(None);
    };
    if number == 0 || number > 12 {
        return Err(format!(
            "CROM0 ROMMAP physical ROM number {number} is out of range"
        ));
    }
    Ok(Some(number))
}

fn red_label_crom0_descriptor_byte(load: RomLoad, chunk_index: usize) -> Result<u8, String> {
    let cpu_start = load
        .cpu_start
        .ok_or_else(|| format!("CROM0 ROMMAP load {} has no CPU start", load.name))?;
    let chunk_offset = u16::try_from(chunk_index * RED_LABEL_CROM0_ROM_CHUNK_SIZE)
        .map_err(|_| format!("CROM0 ROMMAP load {} chunk offset overflows", load.name))?;
    let address = cpu_start
        .checked_add(chunk_offset)
        .ok_or_else(|| format!("CROM0 ROMMAP load {} CPU address overflows", load.name))?;
    if !(0xC000..=0xF800).contains(&address) || (address - 0xC000) % 0x0800 != 0 {
        return Err(format!(
            "CROM0 ROMMAP load {} CPU address {address:#06X} is not a 2K ROM boundary",
            load.name
        ));
    }

    let address_code = ((address - 0xC000) / 0x0800) as u8;
    let block = match load.view {
        RomView::Fixed => RED_LABEL_CROM0_FIXED_ROM_BLOCK,
        RomView::Banked(bank) if bank <= 0x0F => bank,
        RomView::Banked(bank) => {
            return Err(format!(
                "CROM0 ROMMAP load {} bank {bank} exceeds descriptor width",
                load.name
            ));
        }
        RomView::SoundCpu | RomView::Prom => {
            return Err(format!(
                "CROM0 ROMMAP load {} is not a main CPU ROM",
                load.name
            ));
        }
    };

    Ok((address_code << 4) | block)
}

fn parse_rom_view(view: &str, bank: &str, line_number: usize) -> Result<RomView, String> {
    match view {
        "fixed" if bank == "-" => Ok(RomView::Fixed),
        "fixed" => Err(format!(
            "fixed ROM map line {line_number} must use bank '-'"
        )),
        "banked" => Ok(RomView::Banked(parse_u8_number(bank, line_number, "bank")?)),
        "sound" if bank == "-" => Ok(RomView::SoundCpu),
        "sound" => Err(format!(
            "sound ROM map line {line_number} must use bank '-'"
        )),
        "prom" if bank == "-" => Ok(RomView::Prom),
        "prom" => Err(format!("prom ROM map line {line_number} must use bank '-'")),
        other => Err(format!(
            "unknown ROM map view {other} on line {line_number}"
        )),
    }
}

fn validate_rom_cpu_start(
    view: RomView,
    cpu_start: Option<u16>,
    line_number: usize,
) -> Result<(), String> {
    match (view, cpu_start) {
        (RomView::Prom, None) => Ok(()),
        (RomView::Prom, Some(_)) => Err(format!(
            "prom ROM map line {line_number} must use cpu_start '-'"
        )),
        (_, Some(_)) => Ok(()),
        (_, None) => Err(format!(
            "{} ROM map line {line_number} must include cpu_start",
            rom_view_label(view)
        )),
    }
}

fn rom_view_label(view: RomView) -> &'static str {
    match view {
        RomView::Fixed => "fixed",
        RomView::Banked(_) => "banked",
        RomView::SoundCpu => "sound",
        RomView::Prom => "prom",
    }
}

fn parse_u8_number(value: &str, line_number: usize, field: &str) -> Result<u8, String> {
    parse_u32_number(value, line_number, field).and_then(|number| {
        u8::try_from(number)
            .map_err(|error| format!("{field} on line {line_number} is not a u8: {error}"))
    })
}

fn parse_u16_number(value: &str, line_number: usize, field: &str) -> Result<u16, String> {
    parse_u32_number(value, line_number, field).and_then(|number| {
        u16::try_from(number)
            .map_err(|error| format!("{field} on line {line_number} is not a u16: {error}"))
    })
}

fn parse_optional_u16_number(
    value: &str,
    line_number: usize,
    field: &str,
) -> Result<Option<u16>, String> {
    if value == "-" {
        return Ok(None);
    }

    parse_u16_number(value, line_number, field).map(Some)
}

fn parse_usize_number(value: &str, line_number: usize, field: &str) -> Result<usize, String> {
    parse_u32_number(value, line_number, field).map(|number| number as usize)
}

fn parse_u32_number(value: &str, line_number: usize, field: &str) -> Result<u32, String> {
    let parsed = value
        .strip_prefix("0x")
        .map(|hex| u32::from_str_radix(hex, 16))
        .unwrap_or_else(|| value.parse::<u32>());
    parsed.map_err(|error| format!("{field} on line {line_number} is invalid: {error}"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{
        RED_LABEL_CROM0_FIXED_ROM_BLOCK, RED_LABEL_CROM0_ROMMAP_BYTES, RedLabelRomImages,
        RomDescriptor, RomLoad, RomRegion, RomView, VerifiedRomFile, VerifiedRomSet, crc32,
        load_verified_dir_against, parse_rom_descriptors, parse_rom_map, parse_rom_regions,
        red_label_crom0_rom_diagnostics, red_label_crom0_rom_diagnostics_from_descriptors,
        red_label_crom0_rom_map_descriptors, red_label_crom0_rom_map_descriptors_from_loads,
        red_label_main_cpu_region, red_label_rom_map, red_label_rom_regions, red_label_roms,
        scan_dir_against,
    };

    const FAKE_ROMS: [RomDescriptor; 2] = [
        RomDescriptor {
            name: "vector.rom",
            size: 9,
            crc32: "cbf43926",
        },
        RomDescriptor {
            name: "empty.rom",
            size: 0,
            crc32: "00000000",
        },
    ];

    static NEXT_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "defender-rom-test-{}-{}",
                std::process::id(),
                NEXT_DIR_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn red_label_zero_checksum_rom_images(patches: &[(&str, usize, u8)]) -> RedLabelRomImages {
        let files = red_label_rom_map()
            .iter()
            .map(|load| {
                let mut bytes = vec![0; load.size];
                for (name, offset, value) in patches.iter().copied() {
                    if name == load.name {
                        bytes[offset] = value;
                    }
                }

                VerifiedRomFile {
                    descriptor: RomDescriptor {
                        name: load.name,
                        size: load.size as u64,
                        crc32: "00000000",
                    },
                    crc32: 0,
                    bytes,
                }
            })
            .collect::<Vec<_>>();
        let rom_set = VerifiedRomSet::from_files_for_test(files);

        RedLabelRomImages::from_verified_rom_set(&rom_set).expect("zero checksum ROM image")
    }

    #[test]
    fn red_label_rom_metadata_has_expected_count() {
        assert_eq!(red_label_roms().len(), 14);
        assert!(red_label_roms().iter().any(|rom| rom.name == "defend.1"));
        assert!(red_label_roms().iter().any(|rom| rom.name == "decoder.3"));
        assert_eq!(red_label_roms()[0].crc32, "c3e52d7e");
    }

    #[test]
    fn red_label_rom_region_metadata_matches_mame_regions() {
        assert_eq!(red_label_rom_regions().len(), 4);
        assert_eq!(red_label_main_cpu_region().name, "maincpu");
        assert_eq!(red_label_main_cpu_region().size, 0x3000);
        assert!(
            red_label_rom_regions()
                .iter()
                .any(|region| region.name == "banked" && region.size == 0x7000)
        );
        assert!(
            red_label_rom_regions()
                .iter()
                .any(|region| region.name == "soundcpu" && region.size == 0x10000)
        );
        assert!(
            red_label_rom_regions()
                .iter()
                .any(|region| region.name == "proms" && region.size == 0x0400)
        );
    }

    #[test]
    fn red_label_rom_map_matches_mame_load_order() {
        let map = red_label_rom_map();

        assert_eq!(map.len(), 14);
        assert_eq!(
            map[0],
            RomLoad {
                name: "defend.1",
                region: "maincpu",
                region_offset: 0x0000,
                size: 0x0800,
                view: RomView::Fixed,
                cpu_start: Some(0xD000),
            }
        );
        assert_eq!(
            map.iter()
                .copied()
                .find(|load| load.name == "defend.6")
                .expect("bank seven ROM should be mapped"),
            RomLoad {
                name: "defend.6",
                region: "banked",
                region_offset: 0x6000,
                size: 0x0800,
                view: RomView::Banked(7),
                cpu_start: Some(0xC000),
            }
        );
        assert_eq!(
            map.iter()
                .copied()
                .find(|load| load.name == "video_sound_rom_1.ic12")
                .expect("sound ROM should be mapped"),
            RomLoad {
                name: "video_sound_rom_1.ic12",
                region: "soundcpu",
                region_offset: 0xF800,
                size: 0x0800,
                view: RomView::SoundCpu,
                cpu_start: Some(0xF800),
            }
        );
        assert_eq!(
            map.iter()
                .copied()
                .find(|load| load.name == "decoder.3")
                .expect("decoder PROM should be mapped"),
            RomLoad {
                name: "decoder.3",
                region: "proms",
                region_offset: 0x0200,
                size: 0x0200,
                view: RomView::Prom,
                cpu_start: None,
            }
        );
    }

    #[test]
    fn red_label_crom0_rommap_descriptors_follow_romf8_format() {
        let descriptors = red_label_crom0_rom_map_descriptors().expect("CROM0 ROMMAP");

        assert_eq!(descriptors.len(), RED_LABEL_CROM0_ROMMAP_BYTES);
        assert_eq!(
            descriptors,
            [
                0x2F, 0x00, 0x4F, 0x5F, 0x6F, 0x7F, 0x3F, 0x00, 0x00, 0x00, 0x07, 0x00, 0x03, 0x00,
                0x02, 0x00, 0x01, 0x00, 0x13, 0x00, 0x12, 0x00, 0x11, 0x00,
            ]
        );
        assert_eq!(descriptors[0] & 0x0F, RED_LABEL_CROM0_FIXED_ROM_BLOCK);
        assert_eq!(descriptors[0] >> 4, 2);
        assert_eq!(descriptors[10], 0x07);
        assert_eq!(descriptors[16], 0x01);
        assert_eq!(descriptors[18], 0x13);
    }

    #[test]
    fn red_label_crom0_rommap_descriptors_reject_drift() {
        let bad_number = [RomLoad {
            name: "defend.13",
            region: "maincpu",
            region_offset: 0,
            size: 0x0800,
            view: RomView::Fixed,
            cpu_start: Some(0xD000),
        }];
        let oversize = [RomLoad {
            name: "defend.1",
            region: "maincpu",
            region_offset: 0,
            size: 0x1800,
            view: RomView::Fixed,
            cpu_start: Some(0xD000),
        }];
        let unaligned_address = [RomLoad {
            name: "defend.1",
            region: "maincpu",
            region_offset: 0,
            size: 0x0800,
            view: RomView::Fixed,
            cpu_start: Some(0xD400),
        }];
        let duplicate_slot = [
            RomLoad {
                name: "defend.1",
                region: "maincpu",
                region_offset: 0,
                size: 0x0800,
                view: RomView::Fixed,
                cpu_start: Some(0xD000),
            },
            RomLoad {
                name: "defend.1",
                region: "maincpu",
                region_offset: 0x0800,
                size: 0x0800,
                view: RomView::Fixed,
                cpu_start: Some(0xD800),
            },
        ];

        assert!(
            red_label_crom0_rom_map_descriptors_from_loads(&bad_number)
                .expect_err("bad ROM number should fail")
                .contains("out of range")
        );
        assert!(
            red_label_crom0_rom_map_descriptors_from_loads(&oversize)
                .expect_err("oversize ROM should fail")
                .contains("expected at most two")
        );
        assert!(
            red_label_crom0_rom_map_descriptors_from_loads(&unaligned_address)
                .expect_err("unaligned address should fail")
                .contains("not a 2K ROM boundary")
        );
        assert!(
            red_label_crom0_rom_map_descriptors_from_loads(&duplicate_slot)
                .expect_err("duplicate ROM slot should fail")
                .contains("already occupied")
        );
    }

    #[test]
    fn red_label_crom0_rom_diagnostics_pass_zero_checksum_image() {
        let images = red_label_zero_checksum_rom_images(&[]);

        let diagnostics = red_label_crom0_rom_diagnostics(&images).expect("CROM0 diagnostics");

        assert_eq!(diagnostics.checks().len(), 13);
        assert!(diagnostics.all_roms_ok());
        assert!(diagnostics.failures().is_empty());
        assert!(diagnostics.reported_bad_roms().is_empty());
        assert_eq!(diagnostics.last_reported_bad_rom(), None);
    }

    #[test]
    fn red_label_crom0_rom_diagnostics_use_source_checksum_span_and_report_one_rom_number() {
        let images = red_label_zero_checksum_rom_images(&[
            ("defend.2", 0, 0x01),
            ("defend.2", 0x0800, 0x02),
        ]);
        let mut descriptors = [0; RED_LABEL_CROM0_ROMMAP_BYTES];
        descriptors[2] = 0x4F;
        descriptors[3] = 0x5F;

        let diagnostics = red_label_crom0_rom_diagnostics_from_descriptors(&images, &descriptors)
            .expect("CROM0 diagnostics");

        assert_eq!(diagnostics.failures().len(), 2);
        assert_eq!(diagnostics.failures()[0].descriptor_index, 2);
        assert_eq!(diagnostics.failures()[0].rom_number, 2);
        assert_eq!(diagnostics.failures()[0].half, 1);
        assert_eq!(diagnostics.failures()[0].start_address, 0xE000);
        assert_eq!(diagnostics.failures()[0].checksum, 0x0300);
        assert_eq!(diagnostics.failures()[1].descriptor_index, 3);
        assert_eq!(diagnostics.failures()[1].rom_number, 2);
        assert_eq!(diagnostics.failures()[1].half, 2);
        assert_eq!(diagnostics.failures()[1].start_address, 0xE800);
        assert_eq!(diagnostics.failures()[1].checksum, 0x0200);
        assert_eq!(diagnostics.reported_bad_roms(), &[2]);
        assert_eq!(diagnostics.last_reported_bad_rom(), Some(2));
    }

    #[test]
    fn red_label_crom0_rom_diagnostics_select_banked_descriptor_view() {
        let images = red_label_zero_checksum_rom_images(&[("defend.9", 0, 0x01)]);

        let diagnostics = red_label_crom0_rom_diagnostics(&images).expect("CROM0 diagnostics");

        assert_eq!(diagnostics.failures().len(), 1);
        assert_eq!(diagnostics.failures()[0].descriptor_index, 16);
        assert_eq!(diagnostics.failures()[0].descriptor, 0x01);
        assert_eq!(diagnostics.failures()[0].rom_number, 9);
        assert_eq!(diagnostics.failures()[0].half, 1);
        assert_eq!(diagnostics.failures()[0].view, RomView::Banked(1));
        assert_eq!(diagnostics.failures()[0].start_address, 0xC000);
        assert_eq!(diagnostics.failures()[0].checksum, 0x0100);
        assert_eq!(diagnostics.reported_bad_roms(), &[9]);
    }

    #[test]
    fn red_label_crom0_rom_diagnostics_mirror_sparse_banked_half() {
        let images = red_label_zero_checksum_rom_images(&[("defend.6", 0, 0x01)]);

        let diagnostics = red_label_crom0_rom_diagnostics(&images).expect("CROM0 diagnostics");

        assert_eq!(diagnostics.failures().len(), 1);
        assert_eq!(diagnostics.failures()[0].descriptor_index, 10);
        assert_eq!(diagnostics.failures()[0].descriptor, 0x07);
        assert_eq!(diagnostics.failures()[0].rom_number, 6);
        assert_eq!(diagnostics.failures()[0].view, RomView::Banked(7));
        assert_eq!(diagnostics.failures()[0].start_address, 0xC000);
        assert_eq!(diagnostics.failures()[0].checksum, 0x0200);
        assert_eq!(diagnostics.reported_bad_roms(), &[6]);
    }

    #[test]
    fn scan_dir_reports_missing_and_unexpected_files() {
        let temp_dir = TempDir::new();
        fs::write(temp_dir.path().join("vector.rom"), b"123456789").expect("write rom");
        fs::write(temp_dir.path().join("notes.txt"), []).expect("write file");

        let report = scan_dir_against(temp_dir.path(), &FAKE_ROMS).expect("scan rom dir");

        assert_eq!(report.found_count(), 1);
        assert!(report.missing.contains(&String::from("empty.rom")));
        assert_eq!(report.unexpected, vec![String::from("notes.txt")]);
    }

    #[test]
    fn scan_dir_reports_wrong_size() {
        let temp_dir = TempDir::new();
        fs::write(temp_dir.path().join("vector.rom"), []).expect("write rom");

        let report = scan_dir_against(temp_dir.path(), &FAKE_ROMS).expect("scan rom dir");

        assert!(report.wrong_size[0].contains("vector.rom"));
    }

    #[test]
    fn scan_dir_reports_wrong_crc() {
        let temp_dir = TempDir::new();
        fs::write(temp_dir.path().join("vector.rom"), b"123456788").expect("write rom");

        let report = scan_dir_against(temp_dir.path(), &FAKE_ROMS).expect("scan rom dir");

        assert_eq!(report.found_count(), 1);
        assert!(report.wrong_crc[0].contains("vector.rom"));
        assert!(report.wrong_crc[0].contains("cbf43926"));
        assert!(!report.is_complete());
    }

    #[test]
    fn complete_report_is_complete() {
        let temp_dir = TempDir::new();
        fs::write(temp_dir.path().join("vector.rom"), b"123456789").expect("write rom");
        fs::write(temp_dir.path().join("empty.rom"), []).expect("write rom");

        let report = scan_dir_against(temp_dir.path(), &FAKE_ROMS).expect("scan rom dir");

        assert!(report.is_complete());
        assert!(report.summary_line().contains("2/2"));
        assert!(report.summary_line().contains("2/2 CRCs verified"));
    }

    #[test]
    fn verified_loader_returns_bytes_in_descriptor_order() {
        let temp_dir = TempDir::new();
        fs::write(temp_dir.path().join("vector.rom"), b"123456789").expect("write rom");
        fs::write(temp_dir.path().join("empty.rom"), []).expect("write rom");

        let rom_set = load_verified_dir_against(temp_dir.path(), &FAKE_ROMS)
            .expect("load rom dir")
            .expect("rom set should verify");

        assert_eq!(rom_set.files()[0].descriptor.name, "vector.rom");
        assert_eq!(rom_set.files()[0].crc32, 0xCBF4_3926);
        assert_eq!(
            rom_set.file_bytes("vector.rom"),
            Some(b"123456789".as_slice())
        );
        assert_eq!(rom_set.total_bytes(), 9);
    }

    #[test]
    fn verified_loader_returns_report_for_unverified_set() {
        let temp_dir = TempDir::new();
        fs::write(temp_dir.path().join("vector.rom"), b"123456788").expect("write rom");
        fs::write(temp_dir.path().join("empty.rom"), []).expect("write rom");

        let report = load_verified_dir_against(temp_dir.path(), &FAKE_ROMS)
            .expect("load rom dir")
            .expect_err("rom set should not verify");

        assert_eq!(report.found_count(), 2);
        assert_eq!(report.wrong_crc.len(), 1);
    }

    #[test]
    fn red_label_rom_images_expose_fixed_banked_sound_and_prom_views() {
        let rom_set = VerifiedRomSet {
            files: vec![
                VerifiedRomFile {
                    descriptor: RomDescriptor {
                        name: "fixed.rom",
                        size: 2,
                        crc32: "00000000",
                    },
                    crc32: 0,
                    bytes: vec![0xDA, 0x7A],
                },
                VerifiedRomFile {
                    descriptor: RomDescriptor {
                        name: "bank1.rom",
                        size: 2,
                        crc32: "00000000",
                    },
                    crc32: 0,
                    bytes: vec![0xC0, 0x01],
                },
                VerifiedRomFile {
                    descriptor: RomDescriptor {
                        name: "sound.rom",
                        size: 2,
                        crc32: "00000000",
                    },
                    crc32: 0,
                    bytes: vec![0xF8, 0x01],
                },
                VerifiedRomFile {
                    descriptor: RomDescriptor {
                        name: "prom.rom",
                        size: 2,
                        crc32: "00000000",
                    },
                    crc32: 0,
                    bytes: vec![0xDE, 0xC0],
                },
            ],
        };
        let regions = [
            RomRegion {
                name: "maincpu",
                size: 2,
                source: "test",
            },
            RomRegion {
                name: "banked",
                size: 2,
                source: "test",
            },
            RomRegion {
                name: "soundcpu",
                size: 0xF802,
                source: "test",
            },
            RomRegion {
                name: "proms",
                size: 2,
                source: "test",
            },
        ];
        let loads = [
            RomLoad {
                name: "fixed.rom",
                region: "maincpu",
                region_offset: 0,
                size: 2,
                view: RomView::Fixed,
                cpu_start: Some(0xD000),
            },
            RomLoad {
                name: "bank1.rom",
                region: "banked",
                region_offset: 0,
                size: 2,
                view: RomView::Banked(1),
                cpu_start: Some(0xC000),
            },
            RomLoad {
                name: "sound.rom",
                region: "soundcpu",
                region_offset: 0xF800,
                size: 2,
                view: RomView::SoundCpu,
                cpu_start: Some(0xF800),
            },
            RomLoad {
                name: "prom.rom",
                region: "proms",
                region_offset: 0,
                size: 2,
                view: RomView::Prom,
                cpu_start: None,
            },
        ];

        let images =
            RedLabelRomImages::from_parts(&rom_set, &regions, &loads).expect("image should build");

        assert_eq!(images.region("maincpu"), Some(regions[0]));
        assert_eq!(images.regions().len(), 4);
        assert_eq!(images.loads().len(), 4);
        assert_eq!(images.byte_at_region_offset("maincpu", 1), Some(0x7A));
        assert_eq!(images.fixed_byte(0xD000), Some(0xDA));
        assert_eq!(images.fixed_byte(0xD001), Some(0x7A));
        assert_eq!(images.fixed_byte(0xC000), None);
        assert_eq!(images.banked_byte(1, 0xC000), Some(0xC0));
        assert_eq!(images.banked_byte(1, 0xC001), Some(0x01));
        assert_eq!(images.banked_byte(2, 0xC000), None);
        assert_eq!(images.sound_cpu_byte(0xF800), Some(0xF8));
        assert_eq!(images.sound_cpu_byte(0xF801), Some(0x01));
        assert_eq!(images.prom_byte(1), Some(0xC0));
        assert_eq!(images.prom_byte(2), None);
    }

    #[test]
    fn red_label_rom_images_reject_missing_region_or_overlapping_loads() {
        let rom_set = VerifiedRomSet {
            files: vec![VerifiedRomFile {
                descriptor: RomDescriptor {
                    name: "one.rom",
                    size: 2,
                    crc32: "00000000",
                },
                crc32: 0,
                bytes: vec![1, 2],
            }],
        };
        let region = RomRegion {
            name: "maincpu",
            size: 4,
            source: "test",
        };
        let missing = [RomLoad {
            name: "missing.rom",
            region: "maincpu",
            region_offset: 0,
            size: 2,
            view: RomView::Fixed,
            cpu_start: Some(0xD000),
        }];
        let overlapping = [
            RomLoad {
                name: "one.rom",
                region: "maincpu",
                region_offset: 0,
                size: 2,
                view: RomView::Fixed,
                cpu_start: Some(0xD000),
            },
            RomLoad {
                name: "one.rom",
                region: "maincpu",
                region_offset: 1,
                size: 2,
                view: RomView::Fixed,
                cpu_start: Some(0xD001),
            },
        ];
        let missing_region = [RomLoad {
            name: "one.rom",
            region: "missing",
            region_offset: 0,
            size: 2,
            view: RomView::Fixed,
            cpu_start: Some(0xD000),
        }];

        let missing_error = RedLabelRomImages::from_parts(&rom_set, &[region], &missing)
            .expect_err("missing load should fail");
        let overlap_error = RedLabelRomImages::from_parts(&rom_set, &[region], &overlapping)
            .expect_err("overlap should fail");
        let missing_region_error =
            RedLabelRomImages::from_parts(&rom_set, &[region], &missing_region)
                .expect_err("missing region should fail");

        assert!(missing_error.contains("missing.rom"));
        assert!(overlap_error.contains("overlaps"));
        assert!(missing_region_error.contains("targets missing region"));
    }

    #[test]
    fn rom_metadata_parser_rejects_bad_size() {
        let error = parse_rom_descriptors("name\tsize\tcrc32\ndefend.1\twat\tc3e52d7e\n")
            .expect_err("metadata should fail");

        assert!(error.contains("invalid size"));
    }

    #[test]
    fn rom_metadata_parser_rejects_bad_crc() {
        let error = parse_rom_descriptors("name\tsize\tcrc32\ndefend.1\t2048\twat\n")
            .expect_err("metadata should fail");

        assert!(error.contains("invalid CRC-32"));
    }

    #[test]
    fn rom_region_parser_rejects_bad_size() {
        let error =
            parse_rom_regions("region\tsize\tsource\nmaincpu\twat\tMAME\n").expect_err("region");

        assert!(error.contains("size"));
    }

    #[test]
    fn rom_map_parser_rejects_bad_view() {
        let error = parse_rom_map(
            "name\tregion\tregion_offset\tsize\tview\tbank\tcpu_start\n\
             defend.1\tmaincpu\t0x0d000\t0x0800\twat\t-\t0xd000\n",
        )
        .expect_err("map");

        assert!(error.contains("unknown ROM map view"));
    }

    #[test]
    fn crc32_matches_the_standard_check_vector() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(&[]), 0);
    }
}
