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
        
        // PDF에서 이미지 추출
        let extracted_images = self.extract_pdf_images(file_path).await?;
        
        // HTML 생성 (추출된 이미지 포함)
        let html = self.generate_pdf_html(&text, &metadata, &extracted_images, &thumbnail).await?;
        
        // 결과 반환
        Ok(ParsedContent {
            text: Some(text),
            sheets: None,
            metadata,
            thumbnail,
            original_file: Some(original_file),
            html: Some(html),
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
                    // HTML 생성
                    let html = self.generate_simple_html(&text, &metadata, "DOCX 문서", &None).await?;
                    Ok(ParsedContent {
                        text: Some(text),
                        sheets: None,
                        metadata,
                        thumbnail: None, // DOCX는 썸네일 생성 안함
                        original_file: Some(original_file),
                        html: Some(html),
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
        let text = "DOCX 파일이 업로드되었습니다. (상세 파싱 미구현)".to_string();
        let html = self.generate_simple_html(&text, &metadata, "DOCX 문서", &None).await?;
        Ok(ParsedContent {
            text: Some(text),
            sheets: None,
            metadata,
            thumbnail: None, // DOCX는 썸네일 생성 안함
            original_file: None,
            html: Some(html),
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
                    let sheets = vec![sheet];
                    
                    // 원본 파일 읽기
                    let original_file = self.read_file_as_base64(file_path)?;
                    
                    let html = self.generate_xlsx_html(&sheets, &metadata).await?;
                    Ok(ParsedContent {
                        text: None,
                        sheets: Some(sheets),
                        metadata,
                        thumbnail: None,
                        original_file: Some(original_file),
                        html: Some(html),
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
        let text = "XLSX 파일이 업로드되었습니다. (상세 파싱 미구현)".to_string();
        let html = self.generate_simple_html(&text, &metadata, "XLSX 문서", &None).await?;
        Ok(ParsedContent {
            text: Some(text),
            sheets: None,
            metadata,
            thumbnail: None,
            original_file: None,
            html: Some(html),
        })
    }

    async fn parse_txt(&self, file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                let original_file = self.read_file_as_base64(file_path)?;
                let html = self.generate_simple_html(&content, &metadata, "텍스트 문서", &None).await?;
                Ok(ParsedContent {
                    text: Some(content),
                    sheets: None,
                    metadata,
                    thumbnail: None,
                    original_file: Some(original_file),
                    html: Some(html),
                })
            },
            Err(e) => Err(format!("TXT 파일 읽기 실패: {}", e)),
        }
    }

    async fn parse_md(&self, file_path: &str, metadata: DocumentMetadata) -> Result<ParsedContent, String> {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                let original_file = self.read_file_as_base64(file_path)?;
                let html = self.generate_simple_html(&content, &metadata, "마크다운 문서", &None).await?;
                Ok(ParsedContent {
                    text: Some(content),
                    sheets: None,
                    metadata,
                    thumbnail: None,
                    original_file: Some(original_file),
                    html: Some(html),
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
    
    // 단순하고 편집 가능한 HTML 생성
    async fn generate_simple_html(
        &self, 
        text: &str, 
        metadata: &DocumentMetadata, 
        doc_type: &str,
        thumbnail: &Option<String>
    ) -> Result<String, String> {
        let mut html = String::new();
        
        // HTML 기본 구조
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang='ko'>\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset='UTF-8'>\n");
        html.push_str("    <meta name='viewport' content='width=device-width, initial-scale=1.0'>\n");
        html.push_str(&format!("    <title>{} - 편집 가능한 문서</title>\n", doc_type));
        html.push_str("    <style>\n");
        html.push_str("        body { font-family: 'Malgun Gothic', Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; line-height: 1.6; background: #fff; }\n");
        html.push_str("        .header { border-bottom: 2px solid #333; padding-bottom: 10px; margin-bottom: 20px; }\n");
        html.push_str("        .title { font-size: 24px; font-weight: bold; color: #333; margin-bottom: 10px; }\n");
        html.push_str("        .metadata { font-size: 12px; color: #666; margin-bottom: 20px; }\n");
        html.push_str("        .thumbnail { text-align: center; margin: 20px 0; }\n");
        html.push_str("        .thumbnail img { max-width: 300px; border: 1px solid #ddd; border-radius: 5px; }\n");
        html.push_str("        .content { line-height: 1.8; white-space: pre-wrap; background: #f9f9f9; padding: 20px; border-radius: 5px; margin: 20px 0; }\n");
        html.push_str("        .editable { border: none; outline: none; background: transparent; font-family: inherit; font-size: inherit; width: 100%; resize: vertical; }\n");
        html.push_str("        .images-folder { margin-top: 20px; padding: 10px; background: #f0f8ff; border-radius: 5px; font-size: 12px; color: #666; }\n");
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        
        // 헤더 정보
        html.push_str("    <div class='header'>\n");
        html.push_str(&format!("        <div class='title'>{}</div>\n", doc_type));
        html.push_str("        <div class='metadata'>\n");
        html.push_str(&format!("            파일 크기: {:.1} KB | ", metadata.file_size as f64 / 1024.0));
        if let Some(pages) = metadata.page_count {
            html.push_str(&format!("페이지 수: {} | ", pages));
        }
        html.push_str(&format!("생성일: {}\n", metadata.created_at));
        html.push_str("        </div>\n");
        html.push_str("    </div>\n");
        
        // 썸네일 표시 (PDF인 경우)
        if let Some(thumb_base64) = thumbnail {
            html.push_str("    <div class='thumbnail'>\n");
            html.push_str(&format!("        <img src='data:image/jpeg;base64,{}' alt='문서 썸네일' />\n", thumb_base64));
            html.push_str("    </div>\n");
        }
        
        // 편집 가능한 내용 영역
        html.push_str("    <div class='content'>\n");
        html.push_str("        <textarea class='editable' rows='30'>\n");
        html.push_str(&self.escape_html(text));
        html.push_str("        </textarea>\n");
        html.push_str("    </div>\n");
        
        // 이미지 폴더 안내
        html.push_str("    <div class='images-folder'>\n");
        html.push_str("        💡 이미지 파일들은 './images/' 폴더에 저장되며, HTML에서 <img src='images/파일명.jpg'> 형식으로 사용할 수 있습니다.\n");
        html.push_str("    </div>\n");
        
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        
        Ok(html)
    }
    
    // XLSX용 HTML 생성
    async fn generate_xlsx_html(
        &self, 
        sheets: &Vec<crate::models::document::SheetData>, 
        metadata: &DocumentMetadata
    ) -> Result<String, String> {
        let mut html = String::new();
        
        // HTML 기본 구조
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang='ko'>\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset='UTF-8'>\n");
        html.push_str("    <meta name='viewport' content='width=device-width, initial-scale=1.0'>\n");
        html.push_str("    <title>XLSX 문서 - 편집 가능한 스프레드시트</title>\n");
        html.push_str("    <style>\n");
        html.push_str("        body { font-family: 'Malgun Gothic', Arial, sans-serif; margin: 20px; }\n");
        html.push_str("        .header { border-bottom: 2px solid #333; padding-bottom: 10px; margin-bottom: 20px; }\n");
        html.push_str("        .title { font-size: 24px; font-weight: bold; color: #333; margin-bottom: 10px; }\n");
        html.push_str("        .metadata { font-size: 12px; color: #666; margin-bottom: 20px; }\n");
        html.push_str("        .sheet { margin-bottom: 30px; }\n");
        html.push_str("        .sheet-title { font-size: 18px; font-weight: bold; color: #444; margin-bottom: 10px; }\n");
        html.push_str("        table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }\n");
        html.push_str("        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("        th { background-color: #f5f5f5; font-weight: bold; }\n");
        html.push_str("        tr:nth-child(even) { background-color: #f9f9f9; }\n");
        html.push_str("        .editable-cell { border: none; background: transparent; width: 100%; padding: 4px; }\n");
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        
        // 헤더 정보
        html.push_str("    <div class='header'>\n");
        html.push_str("        <div class='title'>XLSX 스프레드시트</div>\n");
        html.push_str("        <div class='metadata'>\n");
        html.push_str(&format!("            파일 크기: {:.1} KB | 시트 수: {} | 생성일: {}\n", 
            metadata.file_size as f64 / 1024.0, sheets.len(), metadata.created_at));
        html.push_str("        </div>\n");
        html.push_str("    </div>\n");
        
        // 각 시트별 테이블
        for sheet in sheets {
            html.push_str(&format!("    <div class='sheet'>\n"));
            html.push_str(&format!("        <div class='sheet-title'>{}</div>\n", self.escape_html(&sheet.name)));
            html.push_str("        <table>\n");
            
            for (row_idx, row) in sheet.rows.iter().enumerate() {
                html.push_str("            <tr>\n");
                for cell in row {
                    if row_idx == 0 {
                        // 첫 번째 행은 헤더로 처리
                        html.push_str(&format!("                <th>{}</th>\n", self.escape_html(cell)));
                    } else {
                        html.push_str("                <td>\n");
                        html.push_str(&format!("                    <input type='text' class='editable-cell' value='{}' />\n", 
                            self.escape_html(cell)));
                        html.push_str("                </td>\n");
                    }
                }
                html.push_str("            </tr>\n");
            }
            
            html.push_str("        </table>\n");
            html.push_str("    </div>\n");
        }
        
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        
        Ok(html)
    }

    // PDF에서 이미지 추출 함수
    async fn extract_pdf_images(&self, file_path: &str) -> Result<Vec<String>, String> {
        use std::fs;
        
        // images 디렉토리 생성
        let images_dir = "./images";
        if let Err(_) = fs::create_dir_all(images_dir) {
            return Err("images 디렉토리 생성 실패".to_string());
        }
        
        // pdfimages 명령어로 이미지 추출
        let _file_stem = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
            
        let prefix = format!("{}/page", images_dir);
        
        match Command::new("pdfimages")
            .arg("-png")  // PNG 형식으로 추출
            .arg(file_path)
            .arg(&prefix)
            .output() {
            Ok(output) => {
                if output.status.success() {
                    // 추출된 이미지 파일 목록 가져오기
                    let mut image_files = Vec::new();
                    
                    if let Ok(entries) = fs::read_dir(images_dir) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let path = entry.path();
                                if let Some(file_name) = path.file_name() {
                                    if let Some(name_str) = file_name.to_str() {
                                        if name_str.starts_with("page") && name_str.ends_with(".png") {
                                            image_files.push(name_str.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // 파일명으로 정렬
                    image_files.sort();
                    Ok(image_files)
                } else {
                    // pdfimages 실패 시 빈 벡터 반환
                    println!("pdfimages 실행 실패, 이미지 추출 건너뜀");
                    Ok(Vec::new())
                }
            }
            Err(e) => {
                println!("pdfimages 명령어 실행 오류: {:?}", e);
                Ok(Vec::new())
            }
        }
    }

    async fn generate_pdf_html(
    &self, 
    text: &str, 
    metadata: &DocumentMetadata, 
    images: &Vec<String>,
    thumbnail: &Option<String>
) -> Result<String, String> {
    let paragraphs: Vec<&str> = text.split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();

    let mut html = String::new();
    html.push_str(r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>PDF 미리보기</title>
  <style>
    body { font-family: "Malgun Gothic", Arial, sans-serif; margin: 2em; background: #fff; }
    p { margin: 1em 0; }
    img { max-width: 100%; display: block; margin: 1em 0; }
  </style>
</head>
<body>
"#);
    // 문단 먼저 출력
    for paragraph in paragraphs {
        html.push_str(&format!("<p>{}</p>\n", self.escape_html(paragraph.trim()).replace("\n", "<br>")));
    }
    // 이미지 삽입
    for image_file in images {
        html.push_str(&format!("<img src=\"./images/{}\" alt=\"추출 이미지\">\n", image_file));
    }
    html.push_str("</body>\n</html>\n");
    Ok(html)
}


// escape_html 함수 정의
fn escape_html(&self, text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

}
