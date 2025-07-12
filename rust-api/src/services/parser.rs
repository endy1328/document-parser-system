use crate::models::document::{FileType, ParsedContent, SheetData, DocumentMetadata};
use std::process::Command;
use std::fs;
use std::path::Path;
use chrono::Utc;
use std::io::Read;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};

pub struct DocumentParser;

impl DocumentParser {
    pub fn new() -> Self {
        Self
    }

    pub async fn parse_document(&self, file_path: &str, file_type: FileType) -> Result<ParsedContent, String> {
        let metadata = self.extract_metadata(file_path, &file_type)?;
        
        match file_type {
            FileType::Pdf => self.parse_pdf(file_path, metadata).await,
            FileType::Docx => self.parse_docx(file_path, metadata).await,
            FileType::Xlsx => self.parse_xlsx(file_path, metadata).await,
            FileType::Txt => self.parse_txt(file_path, metadata).await,
            FileType::Md => self.parse_md(file_path, metadata).await,
            FileType::Unknown => Err("지원하지 않는 파일 형식입니다.".to_string()),
        }
    }

    async fn parse_pdf(&self, file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        // PDF 텍스트 추출
        let text = match Command::new("pdftotext").arg(file_path).arg("-").output() {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    return Err(format!("PDF 파싱 오류: {}", err));
                }
            },
            Err(e) => return Err(format!("pdftotext 실행 실패: {}", e)),
        };
        
        // 썸네일 생성
        let thumbnail = self.generate_thumbnail(file_path).await?;
        
        // 원본 파일 읽기
        let original_file = self.read_file_as_base64(file_path)?;
        
        // 결과 반환
        Ok(ParsedContent {
            text: Some(text),
            sheets: None,
            metadata,
            thumbnail,
            original_file: Some(original_file),
        })
    }

    async fn parse_docx(&self, file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        // docx2txt 명령어 사용 (시스템에 설치 필요)
        match Command::new("docx2txt").arg(file_path).arg("-").output() {
            Ok(output) => {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    // 원본 파일 읽기
                    let original_file = self.read_file_as_base64(file_path)?;
                    Ok(ParsedContent {
                        text: Some(text),
                        sheets: None,
                        metadata,
                        thumbnail: None, // DOCX는 썸네일 생성 안함
                        original_file: Some(original_file),
                    })
                } else {
                    // fallback: 기본 텍스트 추출 시도
                    self.parse_docx_fallback(file_path, metadata).await
                }
            },
            Err(_) => {
                // docx2txt가 없으면 fallback 사용
                self.parse_docx_fallback(file_path, metadata).await
            }
        }
    }

    async fn parse_docx_fallback(&self, _file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        // 간단한 DOCX 파싱 (ZIP 압축 해제 후 XML 파싱)
        // 실제 구현에서는 docx-rs 크레이트 사용 권장
        Ok(ParsedContent {
            text: Some("DOCX 파일이 업로드되었습니다. (상세 파싱 미구현)".to_string()),
            sheets: None,
            metadata,
            thumbnail: None, // DOCX는 썸네일 생성 안함
            original_file: None,
        })
    }

    async fn parse_xlsx(&self, file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        // xlsx2csv 명령어 사용 (시스템에 설치 필요)
        match Command::new("xlsx2csv").arg(file_path).output() {
            Ok(output) => {
                if output.status.success() {
                    let csv_content = String::from_utf8_lossy(&output.stdout).to_string();
                    let rows: Vec<Vec<String>> = csv_content
                        .lines()
                        .map(|line| line.split(',').map(|s| s.trim().to_string()).collect())
                        .collect();
                    
                    let sheet = SheetData {
                        name: "Sheet1".to_string(),
                        rows,
                    };
                    
                    // 원본 파일 읽기
                    let original_file = self.read_file_as_base64(file_path)?;
                    
                    Ok(ParsedContent {
                        text: None,
                        sheets: Some(vec![sheet]),
                        metadata,
                        thumbnail: None, // XLSX는 썸네일 생성 안함
                        original_file: Some(original_file),
                    })
                } else {
                    self.parse_xlsx_fallback(file_path, metadata).await
                }
            },
            Err(_) => {
                self.parse_xlsx_fallback(file_path, metadata).await
            }
        }
    }

    async fn parse_xlsx_fallback(&self, _file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        // 간단한 XLSX 파싱 fallback
        Ok(ParsedContent {
            text: Some("XLSX 파일이 업로드되었습니다. (상세 파싱 미구현)".to_string()),
            sheets: None,
            metadata,
            thumbnail: None, // XLSX는 썸네일 생성 안함
            original_file: None,
        })
    }

    async fn parse_txt(&self, file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                // 원본 파일 읽기
                let original_file = self.read_file_as_base64(file_path)?;
                Ok(ParsedContent {
                    text: Some(content),
                    sheets: None,
                    metadata,
                    thumbnail: None, // TXT는 썸네일 생성 안함
                    original_file: Some(original_file),
                })
            },
            Err(e) => Err(format!("TXT 파일 읽기 실패: {}", e)),
        }
    }

    async fn parse_md(&self, file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                // 원본 파일 읽기
                let original_file = self.read_file_as_base64(file_path)?;
                Ok(ParsedContent {
                    text: Some(content),
                    sheets: None,
                    metadata,
                    thumbnail: None, // MD는 썸네일 생성 안함
                    original_file: Some(original_file),
                })
            },
            Err(e) => Err(format!("MD 파일 읽기 실패: {}", e)),
        }
    }

    fn extract_metadata(&self, file_path: &str, file_type: &FileType) -> Result<DocumentMetadata, String> {
        let path = Path::new(file_path);
        let file_size = match fs::metadata(path) {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        };
        
        // PDF 파일인 경우 페이지 수 추출
        let page_count = if *file_type == FileType::Pdf {
            self.get_pdf_page_count(file_path)
        } else {
            None
        };
        
        // XLSX 파일인 경우 시트 수 추출 (미구현)
        let sheet_count = if *file_type == FileType::Xlsx {
            None // TODO: 시트 수 추출 구현
        } else {
            None
        };

        Ok(DocumentMetadata {
            file_type: file_type.clone(),
            file_size,
            page_count,
            sheet_count,
            created_at: Utc::now().to_rfc3339(),
        })
    }
    
    // PDF 페이지 수 추출
    fn get_pdf_page_count(&self, file_path: &str) -> Option<u32> {
        match Command::new("pdfinfo").arg(file_path).output() {
            Ok(output) => {
                if output.status.success() {
                    let info = String::from_utf8_lossy(&output.stdout);
                    for line in info.lines() {
                        if line.starts_with("Pages:") {
                            return line.split_whitespace()
                                .nth(1)
                                .and_then(|s| s.parse::<u32>().ok());
                        }
                    }
                }
                None
            },
            Err(_) => None,
        }
    }
    
    // 파일을 Base64로 인코딩
    fn read_file_as_base64(&self, file_path: &str) -> Result<String, String> {
        let mut file = fs::File::open(file_path)
            .map_err(|e| format!("파일 열기 실패: {}", e))?;
            
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("파일 읽기 실패: {}", e))?;
            
        Ok(BASE64_STANDARD.encode(&buffer))
    }
    
    // PDF 썸네일 생성
    async fn generate_thumbnail(&self, file_path: &str) -> Result<Option<String>, String> {
        // 썸네일 저장 디렉토리 생성
        let thumbnail_dir = "thumbnails";
        fs::create_dir_all(thumbnail_dir).map_err(|e| format!("썸네일 디렉토리 생성 실패: {}", e))?;
        
        // 파일 이름 추출
        let path = Path::new(file_path);
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
            
        // 썸네일 파일 경로 (현재 사용하지 않음)
        let _thumbnail_path = format!("{}/{}.jpg", thumbnail_dir, file_stem);
        
        // PDF 첫 페이지를 JPG로 변환 (pdftoppm 사용)
        let output = Command::new("pdftoppm")
            .args(["-jpeg", "-f", "1", "-l", "1", "-scale-to", "300", file_path, &format!("{}/{}", thumbnail_dir, file_stem)])
            .output()
            .map_err(|e| format!("pdftoppm 실행 실패: {}", e))?;
            
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("썸네일 생성 실패: {}", err));
        }
        
        // 생성된 썸네일 파일 찾기 (pdftoppm은 -1.jpg 같은 접미사를 추가함)
        let thumbnail_file = format!("{}/{}-1.jpg", thumbnail_dir, file_stem);
        if Path::new(&thumbnail_file).exists() {
            // 썸네일을 Base64로 인코딩
            match self.read_file_as_base64(&thumbnail_file) {
                Ok(base64_thumbnail) => Ok(Some(base64_thumbnail)),
                Err(e) => Err(format!("썸네일 인코딩 실패: {}", e))
            }
        } else {
            // 썸네일 생성 실패했지만 치명적 오류는 아님
            Ok(None)
        }
    }
}
