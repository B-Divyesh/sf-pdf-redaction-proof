use chrono::Utc;
use lopdf::{
    content::{Content, Operation},
    Dictionary, Document, Object, ObjectId, Stream,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use thiserror::Error;

const MAX_BYTES: u64 = 500 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("Choose a PDF file ending in .pdf.")]
    NotPdf,
    #[error("This PDF is larger than the 500 MB safety limit.")]
    TooLarge,
    #[error("The PDF could not be read: {0}")]
    Read(String),
    #[error("The PDF is encrypted. Decrypt it in its source application, then inspect the decrypted copy.")]
    Encrypted,
    #[error("The PDF structure is damaged or unsupported: {0}")]
    Parse(String),
    #[error("The sanitized copy could not be written: {0}")]
    Write(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    pub code: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    pub title: String,
    pub detail: String,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SanitizedInfo {
    pub path: String,
    pub sha256: String,
    pub verification_verdict: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditReport {
    pub schema_version: String,
    pub generated_at: String,
    pub app_version: String,
    pub source_name: String,
    pub source_path: String,
    pub source_sha256: String,
    pub byte_size: u64,
    pub page_count: usize,
    pub verdict: String,
    pub recoverable_text_fragments: usize,
    pub redaction_regions: usize,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanitized: Option<SanitizedInfo>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    op_index: usize,
}

impl Rect {
    fn normalized(self) -> Self {
        Self {
            x: self.x.min(self.x + self.w),
            y: self.y.min(self.y + self.h),
            w: self.w.abs(),
            h: self.h.abs(),
            op_index: self.op_index,
        }
    }
    fn overlaps(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

#[derive(Default)]
struct PageScan {
    text_count: usize,
    redactions: Vec<Rect>,
    risky_ops: HashSet<usize>,
    invisible_count: usize,
    covered_count: usize,
}

fn number(obj: &Object) -> Option<f64> {
    match obj {
        Object::Integer(v) => Some(*v as f64),
        Object::Real(v) => Some(*v as f64),
        _ => None,
    }
}

fn text_len(obj: &Object) -> usize {
    match obj {
        Object::String(bytes, _) => bytes.len(),
        Object::Array(items) => items.iter().map(text_len).sum(),
        _ => 0,
    }
}

fn annotation_rects(doc: &Document, page_id: ObjectId) -> Vec<Rect> {
    let mut out = Vec::new();
    let Ok(page) = doc.get_object(page_id).and_then(Object::as_dict) else {
        return out;
    };
    let Ok(annots_obj) = page.get(b"Annots") else {
        return out;
    };
    let resolved = match annots_obj {
        Object::Reference(id) => doc.get_object(*id).ok(),
        other => Some(other),
    };
    let Some(Object::Array(annots)) = resolved else {
        return out;
    };
    for annot in annots {
        let resolved = match annot {
            Object::Reference(id) => doc.get_object(*id).ok(),
            other => Some(other),
        };
        let Some(Object::Dictionary(dict)) = resolved else {
            continue;
        };
        if !matches!(dict.get(b"Subtype"), Ok(Object::Name(name)) if name == b"Redact") {
            continue;
        }
        if let Ok(Object::Array(v)) = dict.get(b"Rect") {
            if v.len() == 4 {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (number(&v[0]), number(&v[1]), number(&v[2]), number(&v[3]))
                {
                    out.push(
                        Rect {
                            x: x1,
                            y: y1,
                            w: x2 - x1,
                            h: y2 - y1,
                            op_index: usize::MAX,
                        }
                        .normalized(),
                    );
                }
            }
        }
    }
    out
}

fn scan_operations(operations: &[Operation], annotation_regions: Vec<Rect>) -> PageScan {
    let mut scan = PageScan {
        redactions: annotation_regions,
        ..Default::default()
    };
    let mut text_items: Vec<(usize, Rect, bool)> = Vec::new();
    let mut filled_rects: Vec<Rect> = Vec::new();
    let (mut tx, mut ty, mut font_size, mut leading, mut render_mode) =
        (0.0, 0.0, 12.0, 12.0, 0i64);
    let mut pending_rect: Option<Rect> = None;
    for (index, op) in operations.iter().enumerate() {
        match op.operator.as_str() {
            "BT" => {
                tx = 0.0;
                ty = 0.0;
                render_mode = 0;
            }
            "Tf" => {
                if let Some(v) = op.operands.get(1).and_then(number) {
                    font_size = v.abs().max(1.0);
                }
            }
            "TL" => {
                if let Some(v) = op.operands.first().and_then(number) {
                    leading = v.abs().max(1.0);
                }
            }
            "Tm" => {
                if op.operands.len() >= 6 {
                    tx = number(&op.operands[4]).unwrap_or(tx);
                    ty = number(&op.operands[5]).unwrap_or(ty);
                }
            }
            "Td" | "TD" => {
                if op.operands.len() >= 2 {
                    tx += number(&op.operands[0]).unwrap_or(0.0);
                    ty += number(&op.operands[1]).unwrap_or(0.0);
                }
            }
            "T*" => ty -= leading,
            "Tr" => {
                render_mode = op
                    .operands
                    .first()
                    .and_then(|o| o.as_i64().ok())
                    .unwrap_or(0)
            }
            "re" => {
                if op.operands.len() >= 4 {
                    pending_rect = Some(
                        Rect {
                            x: number(&op.operands[0]).unwrap_or(0.0),
                            y: number(&op.operands[1]).unwrap_or(0.0),
                            w: number(&op.operands[2]).unwrap_or(0.0),
                            h: number(&op.operands[3]).unwrap_or(0.0),
                            op_index: index,
                        }
                        .normalized(),
                    );
                }
            }
            "f" | "f*" | "F" => {
                if let Some(rect) = pending_rect.take() {
                    if rect.w >= 12.0 && rect.h >= 4.0 {
                        filled_rects.push(rect);
                    }
                }
            }
            "S" | "s" | "n" => pending_rect = None,
            "Tj" | "TJ" | "'" | "\"" => {
                let len = op.operands.iter().map(text_len).sum::<usize>();
                if len > 0 {
                    let rect = Rect {
                        x: tx,
                        y: ty - font_size * 0.2,
                        w: (len as f64 * font_size * 0.52).max(2.0),
                        h: font_size * 1.15,
                        op_index: index,
                    };
                    text_items.push((index, rect, render_mode == 3));
                    scan.text_count += 1;
                    tx += rect.w;
                }
            }
            _ => {}
        }
    }
    scan.redactions
        .extend(filled_rects.into_iter().filter(|region| {
            text_items
                .iter()
                .any(|(index, text, _)| region.op_index > *index && region.overlaps(text))
        }));
    for (index, text_rect, invisible) in text_items {
        if invisible {
            scan.invisible_count += 1;
            scan.risky_ops.insert(index);
            continue;
        }
        if scan
            .redactions
            .iter()
            .any(|r| r.overlaps(&text_rect) && (r.op_index == usize::MAX || r.op_index > index))
        {
            scan.covered_count += 1;
            scan.risky_ops.insert(index);
        }
    }
    scan
}

fn deref_dict<'a>(doc: &'a Document, object: &'a Object) -> Option<&'a Dictionary> {
    match object {
        Object::Dictionary(d) => Some(d),
        Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok(),
        _ => None,
    }
}

fn page_annotation_counts(doc: &Document, page_id: ObjectId) -> (usize, usize, usize) {
    let mut annotations = 0;
    let mut attachments = 0;
    let mut actions = 0;
    let Ok(page) = doc.get_object(page_id).and_then(Object::as_dict) else {
        return (0, 0, 0);
    };
    let Ok(obj) = page.get(b"Annots") else {
        return (0, 0, 0);
    };
    let obj = match obj {
        Object::Reference(id) => doc.get_object(*id).unwrap_or(obj),
        _ => obj,
    };
    if let Object::Array(items) = obj {
        for item in items {
            if let Some(dict) = deref_dict(doc, item) {
                annotations += 1;
                if matches!(dict.get(b"Subtype"), Ok(Object::Name(n)) if n == b"FileAttachment") {
                    attachments += 1;
                }
                if dict.has(b"A") || dict.has(b"AA") {
                    actions += 1;
                }
            }
        }
    }
    (annotations, attachments, actions)
}

fn has_catalog_key(doc: &Document, key: &[u8]) -> bool {
    doc.catalog().map(|d| d.has(key)).unwrap_or(false)
}

fn has_name_tree(doc: &Document, key: &[u8]) -> bool {
    let Ok(catalog) = doc.catalog() else {
        return false;
    };
    let Ok(names_obj) = catalog.get(b"Names") else {
        return false;
    };
    deref_dict(doc, names_obj)
        .map(|d| d.has(key))
        .unwrap_or(false)
}

fn push_finding(
    findings: &mut Vec<Finding>,
    code: &str,
    severity: &str,
    page: Option<u32>,
    title: &str,
    detail: &str,
    count: usize,
) {
    if count > 0 {
        findings.push(Finding {
            code: code.into(),
            severity: severity.into(),
            page,
            title: title.into(),
            detail: detail.into(),
            count,
        });
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub(crate) fn validate_input(path: &Path) -> Result<Vec<u8>, PdfError> {
    if !path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("pdf"))
    {
        return Err(PdfError::NotPdf);
    }
    let mut file = fs::File::open(path).map_err(|e| PdfError::Read(e.to_string()))?;
    let meta = file.metadata().map_err(|e| PdfError::Read(e.to_string()))?;
    if meta.len() > MAX_BYTES {
        return Err(PdfError::TooLarge);
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    file.by_ref()
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| PdfError::Read(e.to_string()))?;
    validate_bytes(&bytes)?;
    Ok(bytes)
}

fn validate_bytes(bytes: &[u8]) -> Result<(), PdfError> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err(PdfError::TooLarge);
    }
    if !bytes.starts_with(b"%PDF-") {
        return Err(PdfError::NotPdf);
    }
    Ok(())
}

fn load_bytes(bytes: &[u8]) -> Result<Document, PdfError> {
    validate_bytes(bytes)?;
    let doc = Document::load_mem(bytes).map_err(|e| PdfError::Parse(e.to_string()))?;
    if doc.is_encrypted() {
        return Err(PdfError::Encrypted);
    }
    Ok(doc)
}

fn build_report(
    path: &Path,
    doc: &Document,
    bytes: &[u8],
) -> Result<(AuditReport, HashMap<ObjectId, PageScan>), PdfError> {
    let mut findings = Vec::new();
    let mut scans = HashMap::new();
    let mut total_text = 0;
    let mut total_regions = 0;
    let mut invisible = 0;
    let mut covered = 0;
    let mut annotations = 0;
    let mut attachments = 0;
    let mut actions = 0;
    for (page_no, page_id) in doc.get_pages() {
        let content = doc
            .get_page_content(page_id)
            .map_err(|e| PdfError::Parse(e.to_string()))?;
        let decoded = Content::decode(&content)
            .map_err(|e| PdfError::Parse(format!("page {page_no}: {e}")))?;
        let scan = scan_operations(&decoded.operations, annotation_rects(doc, page_id));
        total_text += scan.text_count;
        total_regions += scan.redactions.len();
        invisible += scan.invisible_count;
        covered += scan.covered_count;
        let counts = page_annotation_counts(doc, page_id);
        annotations += counts.0;
        attachments += counts.1;
        actions += counts.2;
        if scan.covered_count > 0 {
            push_finding(&mut findings,"covered_text","critical",Some(page_no),"Text survives beneath a redaction", "Text-painting instructions overlap a later opaque rectangle or a redaction annotation and can remain recoverable.",scan.covered_count);
        }
        if scan.invisible_count > 0 {
            push_finding(&mut findings,"invisible_text","critical",Some(page_no),"Invisible text layer is recoverable","Text uses PDF rendering mode 3, which hides glyphs visually without removing their content.",scan.invisible_count);
        }
        scans.insert(page_id, scan);
    }
    let metadata =
        usize::from(doc.trailer.has(b"Info")) + usize::from(has_catalog_key(doc, b"Metadata"));
    attachments += usize::from(has_name_tree(doc, b"EmbeddedFiles"));
    actions += usize::from(has_catalog_key(doc, b"OpenAction"))
        + usize::from(has_catalog_key(doc, b"AA"))
        + usize::from(has_name_tree(doc, b"JavaScript"));
    push_finding(&mut findings,"metadata","warning",None,"Document metadata is present","Authoring or XMP metadata can disclose names, software, paths, dates, or workflow details.",metadata);
    push_finding(
        &mut findings,
        "attachments",
        "critical",
        None,
        "Embedded or attached content is present",
        "Attached files and name trees may carry document content outside the visible pages.",
        attachments,
    );
    push_finding(&mut findings,"actions","critical",None,"Automatic actions are present","The PDF contains launch, JavaScript, URI, or other action dictionaries that should not travel with a redacted copy.",actions);
    push_finding(&mut findings,"annotations","warning",None,"Removable annotations are present","Comments, redaction marks, and other annotations remain independently removable or inspectable.",annotations);
    push_finding(
        &mut findings,
        "forms",
        "warning",
        None,
        "Interactive form data is present",
        "Form fields can retain entered values outside the flattened page appearance.",
        usize::from(has_catalog_key(doc, b"AcroForm")),
    );
    push_finding(
        &mut findings,
        "layers",
        "warning",
        None,
        "Optional content layers are present",
        "PDF layers can hide content without deleting it.",
        usize::from(has_catalog_key(doc, b"OCProperties")),
    );
    let verdict = if covered + invisible + attachments + actions > 0 {
        "fail"
    } else if findings.is_empty() {
        "pass"
    } else {
        "review"
    };
    let report = AuditReport {
        schema_version:"1.0".into(), generated_at:Utc::now().to_rfc3339(), app_version:env!("CARGO_PKG_VERSION").into(),
        source_name:path.file_name().and_then(|x|x.to_str()).unwrap_or("document.pdf").into(), source_path:path.to_string_lossy().into_owned(),
        source_sha256:hash_bytes(bytes), byte_size:bytes.len() as u64, page_count:doc.get_pages().len(), verdict:verdict.into(),
        recoverable_text_fragments:total_text, redaction_regions:total_regions, findings, sanitized:None,
        limitations:vec![
            "This is a technical inspection, not a legal guarantee or a substitute for reviewing the output.".into(),
            "Opaque cover detection is conservative and strongest for axis-aligned rectangles and standard text operators; unusual transforms, clipped paths, or image-only secrets require visual review.".into(),
            "The sanitizer removes risky structural data and overlapping text instructions. It does not use OCR to infer secrets already baked into page images.".into(),
        ],
    };
    Ok((report, scans))
}

pub(crate) fn inspect_bytes(source_name: &str, bytes: &[u8]) -> Result<AuditReport, PdfError> {
    let doc = load_bytes(bytes)?;
    Ok(build_report(Path::new(source_name), &doc, bytes)?.0)
}

#[cfg(test)]
pub fn inspect(path: &str) -> Result<AuditReport, PdfError> {
    let path = Path::new(path);
    let bytes = validate_input(path)?;
    let mut report = inspect_bytes(
        path.file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("document.pdf"),
        &bytes,
    )?;
    report.source_path = path.to_string_lossy().into_owned();
    Ok(report)
}

pub(crate) fn output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let mut candidate = input.with_file_name(format!("{stem}.sanitized.pdf"));
    let mut n = 2;
    while candidate.exists() {
        candidate = input.with_file_name(format!("{stem}.sanitized-{n}.pdf"));
        n += 1;
    }
    candidate
}

fn clean_catalog(doc: &mut Document) {
    let keys: [&[u8]; 8] = [
        b"Metadata",
        b"Names",
        b"OpenAction",
        b"AA",
        b"AcroForm",
        b"OCProperties",
        b"PieceInfo",
        b"AF",
    ];
    if let Ok(root) = doc.trailer.get(b"Root").and_then(Object::as_reference) {
        if let Ok(Object::Dictionary(dict)) = doc.get_object_mut(root) {
            for key in keys {
                dict.remove(key);
            }
        }
    }
    doc.trailer.remove(b"Info");
    doc.trailer.remove(b"Prev");
}

pub(crate) fn sanitize_bytes(
    source_name: &str,
    bytes: &[u8],
) -> Result<(AuditReport, Vec<u8>), PdfError> {
    let input = Path::new(source_name);
    let mut doc = load_bytes(bytes)?;
    let (mut original, scans) = build_report(input, &doc, bytes)?;
    for (page_id, scan) in scans {
        let bytes = doc
            .get_page_content(page_id)
            .map_err(|e| PdfError::Parse(e.to_string()))?;
        let mut content = Content::decode(&bytes).map_err(|e| PdfError::Parse(e.to_string()))?;
        content.operations = content
            .operations
            .into_iter()
            .enumerate()
            .filter_map(|(i, op)| {
                if scan.risky_ops.contains(&i) {
                    None
                } else {
                    Some(op)
                }
            })
            .collect();
        if !scan.redactions.is_empty() {
            content.operations.push(Operation::new("q", vec![]));
            content.operations.push(Operation::new("g", vec![0.into()]));
            for rect in &scan.redactions {
                content.operations.push(Operation::new(
                    "re",
                    vec![rect.x.into(), rect.y.into(), rect.w.into(), rect.h.into()],
                ));
                content.operations.push(Operation::new("f", vec![]));
            }
            content.operations.push(Operation::new("Q", vec![]));
        }
        let encoded = content
            .encode()
            .map_err(|e| PdfError::Parse(e.to_string()))?;
        let stream_id = doc.add_object(Stream::new(Dictionary::new(), encoded));
        if let Ok(Object::Dictionary(page)) = doc.get_object_mut(page_id) {
            page.set("Contents", stream_id);
            page.remove(b"Annots");
            page.remove(b"AA");
        }
    }
    clean_catalog(&mut doc);
    doc.prune_objects();
    doc.compress();
    let mut clean_bytes = Vec::new();
    doc.save_to(&mut clean_bytes)
        .map_err(|e| PdfError::Write(e.to_string()))?;
    let verification = inspect_bytes("sanitized.pdf", &clean_bytes)?;
    original.sanitized = Some(SanitizedInfo {
        path: "sanitized.pdf".into(),
        sha256: verification.source_sha256.clone(),
        verification_verdict: verification.verdict,
    });
    Ok((original, clean_bytes))
}

#[cfg(test)]
pub fn sanitize(path: &str) -> Result<AuditReport, PdfError> {
    let input = Path::new(path);
    let bytes = validate_input(input)?;
    let source_name = input
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("document.pdf");
    let (mut report, clean_bytes) = sanitize_bytes(source_name, &bytes)?;
    let output = output_path(input);
    fs::write(&output, clean_bytes).map_err(|e| PdfError::Write(e.to_string()))?;
    report.source_path = input.to_string_lossy().into_owned();
    if let Some(info) = &mut report.sanitized {
        info.path = output.to_string_lossy().into_owned();
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Object};
    use serde::Deserialize;
    use tempfile::tempdir;

    #[derive(Deserialize)]
    struct CorpusFixture {
        name: String,
        kind: String,
        expected: String,
    }

    fn corpus_manifest() -> Vec<CorpusFixture> {
        serde_json::from_str(include_str!(
            "../../tests/fixtures/pdf-corpus/manifest.json"
        ))
        .unwrap()
    }

    fn corpus_pdf(fixture: &CorpusFixture) -> Vec<u8> {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();
        let font_id =
            doc.add_object(dictionary! {"Type"=>"Font","Subtype"=>"Type1","BaseFont"=>"Helvetica"});
        let resources_id = doc.add_object(dictionary! {"Font"=>dictionary!{"F1"=>font_id}});
        let content = match fixture.kind.as_str() {
            "invisible_text" => "BT /F1 12 Tf 3 Tr 72 700 Td (hidden alpha) Tj ET",
            "invisible_text_large" => "BT /F1 24 Tf 3 Tr 140 420 Td [(hidden) 20 ( beta)] TJ ET",
            "covered_text" => "BT /F1 12 Tf 72 700 Td (covered alpha) Tj ET 68 697 150 18 re f",
            "covered_text_array" => {
                "BT /F1 11 Tf 90 530 Td [(covered) 10 ( beta)] TJ ET 88 527 160 17 re f"
            }
            "covered_text_shifted" => {
                "BT /F1 16 Tf 210 310 Td (covered gamma) Tj ET 205 306 190 24 re f"
            }
            "redaction_annotation" => "BT /F1 12 Tf 72 700 Td (covered by annotation) Tj ET",
            "safe_visible_text" => "BT /F1 12 Tf 72 700 Td (ordinary visible text) Tj ET",
            _ => "",
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.as_bytes().to_vec()));
        let mut page = dictionary! {
            "Type"=>"Page", "Parent"=>pages_id,
            "MediaBox"=>vec![0.into(),0.into(),612.into(),792.into()],
            "Contents"=>content_id, "Resources"=>resources_id
        };
        let annotation_kind = match fixture.kind.as_str() {
            "redaction_annotation" => Some("Redact"),
            "annotation_text" => Some("Text"),
            "annotation_stamp" => Some("Stamp"),
            "attachment_annotation" | "attachment_both" => Some("FileAttachment"),
            "action_annotation" => Some("Link"),
            _ => None,
        };
        if let Some(subtype) = annotation_kind {
            let mut annotation = dictionary! {
                "Type"=>"Annot", "Subtype"=>subtype,
                "Rect"=>vec![68.into(),697.into(),230.into(),720.into()]
            };
            if fixture.kind == "action_annotation" {
                annotation.set("A", dictionary! {"S"=>"URI","URI"=>Object::string_literal("https://example.invalid")});
            }
            let annotation_id = doc.add_object(annotation);
            page.set("Annots", vec![annotation_id.into()]);
        }
        doc.objects.insert(page_id, Object::Dictionary(page));
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type"=>"Pages", "Kids"=>vec![page_id.into()], "Count"=>1
            }),
        );
        let mut catalog = dictionary! {"Type"=>"Catalog","Pages"=>pages_id};
        match fixture.kind.as_str() {
            "metadata_xmp" | "metadata_both" => catalog.set("Metadata", dictionary! {"Type"=>"Metadata"}),
            "attachment_tree" | "attachment_both" => catalog.set("Names", dictionary! {"EmbeddedFiles"=>dictionary! {"Names"=>Vec::<Object>::new()}}),
            "action_open" => catalog.set("OpenAction", dictionary! {"S"=>"JavaScript"}),
            "action_catalog_aa" => catalog.set("AA", dictionary! {"WC"=>dictionary! {"S"=>"JavaScript"}}),
            "action_javascript" => catalog.set("Names", dictionary! {"JavaScript"=>dictionary! {"Names"=>Vec::<Object>::new()}}),
            "layer_single" => catalog.set("OCProperties", dictionary! {"OCGs"=>Vec::<Object>::new()}),
            "layer_group" => catalog.set("OCProperties", dictionary! {"D"=>dictionary! {"Order"=>Vec::<Object>::new()}}),
            "form_empty" => catalog.set("AcroForm", dictionary! {"Fields"=>Vec::<Object>::new()}),
            "form_field" => catalog.set("AcroForm", dictionary! {"Fields"=>vec![dictionary! {"T"=>Object::string_literal("secret")}.into()]}),
            _ => {}
        }
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set("Root", catalog_id);
        if matches!(fixture.kind.as_str(), "metadata_info" | "metadata_both") {
            let info =
                doc.add_object(dictionary! {"Author"=>Object::string_literal("Sensitive Name")});
            doc.trailer.set("Info", info);
        }
        doc.compress();
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).unwrap();
        bytes
    }

    fn committed_corpus_pdf(fixture: &CorpusFixture) -> Vec<u8> {
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../tests/fixtures/pdf-corpus")
                .join(format!("{}.pdf", fixture.name)),
        )
        .unwrap_or_else(|error| panic!("missing committed fixture {}: {error}", fixture.name))
    }

    #[test]
    #[ignore = "maintenance helper; generated PDFs are committed"]
    fn regenerate_committed_corpus() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/pdf-corpus");
        fs::create_dir_all(&directory).unwrap();
        for fixture in corpus_manifest() {
            fs::write(
                directory.join(format!("{}.pdf", fixture.name)),
                corpus_pdf(&fixture),
            )
            .unwrap();
        }
    }

    fn sample(path: &Path, hidden: bool, metadata: bool) {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();
        let font_id =
            doc.add_object(dictionary! {"Type"=>"Font","Subtype"=>"Type1","BaseFont"=>"Helvetica"});
        let resources_id = doc.add_object(dictionary! {"Font"=>dictionary!{"F1"=>font_id}});
        let mode = if hidden { "3 Tr" } else { "0 Tr" };
        let stream = format!("BT /F1 12 Tf 72 720 Td {mode} (secret value) Tj ET");
        let content_id = doc.add_object(Stream::new(dictionary! {}, stream.into_bytes()));
        doc.objects.insert(page_id,Object::Dictionary(dictionary!{"Type"=>"Page","Parent"=>pages_id,"MediaBox"=>vec![0.into(),0.into(),612.into(),792.into()],"Contents"=>content_id,"Resources"=>resources_id}));
        doc.objects.insert(
            pages_id,
            Object::Dictionary(
                dictionary! {"Type"=>"Pages","Kids"=>vec![page_id.into()],"Count"=>1},
            ),
        );
        let catalog_id = doc.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages_id});
        doc.trailer.set("Root", catalog_id);
        if metadata {
            let info =
                doc.add_object(dictionary! {"Author"=>Object::string_literal("Sensitive Name")});
            doc.trailer.set("Info", info);
        }
        doc.compress();
        doc.save(path).unwrap();
    }

    #[test]
    fn detects_invisible_text_and_metadata() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("seed.pdf");
        sample(&path, true, true);
        let report = inspect(path.to_str().unwrap()).unwrap();
        assert_eq!(report.verdict, "fail");
        assert!(report.findings.iter().any(|f| f.code == "invisible_text"));
        assert!(report.findings.iter().any(|f| f.code == "metadata"));
    }
    #[test]
    fn sanitizer_removes_seeded_risks() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("seed.pdf");
        sample(&path, true, true);
        let report = sanitize(path.to_str().unwrap()).unwrap();
        let clean = inspect(&report.sanitized.unwrap().path).unwrap();
        assert_eq!(clean.verdict, "pass");
        assert!(!clean.findings.iter().any(|f| f.code == "metadata"));
    }
    #[test]
    fn rejects_non_pdf() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("x.pdf");
        fs::write(&path, b"hello").unwrap();
        assert!(matches!(
            inspect(path.to_str().unwrap()),
            Err(PdfError::NotPdf)
        ));
    }
    #[test]
    fn claim_input_limit_rejects_files_over_500_mb_before_reading() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("oversize.pdf");
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_BYTES + 1).unwrap();
        assert!(matches!(validate_input(&path), Err(PdfError::TooLarge)));
    }
    #[test]
    fn claim_core_detection_corpus() {
        let fixtures = corpus_manifest();
        let mut detected = 0;
        let mut risky = 0;
        let mut by_category: HashMap<String, (usize, usize)> = HashMap::new();
        for fixture in &fixtures {
            let report = inspect_bytes(
                &format!("{}.pdf", fixture.name),
                &committed_corpus_pdf(fixture),
            )
            .unwrap();
            if fixture.expected == "pass" {
                assert_eq!(report.verdict, "pass", "{} should be clean", fixture.name);
                continue;
            }
            risky += 1;
            let found = report
                .findings
                .iter()
                .any(|finding| finding.code == fixture.expected);
            let entry = by_category.entry(fixture.expected.clone()).or_default();
            entry.1 += 1;
            if found {
                detected += 1;
                entry.0 += 1;
            }
            assert!(
                found,
                "{} did not report {}",
                fixture.name, fixture.expected
            );
        }
        let rate = detected as f64 / risky as f64;
        eprintln!(
            "detection corpus: {detected}/{risky} ({:.1}%); {by_category:?}",
            rate * 100.0
        );
        assert!(
            rate >= 0.95,
            "detected {detected}/{risky} varied risky fixtures"
        );
    }

    #[test]
    fn claim_sanitized_copy_preserves_original_and_clears_structural_risks() {
        for fixture in corpus_manifest()
            .iter()
            .filter(|fixture| fixture.expected != "pass")
        {
            let bytes = committed_corpus_pdf(fixture);
            let before_hash = hash_bytes(&bytes);
            let (report, clean_bytes) =
                sanitize_bytes(&format!("{}.pdf", fixture.name), &bytes).unwrap();
            assert_eq!(
                hash_bytes(&bytes),
                before_hash,
                "{} input changed",
                fixture.name
            );
            assert_ne!(
                clean_bytes, bytes,
                "{} did not create a separate copy",
                fixture.name
            );
            assert_eq!(report.source_sha256, before_hash);
            let clean = inspect_bytes("clean.pdf", &clean_bytes).unwrap();
            assert_eq!(
                clean.verdict, "pass",
                "{} remained risky after sanitizing",
                fixture.name
            );
            let proof = report.sanitized.as_ref().unwrap();
            assert_eq!(proof.sha256, clean.source_sha256);
            assert_eq!(proof.verification_verdict, "pass");
        }
    }
}
