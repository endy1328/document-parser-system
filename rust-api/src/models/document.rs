use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileType {
    Pdf,
    Docx,
    Xlsx,
    Txt,
    Md,
    Unknown,
}

impl FileType {
    pub fn from_filename(filename: &str) -> Self {
        let extension = filename.split('.').last().unwrap_or("").to_lowercase();
        match extension.as_str() {
            "pdf" => FileType::Pdf,
            "docx" => FileType::Docx,
            "xlsx" => FileType::Xlsx,
            "txt" => FileType::Txt,
            "md" => FileType::Md,
            _ => FileType::Unknown,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            FileType::Pdf => "pdf",
            FileType::Docx => "docx",
            FileType::Xlsx => "xlsx",
            FileType::Txt => "txt",
            FileType::Md => "md",
            FileType::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedContent {
    pub text: Option<String>,
    pub sheets: Option<Vec<SheetData>>,
    pub metadata: DocumentMetadata,
    pub thumbnail: Option<String>,      // Base64로 인코딩된 썸네일 이미지
    pub original_file: Option<String>, // Base64로 인코딩된 원본 파일
    pub html: Option<String>,          // 생성된 편집 가능한 HTML
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SheetData {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_type: FileType,
    pub file_size: u64,
    pub page_count: Option<u32>,
    pub sheet_count: Option<u32>,
    pub created_at: String,
}
