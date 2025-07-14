<script>
import { onMount, onDestroy, tick } from 'svelte';
import { Splitpanes, Pane } from 'svelte-splitpanes';
import * as monaco from 'monaco-editor';
import * as pdfjsLib from 'pdfjs-dist';
import JSZip from 'jszip';

// Monaco Editor 워커 환경 설정
if (typeof window !== 'undefined') {
	window.MonacoEnvironment = {
		getWorkerUrl: function (moduleId, label) {
			if (label === 'html' || label === 'handlebars' || label === 'razor') {
				return './monaco-editor/esm/vs/language/html/html.worker.js';
			}
			if (label === 'css' || label === 'scss' || label === 'less') {
				return './monaco-editor/esm/vs/language/css/css.worker.js';
			}
			if (label === 'javascript' || label === 'typescript') {
				return './monaco-editor/esm/vs/language/typescript/ts.worker.js';
			}
			return './monaco-editor/esm/vs/editor/editor.worker.js';
		}
	};
}

let showHtmlModal = false;
let pdfCanvas; // PDF 뷰어에서 <canvas bind:this={pdfCanvas}> 바인딩용
let pdfDocument = null; // PDF.js 문서 객체 (전역 선언)

// === Svelte template에서 사용하는 주요 상태 변수 선언 (IDE 경고 방지) ===
let pollingInterval = null;
let uploading = false;
let files = [];
let errorMessage = '';
let error = null;
let currentJob = null;
let jobStatus = null;
let jobResult = null;
let selectedJobId = null;
let jobHistory = [];
let activeTab = 'preview';
let pdfPageNum = 1;
let pdfPageCount = 1;
// === ===
let htmlModalElement;

// 2차 구현: HTML 미리보기 모달 고급 기능 상태
let htmlModalMode = 'original'; // 'original' | 'edited' | 'diff'
let monacoHtmlOriginalContainer, monacoHtmlEditedContainer, monacoHtmlDiffContainer;
let monacoHtmlOriginalEditor = null, monacoHtmlEditedEditor = null, monacoHtmlDiffEditor = null;
let originalHtmlValue = '';
let editedHtmlValue = '';
let htmlDiffModel = null;

// 모달이 열릴 때 원본/수정본 값 초기화
$: if (showHtmlModal && jobResult?.result?.content) {
	// 원본 값: HTML 코드가 string 형태로 들어가도록 보장
	let htmlRaw = jobResult.result.content.html || jobResult.result.content.text || '';
	
	// 만약 htmlRaw가 단순 텍스트라면 HTML로 변환
	// HTML태그가 없거나 공백문자로만 이루어져 있으면 변환 필요
	if (htmlRaw && (!htmlRaw.includes('<') || htmlRaw.trim().length === 0)) {
		// generateEnhancedHtml 함수를 사용하여 텍스트를 HTML로 변환
		console.log('[HTML Modal DEBUG] 텍스트를 HTML로 변환 시도');
		htmlRaw = generateEnhancedHtml({ text: htmlRaw });
	}
	
	originalHtmlValue = typeof htmlRaw === 'string' ? htmlRaw : '';
	
	// 디버깅: 실제로 설정되는 값 확인
	console.log('[HTML Modal DEBUG] jobResult.result.content:', jobResult.result.content);
	console.log('[HTML Modal DEBUG] htmlRaw (before processing):', jobResult.result.content.html || jobResult.result.content.text || '');
	console.log('[HTML Modal DEBUG] htmlRaw (after processing):', htmlRaw);
	console.log('[HTML Modal DEBUG] originalHtmlValue:', originalHtmlValue);
	console.log('[HTML Modal DEBUG] originalHtmlValue.length:', originalHtmlValue.length);
	console.log('[HTML Modal DEBUG] Is HTML content?:', originalHtmlValue.includes('<') && originalHtmlValue.includes('>'));
	
	// 추가 디버깅: 원본 텍스트 데이터 샘플 확인
	const originalText = jobResult.result.content.text || '';
	console.log('[HTML Modal DEBUG] 원본 텍스트 데이터 샘플 (100자):', originalText.substring(0, 100));
	console.log('[HTML Modal DEBUG] 전체 텍스트 길이:', originalText.length);
	
	// 표 인식 테스트
	const lines = originalText.split('\n');
	const sampleLines = lines.slice(0, 10);
	console.log('[HTML Modal DEBUG] 처음 10줄 샘플:', sampleLines);
	
	// 표 인식 테스트 시연
	sampleLines.forEach((line, idx) => {
		if (line.trim() !== '') {
			const hasTab = line.includes('\t');
			const hasMultipleSpaces = /\s{2,}/.test(line);
			const hasPipe = line.includes('|');
			console.log(`[HTML Modal DEBUG] 줄 ${idx}: 탭=${hasTab}, 다중공백=${hasMultipleSpaces}, 파이프=${hasPipe}, 내용: "${line}"`);
		}
	});
	
	// 수정본 값: 이전에 편집한 값이 있으면 유지, 없으면 원본 복사
	if (!editedHtmlValue) {
		editedHtmlValue = originalHtmlValue;
	}
	// 에디터가 항상 최신 값을 반영하도록 dispose 후 mount
	disposeHtmlEditors();
	tick().then(() => {
		mountHtmlEditors();
	});
}

// 모달이 닫힐 때 에디터 dispose 및 값 초기화
$: if (!showHtmlModal) {
	disposeHtmlEditors();
	htmlModalMode = 'original';
}

// 모드 전환 시 에디터 mount/dispose
$: if (showHtmlModal) {
	tick().then(() => {
		mountHtmlEditors();
	});
}

function mountHtmlEditors() {
	// 원본 에디터 (읽기 전용)
	if (htmlModalMode === 'original' && monacoHtmlOriginalContainer && !monacoHtmlOriginalEditor) {
		// 디버깅: 에디터에 전달되는 값 확인
		console.log('[MONACO DEBUG] Creating original editor with value:', originalHtmlValue);
		console.log('[MONACO DEBUG] Value length:', originalHtmlValue.length);
		console.log('[MONACO DEBUG] Value preview:', originalHtmlValue.substring(0, 200) + '...');
		
		monacoHtmlOriginalEditor = monaco.editor.create(monacoHtmlOriginalContainer, {
			value: originalHtmlValue,
			language: 'html', // HTML 코드 하이라이팅
			readOnly: true,
			theme: 'vs-light',
			automaticLayout: true,
			minimap: { enabled: false },
			fontSize: 15,
			lineNumbers: 'on',
			wordWrap: 'on',
		});
		
		// 디버깅: 에디터 생성 후 실제 값 확인
		console.log('[MONACO DEBUG] Editor created, getting value:', monacoHtmlOriginalEditor.getValue());
	}
	// 수정본 에디터
	if (htmlModalMode === 'edited' && monacoHtmlEditedContainer && !monacoHtmlEditedEditor) {
		monacoHtmlEditedEditor = monaco.editor.create(monacoHtmlEditedContainer, {
			value: editedHtmlValue,
			language: 'html',
			readOnly: false,
			theme: 'vs-light',
			automaticLayout: true,
			minimap: { enabled: false },
			fontSize: 15,
			lineNumbers: 'on',
		});
		monacoHtmlEditedEditor.onDidChangeModelContent(() => {
			editedHtmlValue = monacoHtmlEditedEditor.getValue();
		});
	}
	// Diff Editor
	if (htmlModalMode === 'diff' && monacoHtmlDiffContainer && !monacoHtmlDiffEditor) {
		const originalModel = monaco.editor.createModel(originalHtmlValue, 'html');
		const editedModel = monaco.editor.createModel(editedHtmlValue, 'html');
		htmlDiffModel = { original: originalModel, modified: editedModel };
		monacoHtmlDiffEditor = monaco.editor.createDiffEditor(monacoHtmlDiffContainer, {
			enableSplitViewResizing: true,
			readOnly: true,
			theme: 'vs-light',
			automaticLayout: true,
			fontSize: 15,
			lineNumbers: 'on',
			minimap: { enabled: false },
		});
		monacoHtmlDiffEditor.setModel(htmlDiffModel);
	}
}

function disposeHtmlEditors() {
	if (monacoHtmlOriginalEditor) {
		monacoHtmlOriginalEditor.dispose();
		monacoHtmlOriginalEditor = null;
	}
	if (monacoHtmlEditedEditor) {
		monacoHtmlEditedEditor.dispose();
		monacoHtmlEditedEditor = null;
	}
	if (monacoHtmlDiffEditor) {
		monacoHtmlDiffEditor.dispose();
		monacoHtmlDiffEditor = null;
		if (htmlDiffModel) {
			htmlDiffModel.original.dispose();
			htmlDiffModel.modified.dispose();
			htmlDiffModel = null;
		}
	}
}

function modalFocus(node) {
	if (showHtmlModal) {
		node.focus();
		// ESC 키로 모달 닫기
		const handleKeydown = (event) => {
			if (event.key === 'Escape') {
				showHtmlModal = false;
			}
		};
		window.addEventListener('keydown', handleKeydown);
		return {
			destroy() {
				window.removeEventListener('keydown', handleKeydown);
			}
		};
	}
}

// 텍스트를 원본 문서 구조를 유지하는 HTML로 변환 (표, 이미지, 레이아웃 포함)
function textToHtml(text) {
	if (!text) return '';
	
	// HTML 이스케이프 처리
	let html = text
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;");
	
	// 줄별로 분리하여 처리
	const lines = html.split('\n');
	const processedLines = [];
	let inList = false;
	let inTable = false;
	let tableRows = [];
	
	// 표 구조 감지 함수 - 개선버전
	function isTableRow(line) {
		// 탭 구분 데이터 감지 
		const tabParts = line.split('\t');
		const tabSeparated = tabParts.length > 2 && tabParts.filter(p => p.trim() !== '').length > 1;

		// 두 개 이상의 공백으로 구분된 데이터 감지
		const spaceParts = line.split(/\s{2,}/);
		const spaceSeparated = spaceParts.length > 2 && spaceParts.filter(p => p.trim() !== '').length > 1;

		// 파이프 기호로 구분된 데이터 감지
		const pipeParts = line.split('|');
		const hasPipeSymbol = pipeParts.length > 2 && pipeParts.filter(p => p.trim() !== '').length > 1;

		// 정규적인 간격으로 배열된 데이터 감지 (표 형태)
		const hasRegularSpacing = /\S+\s{2,}\S+\s{2,}\S+/.test(line);

		const isTable = tabSeparated || spaceSeparated || hasPipeSymbol || hasRegularSpacing;
		if (isTable) {
			console.log('[TABLE DEBUG] 표 행 발견:', line, { tabSeparated, spaceSeparated, hasPipeSymbol, hasRegularSpacing });
		}
		return isTable;
	}
	
	// 표 행 처리 함수 - 개선버전
	function processTableRow(line) {
		console.log('표 행 처리:', line);
		let cells = [];

		// 파이프 기호로 구분된 경우 처리
		if (line.includes('|')) {
			cells = line.split('|')
				.map(cell => cell.trim())
				.filter(cell => cell.length > 0);
			console.log('파이프 구분 셀:', cells);
		} 
		// 탭으로 구분된 경우 처리
		else if (line.includes('\t')) {
			cells = line.split('\t')
				.map(cell => cell.trim())
				.filter(cell => cell.length > 0);
			console.log('탭 구분 셀:', cells);
		} 
		// 두 개 이상의 공백으로 구분된 경우 처리
		else {
			cells = line.split(/\s{2,}/)
				.map(cell => cell.trim())
				.filter(cell => cell.length > 0);
			console.log('공백 구분 셀:', cells);
		}

		// 셀 데이터 검증
		if (cells.length < 2) {
			console.warn('표 행 처리 결과 셀이 부족함:', cells.length);
		}

		return cells;
	}
	
	// 표 완료 처리 함수 - 개선버전
	function finishTable() {
		if (tableRows.length > 0) {
			console.log('표 생성 시작:', tableRows.length, '행');
			
			// 모든 행의 셀 수가 다를 수 있으므로 최대 셀 수 찾기
			const maxCells = Math.max(...tableRows.map(row => row.length));
			console.log('표의 최대 셀 수:', maxCells);
			
			// 표 시작
			processedLines.push('<table border="1" style="border-collapse: collapse; width: 100%; margin: 1rem 0;">');
			
			// 표 행 처리
			tableRows.forEach((row, index) => {
				const isHeader = index === 0;
				const tag = isHeader ? 'th' : 'td';
				const style = isHeader ? 'background-color: #f5f5f5; font-weight: bold;' : '';
				
				// 행 시작
				processedLines.push('<tr>');
				
				// 부족한 셀 채우기 (maxCells로 통일)
				for (let i = 0; i < maxCells; i++) {
					const cellContent = i < row.length ? row[i] : '';
					processedLines.push(`<${tag} style="padding: 8px; border: 1px solid #ddd; ${style}">${cellContent}</${tag}>`);
				}
				
				// 행 끝
				processedLines.push('</tr>');
			});
			
			// 표 끝
			processedLines.push('</table>');
			tableRows = [];
			console.log('표 생성 완료');
		}
		inTable = false;
	}
	
	for (let i = 0; i < lines.length; i++) {
		const line = lines[i].trim();
		const nextLine = i + 1 < lines.length ? lines[i + 1].trim() : '';
		
		// 빈 줄 처리
		if (line === '') {
			if (inTable) finishTable();
			processedLines.push('<br>');
			continue;
		}

		// 표 행 처리
		if (isTableRow(line)) {
			const cells = processTableRow(line);
			if (cells.length > 1) {
				if (!inTable) {
					inTable = true;
				}
				tableRows.push(cells);
				continue;
			}
		} else if (inTable) {
			finishTable();
		}
		
		// 메인 제목 (1줄이고 다음줄이 비어있거나 짧을 때)
		if (i === 0 || (line.length > 10 && line.length < 50 && nextLine === '')) {
			if (inList) {
				processedLines.push(`</${inList}>`);
				inList = false;
			}
			processedLines.push(`<h1 style="color: #333; margin-top: 2rem; margin-bottom: 1rem; font-size: 1.5em; font-weight: bold;">${line}</h1>`);
			continue;
		}
		
		// 목차 제목
		if (line.match(/^(목차|Table of Contents|정의|Contents)$/i)) {
			if (inList) {
				processedLines.push(`</${inList}>`);
				inList = false;
			}
			processedLines.push(`<h2 style="color: #555; margin-top: 1.5rem; margin-bottom: 0.5rem; font-size: 1.3em; font-weight: bold;">${line}</h2>`);
			continue;
		}
		
		// 숫자 리스트 (1. 2. 3. 또는 1) 2) 3) 형태)
		if (line.match(/^[0-9]+[\.\)]\s+.+/)) {
			if (inList !== 'ol') {
				if (inList) processedLines.push(`</${inList}>`);
				processedLines.push('<ol style="margin: 0.5rem 0; padding-left: 2rem;">');
				inList = 'ol';
			}
			const content = line.replace(/^[0-9]+[\.\)]\s+/, '');
			processedLines.push(`<li style="margin-bottom: 0.25rem;">${content}</li>`);
			continue;
		}
			
		// 불릿 리스트 (- 또는 • 시작)
		if (line.match(/^[\-•]\s+.+/)) {
			if (inList !== 'ul') {
				if (inList) processedLines.push(`</${inList}>`);
				processedLines.push('<ul style="margin: 0.5rem 0; padding-left: 2rem;">');
				inList = 'ul';
			}
			const content = line.replace(/^[\-•]\s+/, '');
			processedLines.push(`<li style="margin-bottom: 0.25rem;">${content}</li>`);
			continue;
		}
		
		// 소제목 (숫자.숫자 형태 또는 짧은 제목성 라인)
		if (line.match(/^[0-9]+\.[0-9]+/) || (line.length < 40 && line.match(/^[A-Za-z가-힣].*[가-힣A-Za-z]$/))) {
			if (inList) {
				processedLines.push(`</${inList}>`);
				inList = false;
			}
			processedLines.push(`<h3 style="color: #666; margin-top: 1rem; margin-bottom: 0.5rem; font-size: 1.1em; font-weight: bold;">${line}</h3>`);
			continue;
		}
		
		// 일반 문단
		if (inList) {
			processedLines.push(`</${inList}>`);
			inList = false;
		}
		processedLines.push(`<p style="margin: 0.5rem 0; line-height: 1.6; text-align: justify;">${line}</p>`);
	}
	
	// 마지막에 열린 구조 닫기
	if (inTable) {
		finishTable();
	}
	if (inList) {
		processedLines.push(`</${inList}>`);
	}
	
	return processedLines.join('\n');
	}
	
	// 안정적인 HTML 생성 (회색 화면 문제 해결 최우선)
	function generateEnhancedHtml(content) {
		console.log('[DEBUG] generateEnhancedHtml 진입', content);
		try {
			if (!content || !content.text) {
				console.log('[DEBUG] generateEnhancedHtml: 내용 없음');
				return '내용이 없습니다.';
			}

			console.log('[DEBUG] 텍스트 길이:', content.text.length, '줄 수:', content.text.split('\n').length);
			// 프리징 방지: 너무 큰 텍스트/줄수 제한
			if (content.text.length > 100000 || content.text.split('\n').length > 2000) {
				console.warn('[DEBUG] 텍스트가 너무 커서 미리보기 불가');
				return '<div style="color:#a00;padding:2em;text-align:center;font-weight:bold;">HTML 미리보기 불가: 문서가 너무 큽니다.<br>100KB 이하, 2000줄 이하의 텍스트만 미리보기가 지원됩니다.</div>';
			}
			
			// textToHtml 함수를 사용하여 더 고급 HTML 변환
			console.log('[generateEnhancedHtml DEBUG] textToHtml 호출 전, content.text:', content.text.substring(0, 200));
			const enhancedHtml = textToHtml(content.text);
			console.log('[generateEnhancedHtml DEBUG] textToHtml 결과:', enhancedHtml.substring(0, 500));
			console.log('[generateEnhancedHtml DEBUG] HTML에 표가 포함되어 있나?:', enhancedHtml.includes('<table'));
			
			let result = '<div style="font-family: system-ui, -apple-system, sans-serif; line-height: 1.5;">';
			result += enhancedHtml;
			
			// 이미지 삽입 (썸네일이 있는 경우) - 텍스트 뒤에 추가
			if (content.thumbnail && content.thumbnail.startsWith('data:image/')) {
				try {
					// 이미지 크기 제한
					result += `<div style="margin-top:1rem;text-align:center;">
						<img 
							src="${content.thumbnail}" 
							alt="문서 썸네일" 
							style="max-width:400px;max-height:250px;border:1px solid #ddd;" 
							loading="lazy"
						/>
					</div>`;
				} catch (imgError) {
					console.warn('이미지 처리 오류:', imgError);
				}
			}
			
			result += '</div>';
			return result;
		} catch (error) {
			console.error('HTML 생성 오류:', error);
			// 오류 발생 시 가장 기본적인 HTML로 폴백
			return '<div style="white-space:pre-wrap;font-family:monospace;">' + 
				(content?.text || '')
					.replace(/&/g, '&amp;')
					.replace(/</g, '&lt;')
					.replace(/>/g, '&gt;')
					.replace(/\n/g, '<br>') + 
				'</div>';
		}
	}
	
	// 간단한 HTML 생성 (폴백용)
	function generateSimpleHtml(content) {
		try {
			if (!content || !content.text) return '내용이 없습니다.';
			
			// 원본 텍스트를 그대로 표시
			const text = content.text || '';
			
			// 줄바꿈을 <br>로 변환
			const htmlText = text
				.replace(/&/g, '&amp;')
				.replace(/</g, '&lt;')
				.replace(/>/g, '&gt;')
				.replace(/\n/g, '<br>');
			
			return '<div style="white-space:pre-wrap;font-family:monospace;">' + htmlText + '</div>';
		} catch (error) {
			console.error('HTML 생성 오류:', error);
			return '오류 발생';
		}
	}
	
	// HTML 이스케이프 유틸리티 함수
	function escapeHtml(text) {
		return text
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;')
			.replace(/"/g, '&quot;')
			.replace(/'/g, '&#039;');
	}

	// 대용량 base64 필드 등은 생략해서 표시
	function filterLargeFields(obj) {
		if (!obj || typeof obj !== 'object') return obj;
		const filtered = Array.isArray(obj) ? [] : {};
		for (const key in obj) {
			if (typeof obj[key] === 'string' && obj[key].length > 300) {
				filtered[key] = `[생략: ${key} (${obj[key].length} bytes)]`;
			} else if (typeof obj[key] === 'object') {
				filtered[key] = filterLargeFields(obj[key]);
			} else {
				filtered[key] = obj[key];
			}
		}
		return filtered;
	}

	// PDF 뷰어 초기화
	function initPdfViewer() {
		// PDF.js Worker 설정 - 안정적인 3.11 버전 사용
		pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js';
		
		// 버전 체크 비활성화 - 글로벌 설정
		if (pdfjsLib.GlobalWorkerOptions) {
			pdfjsLib.GlobalWorkerOptions.disableWorker = true;
		}
		
		// 로그 비활성화
		if (pdfjsLib.VerbosityLevel) {
			pdfjsLib.VerbosityLevel.ERRORS = 0;
		}
		
		console.log('PDF.js Worker 설정 완료 - 버전 체크 비활성화');
	}

	// Monaco 에디터 초기화
	function initJsonEditor() {
		if (jsonEditorContainer && !jsonEditor && parsedContent) {
			try {
				const editorContent = typeof parsedContent.data === 'string' ? parsedContent.data : JSON.stringify(parsedContent.data, null, 2);
				jsonEditor = monaco.editor.create(jsonEditorContainer, {
					value: editorContent,
					language: 'json',
					theme: 'vs',
					automatic: true,
					readOnly: true,
					minimap: { enabled: false },
					scrollBeyondLastLine: false,
					fontSize: 14
				});
			} catch (e) {
				console.error('JSON 에디터 초기화 오류:', e);
			}
		}
	}

	// PDF 로드 및 렌더링
	async function loadPdf(pdfBytes) {
		if (!pdfBytes) {
			console.warn('PDF 바이트 데이터 없음');
			return;
		}
		console.log('PDF 로딩 시작, 데이터 크기:', pdfBytes.length);
		try {
			const loadingTask = pdfjsLib.getDocument({
				data: pdfBytes,
				disableWorker: true,
				isEvalSupported: false,
				verbosity: 0,
				standardFontDataUrl: null
			});
			pdfDocument = await loadingTask.promise;
			pdfPageCount = pdfDocument.numPages;
			pdfPageNum = 1;
			console.log('PDF 로딩 완료, 페이지 수:', pdfPageCount);
			await tick(); // 캔버스가 DOM에 바인딩된 후 렌더링 보장
			console.log('DOM tick 완료, 캔버스 엘리먼트:', pdfCanvas);
			renderPdfPage();
		} catch (e) {
			console.error('PDF 로드 오류:', e);
		}
	}

	// PDF 페이지 렌더링
	async function renderPdfPage() {
		// DOM이 업데이트될 때까지 기다림
		await tick();
		
		if (!pdfDocument) {
			console.warn('PDF 렌더링 불가: pdfDocument 없음');
			return;
		}
		
		if (!pdfCanvas) {
			console.warn('PDF 렌더링 불가: pdfCanvas 없음');
			return;
		}
		
		try {
			console.log('PDF 페이지', pdfPageNum, '렌더링 시작');
			const page = await pdfDocument.getPage(pdfPageNum);
			const viewport = page.getViewport({ scale: 1.5 });
			
			// 캔버스 컨텍스트가 유효한지 다시 확인
			if (!pdfCanvas) {
				console.error('PDF 렌더링 오류: 캔버스 요소가 null');
				return;
			}
			
			const context = pdfCanvas.getContext('2d');
			if (!context) {
				console.error('PDF 렌더링 오류: 캔버스 컨텍스트를 가져올 수 없음');
				return;
			}
			
			pdfCanvas.height = viewport.height;
			pdfCanvas.width = viewport.width;
			console.log('캔버스 크기 설정:', pdfCanvas.width, 'x', pdfCanvas.height);
			
			// 렌더링 실행
			await page.render({ canvasContext: context, viewport }).promise;
			console.log('PDF 페이지 렌더링 완료');
		} catch (e) {
			console.error('PDF 페이지 렌더링 오류:', e);
		}
	}

	// 탭 변경 처리
	function changeTab(tab) {
		activeTab = tab;
		
		// JSON 탭이 선택되면 에디터 초기화
		if (tab === 'json' && jobResult) {
			setTimeout(() => {
				initJsonEditor();
			}, 100);
		}
	}
	
	// 원본 파일 다운로드
	function downloadOriginalFile() {
		if (!jobResult || !jobResult.original_file) {
			alert('다운로드할 원본 파일이 없습니다.');
			return;
		}
		
		try {
			// 파일 이름 생성
			const filename = jobResult.filename || `original-${jobResult.job_id}.${jobResult.file_type.toLowerCase()}`;
			
			// base64 데이터를 Blob으로 변환
			const byteCharacters = atob(jobResult.original_file);
			const byteArrays = [];
			
			for (let offset = 0; offset < byteCharacters.length; offset += 512) {
				const slice = byteCharacters.slice(offset, offset + 512);
				
				const byteNumbers = new Array(slice.length);
				for (let i = 0; i < slice.length; i++) {
					byteNumbers[i] = slice.charCodeAt(i);
				}
				
				const byteArray = new Uint8Array(byteNumbers);
				byteArrays.push(byteArray);
			}
			
			const blob = new Blob(byteArrays);
			
			// 다운로드 링크 생성 및 클릭
			const link = document.createElement('a');
			link.href = URL.createObjectURL(blob);
			link.download = filename;
			link.click();
			
			// 메모리 해제
			setTimeout(() => {
				URL.revokeObjectURL(link.href);
			}, 100);
		} catch (e) {
			console.error('파일 다운로드 오류:', e);
			alert('파일 다운로드 중 오류가 발생했습니다.');
		}
	}
	
	// 파싱 결과 다운로드
	function downloadParsedResult() {
		if (!jobResult || !jobResult.content) {
			alert('다운로드할 파싱 결과가 없습니다.');
			return;
		}
		
		try {
			// 파일 이름 생성
			const baseFilename = jobResult.filename ? jobResult.filename.split('.')[0] : `result-${jobResult.job_id}`;
			let content;
			let filename;
			let type;
			
			// 현재 활성화된 탭에 따라 다른 형식으로 다운로드
			switch (activeTab) {
				case 'json':
					// Monaco 에디터가 있으면 에디터의 값을, 없으면 원본 content 사용
					if (jsonEditor) {
						content = jsonEditor.getValue();
					} else {
						content = JSON.stringify(jobResult.result.content, null, 2);
					}
					filename = `${baseFilename}.json`;
					type = 'application/json';
					break;
					
				case 'raw':
					content = JSON.stringify(jobResult, null, 2);
					filename = `${baseFilename}-raw.json`;
					type = 'application/json';
					break;
					
				default: // preview
					if (parsedContent && parsedContent.type === 'html') {
						content = parsedContent.data;
						filename = `${baseFilename}.html`;
						type = 'text/html';
					} else {
						content = parsedContent ? parsedContent.data : JSON.stringify(jobResult.result.content, null, 2);
						filename = `${baseFilename}.txt`;
						type = 'text/plain';
					}
					break;
			}
			
			// Blob 생성 및 다운로드
			const blob = new Blob([content], { type });
			const link = document.createElement('a');
			link.href = URL.createObjectURL(blob);
			link.download = filename;
			link.click();
			
			// 메모리 해제
			setTimeout(() => {
				URL.revokeObjectURL(link.href);
			}, 100);
		} catch (e) {
			console.error('결과 다운로드 오류:', e);
			alert('결과 다운로드 중 오류가 발생했습니다.');
		}
	}

	// HTML 및 이미지 다운로드 함수
	async function downloadHtmlWithImages() {
		// HTML 콘텐츠: 에디터가 열려 있으면 그 값을, 아니면 jobResult.result.html 사용
		let htmlContent = '';
		if (showHtmlModal) {
			if (htmlModalMode === 'edited' && monacoHtmlEditedEditor) {
				htmlContent = monacoHtmlEditedEditor.getValue();
			} else if (htmlModalMode === 'original' && monacoHtmlOriginalEditor) {
				htmlContent = monacoHtmlOriginalEditor.getValue();
			} else if (htmlModalMode === 'diff' && monacoHtmlDiffEditor) {
				// diff 에디터는 수정본 기준 value 사용
				const model = monacoHtmlDiffEditor.getModel();
				if (model && model.modified) {
					htmlContent = model.modified.getValue();
				}
			}
		}
		if (!htmlContent && jobResult && jobResult.result && jobResult.result.html) {
			htmlContent = jobResult.result.html;
		}
		if (!htmlContent || htmlContent.trim() === '') {
			alert('다운로드할 HTML 콘텐츠가 없습니다.');
			return;
		}
		
		try {
			// ZIP 파일 생성
			const zip = new JSZip();
			
			// HTML 콘텐츠는 이미 위에서 설정됨
			
			// HTML 파일을 ZIP에 추가
			const baseFilename = jobResult.filename ? jobResult.filename.split('.')[0] : `document-${jobResult.job_id}`;
			zip.file(`${baseFilename}.html`, htmlContent);
			
			// HTML에서 이미지 참조 찾기
			const imagePattern = /src="\.\/images\/([^"]+)"/g;
			const imageFiles = new Set();
			let match;
			
			while ((match = imagePattern.exec(htmlContent)) !== null) {
				imageFiles.add(match[1]);
			}
			
			// 이미지 파일들을 서버에서 가져와서 ZIP에 추가
			if (imageFiles.size > 0) {
				const imagesFolder = zip.folder('images');
				
				for (const imageFile of imageFiles) {
					try {
						// 서버에서 이미지 파일 가져오기
						const imageResponse = await fetch(`http://localhost:8080/images/${imageFile}`);
						if (imageResponse.ok) {
							const imageBlob = await imageResponse.blob();
							imagesFolder.file(imageFile, imageBlob);
							console.log(`이미지 추가됨: ${imageFile}`);
						} else {
							console.warn(`이미지를 찾을 수 없음: ${imageFile}`);
						}
					} catch (imgError) {
						console.error(`이미지 생성 오류 (${imageFile}):`, imgError);
					}
				}
			}
			
			// README 파일 추가
			const readmeContent = `# ${baseFilename}

이 파일들은 Document Parser System에서 생성된 HTML 미리보기 파일입니다.

## 파일 구성
- ${baseFilename}.html: 메인 HTML 파일
- images/: 추출된 이미지 파일들

## 사용 방법
1. 모든 파일을 동일한 폴더에 압축 해제합니다.
2. ${baseFilename}.html 파일을 웹 브라우저로 엽니다.
3. HTML 파일을 자유롭게 편집할 수 있습니다.

생성 시간: ${new Date().toLocaleString('ko-KR')}
`;
			zip.file('README.md', readmeContent);
			
			// ZIP 파일 생성 및 다운로드
			const zipBlob = await zip.generateAsync({ type: 'blob' });
			
			// 다운로드 링크 생성 및 클릭
			const link = document.createElement('a');
			link.href = URL.createObjectURL(zipBlob);
			link.download = `${baseFilename}-html-preview.zip`;
			link.click();
			
			// 메모리 해제
			setTimeout(() => {
				URL.revokeObjectURL(link.href);
			}, 100);
			
			console.log('HTML 다운로드 완료:', `${baseFilename}-html-preview.zip`);
			
		} catch (error) {
			console.error('HTML 다운로드 오류:', error);
			alert('HTML 다운로드 중 오류가 발생했습니다.');
		}
	}

	// 로컬 스토리지에서 작업 히스토리 로드
	onMount(() => {
		try {
			const savedHistory = localStorage.getItem('jobHistory');
			if (savedHistory) {
				jobHistory = JSON.parse(savedHistory);
			}
		} catch (e) {
			console.error('작업 히스토리 로드 실패:', e);
		}
		
		// PDF 뷰어 초기화
		initPdfViewer();
		
		// 디버깅을 위해 전역 함수 노출
		window.fetchJobResult = fetchJobResult;
		window.jobHistory = jobHistory;
		window.jobStatus = jobStatus;
		window.jobResult = jobResult;
	});
	
	// 상태 변경 시 전역 변수 업데이트
	$: if (typeof window !== 'undefined') {
		window.jobStatus = jobStatus;
		window.jobResult = jobResult;
	}

	// 컴포넌트 제거 시 폴링 중지 및 리소스 정리
	onDestroy(() => {
		stopPolling();
		
		// Monaco 에디터 정리
		if (jsonEditor) {
			jsonEditor.dispose();
		}
		
		// PDF 문서 정리
		if (pdfDocument) {
			pdfDocument.destroy();
			pdfDocument = null;
		}
	});

	// 파일 업로드 처리
	async function handleUpload() {
		if (!files.length) return;
		
		uploading = true;
		error = null;
		jobStatus = null;
		jobResult = null;

		const formData = new FormData();
		formData.append('file', files[0]);

		try {
			const response = await fetch('http://localhost:8080/upload', {
				method: 'POST',
				body: formData
			});

			if (response.ok) {
				currentJob = await response.json();
				
				// 작업 히스토리에 추가
				const newJob = {
					job_id: currentJob.job_id,
					filename: files[0].name,
					filesize: files[0].size,
					filetype: files[0].type,
					created_at: new Date().toISOString()
				};
				
				jobHistory = [newJob, ...jobHistory].slice(0, 10); // 최대 10개 히스토리 유지
				localStorage.setItem('jobHistory', JSON.stringify(jobHistory));
				
				// 작업 상태 폴링 시작
				startPolling(currentJob.job_id);
				selectedJobId = currentJob.job_id;
			} else {
				error = `업로드 실패: ${response.status}`;
			}
		} catch (e) {
			error = `네트워크 오류: ${e.message}`;
		} finally {
			uploading = false;
		}
	}

	// 작업 상태 폴링 시작
	function startPolling(jobId) {
		// 이전 폴링이 있으면 중지
		stopPolling();
		
		// 즉시 한 번 상태 확인
		fetchJobStatus(jobId);
		
		// 2초마다 작업 상태 확인
		pollingInterval = setInterval(() => {
			fetchJobStatus(jobId);
		}, 2000);
	}

	// 폴링 중지
	function stopPolling() {
		if (pollingInterval) {
			clearInterval(pollingInterval);
			pollingInterval = null;
		}
	}

	// 작업 상태 가져오기
	async function fetchJobStatus(jobId) {
		try {
			const response = await fetch(`http://localhost:8080/jobs/${jobId}`);
			if (!response.ok) {
				if (response.status === 404) {
					errorMessage = '해당 작업을 찾을 수 없습니다. 목록에서 제거합니다.';
					// jobHistory에서 제거
					jobHistory = jobHistory.filter(job => job.job_id !== jobId);
					// 선택된 job이 삭제된 경우, 다른 job으로 자동 포커스
					if (selectedJobId === jobId) {
						selectedJobId = jobHistory.length > 0 ? jobHistory[0].job_id : null;
					}
				}
				console.error('작업 상태 가져오기 실패:', response.status);
				return;
			}
			// 200(성공)일 때만 정상 처리
			jobStatus = await response.json();
			console.log('작업 상태 업데이트:', jobStatus);
			// 작업 완료 시 폴링 중지
			if (jobStatus.status === 'completed' || jobStatus.status === 'Completed') {
				stopPolling();
				// 필요하다면 결과 가져오기 등 추가 로직
			}
			// 이후 로직...
		} catch (e) {
			console.error('작업 상태 확인 중 오류:', e);
		}
	}

	// 작업 결과 가져오기
	async function fetchJobResult(jobId) {
		try {
			console.log('작업 결과 요청:', jobId);
			const response = await fetch(`http://localhost:8080/jobs/${jobId}/result`);
			
			if (response.ok) {
				jobResult = await response.json();
				console.log('받은 작업 결과:', jobResult);
				
				// 파일 형식에 따라 파싱된 내용 처리
				if (jobResult && jobResult.result && jobResult.result.content) {
					console.log('파일 유형:', jobResult.result.file_type);
					const processedContent = processContent(jobResult.result.content, jobResult.result.file_type);
					console.log('처리된 내용:', processedContent);
					console.log('jobResult.result 전체 스크릭처:', JSON.stringify(jobResult.result, null, 2));
					console.log('jobResult.result?.file_type:', jobResult.result?.file_type);
					console.log('PDF 조건 검사:', jobResult.result?.file_type && jobResult.result.file_type.toLowerCase() === 'pdf');
					
					// PDF 파일인 경우 원본 문서 로드
					if (jobResult.result.file_type.toLowerCase() === 'pdf') {
						console.log('PDF 데이터 구조 확인:', jobResult.result.content);
						
						// 다양한 경로에서 PDF 데이터 찾기
						let pdfData = null;
						if (jobResult.result.content?.original_file) {
							pdfData = jobResult.result.content.original_file;
						} else if (jobResult.original_file) {
							pdfData = jobResult.original_file;
						} else if (jobResult.result.original_file) {
							pdfData = jobResult.result.original_file;
						}
						
						if (pdfData) {
							try {
								// Base64 → Uint8Array 변환
								const binary = atob(pdfData);
								const len = binary.length;
								const bytes = new Uint8Array(len);
								
								for (let i = 0; i < len; i++) {
									bytes[i] = binary.charCodeAt(i);
								}
								
								// PDF 로드
								setTimeout(() => {
									loadPdf(bytes);
								}, 100);
							} catch (e) {
								console.error('PDF 로드 오류:', e);
							}
						} else {
							console.log('PDF 데이터를 찾을 수 없습니다');
						}
					}
					
					// JSON 탭이 활성화된 경우 에디터 초기화
					if (activeTab === 'json') {
						setTimeout(() => {
							initJsonEditor();
						}, 100);
					}
				} else {
					console.log('작업 결과에 content가 없습니다:', jobResult);
					parsedContent = null;
				}
			} else {
				console.error('작업 결과 가져오기 실패:', response.status);
			}
		} catch (e) {
			console.error('작업 결과 확인 중 오류:', e);
		}
	}
	
	// 파일 형식에 따라 내용 처리
	function processContent(content, fileType) {
		console.log('processContent 호출됨:', { content, fileType });
		if (!content) {
			console.log('내용이 없습니다');
			return { type: 'text', data: '내용이 없습니다.' };
		}
		
		if (!fileType) {
			console.log('파일 형식이 없습니다');
			return { type: 'text', data: JSON.stringify(content, null, 2) };
		}
		
		// 파일 형식에 따라 다르게 처리
		console.log('파일 형식:', fileType.toLowerCase());
		switch(fileType.toLowerCase()) {
			case 'pdf':
				return {
					type: 'text',
					data: typeof content === 'string' ? content : JSON.stringify(content, null, 2)
				};
				
			case 'docx':
				return {
					type: 'text',
					data: typeof content === 'string' ? content : JSON.stringify(content, null, 2)
				};
				
			case 'xlsx':
				// 스프레드시트는 테이블로 보여주기 위해 HTML 형식으로 변환
				if (Array.isArray(content)) {
					let tableHtml = '<table class="excel-table">';
					
					// 헤더 추가 (첫 번째 행을 헤더로 간주)
					if (content.length > 0) {
						tableHtml += '<thead><tr>';
						for (const cell of content[0]) {
							tableHtml += `<th>${cell || ''}</th>`;
						}
						tableHtml += '</tr></thead>';
					}
					
					// 데이터 행 추가
					tableHtml += '<tbody>';
					for (let i = 1; i < content.length; i++) {
						tableHtml += '<tr>';
						for (const cell of content[i]) {
							tableHtml += `<td>${cell || ''}</td>`;
						}
						tableHtml += '</tr>';
					}
					tableHtml += '</tbody></table>';
					
					return {
						type: 'html',
						data: tableHtml
					};
				} else {
					return {
						type: 'text',
						data: JSON.stringify(content, null, 2)
					};
				}
				
			case 'txt':
				return {
					type: 'text',
					data: typeof content === 'string' ? content : JSON.stringify(content, null, 2)
				};
				
			case 'md':
				return {
					type: 'markdown',
					data: typeof content === 'string' ? content : JSON.stringify(content, null, 2)
				};
				
			default:
				return {
					type: 'json',
					data: JSON.stringify(content, null, 2)
				};
		}
	}

	// 히스토리에서 작업 선택
	function selectJob(jobId) {
		selectedJobId = jobId;
		jobStatus = null;
		jobResult = null;
		error = null;
		startPolling(jobId);
	}

	function handleFileSelect(event) {
		files = Array.from(event.target.files);
	}

	// 파일 크기 포맷팅
	function formatFileSize(bytes) {
		if (bytes < 1024) return bytes + ' B';
		else if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB';
		else return (bytes / 1048576).toFixed(1) + ' MB';
	}

	// 날짜 포맷팅
	function formatDate(dateString) {
		const date = new Date(dateString);
		return date.toLocaleString('ko-KR');
	}

</script>

<style>
.pdf-container {
  width: 100%;
  min-height: 400px;
  height: auto;
  background: #eee;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: 16px;
  overflow: auto;
  border: 2px solid #007bff;
  box-sizing: border-box;
}
.pdf-canvas {
  border: 3px solid #ff0000;
  background: #ffeeee;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  max-width: 100%;
  min-width: 300px;
  min-height: 400px;
  height: auto;
  display: block;
  margin: 0 auto;
}

	/* Splitpanes 스타일 */
	:global(.splitpanes) {
		display: flex;
		height: 100%;
		width: 100%;
	}
	
	:global(.splitpanes--vertical) {
		flex-direction: row;
	}
	
	:global(.splitpanes--horizontal) {
		flex-direction: column;
	}
	
	:global(.splitpanes__pane) {
		width: 100%;
		height: 100%;
		overflow: hidden;
	}
	
	:global(.splitpanes__splitter) {
		background-color: #ccc;
		position: relative;
		flex-shrink: 0;
	}
	
	:global(.splitpanes--vertical > .splitpanes__splitter) {
		width: 7px;
		border-left: 1px solid #ddd;
		border-right: 1px solid #ddd;
		cursor: col-resize;
	}
	
	:global(.splitpanes--horizontal > .splitpanes__splitter) {
		height: 7px;
		border-top: 1px solid #ddd;
		border-bottom: 1px solid #ddd;
		cursor: row-resize;
	}
	
	:global(body) {
		font-family: 'Pretendard', -apple-system, BlinkMacSystemFont, system-ui, Roboto, sans-serif;
		background-color: #f5f5f7;
		margin: 0;
		padding: 20px;
		color: #333;
	}

	h1 {
		text-align: center;
		color: #2c3e50;
		margin-bottom: 2rem;
	}

	h2, h3 {
		color: #2c3e50;
		margin-top: 0;
	}

	.container {
		display: flex;
		flex-direction: column;
		gap: 2rem;
		max-width: 1200px;
		margin: 0 auto;
		margin-top: 0;
	}

	.upload-section {
		background: white;
		padding: 1.5rem;
		border-radius: 8px;
		box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
	}

	.file-input {
		display: flex;
		gap: 1rem;
		align-items: center;
		margin-bottom: 1rem;
	}

	.upload-button {
		background-color: #3498db;
		color: white;
		border: none;
		padding: 0.5rem 1rem;
		border-radius: 4px;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.upload-button:hover {
		background-color: #2980b9;
	}

	.upload-button:disabled {
		background-color: #95a5a6;
		cursor: not-allowed;
	}

	.selected-file {
		margin-top: 0.5rem;
		font-size: 0.9rem;
	}

	.content-section {
		display: grid;
		grid-template-columns: 1fr 300px;
		gap: 1.5rem;
	}

	.job-status-section {
		background: white;
		padding: 1.5rem;
		border-radius: 8px;
		box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
	}

	.job-history-section {
		background: white;
		padding: 1.5rem;
		border-radius: 8px;
		box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
	}

	.error {
		background: #fee;
		border: 1px solid #fcc;
		padding: 1rem;
		margin: 1rem 0;
		border-radius: 4px;
	}

	.current-job, .job-status, .job-result {
		margin-bottom: 1.5rem;
	}

	.status {
		display: inline-block;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		font-size: 0.9rem;
		font-weight: bold;
	}

	.status.queued {
		background-color: #f1c40f;
		color: #7f6000;
	}

	.status.processing {
		background-color: #3498db;
		color: white;
	}

	.status.completed {
		background-color: #2ecc71;
		color: white;
	}

	.status.failed {
		background-color: #e74c3c;
		color: white;
	}

	.progress-container {
		margin: 1rem 0;
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.progress-bar {
		flex-grow: 1;
		height: 12px;
		background-color: #ecf0f1;
		border-radius: 6px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: #3498db;
		transition: width 0.3s ease;
	}

	.progress-text {
		font-size: 0.9rem;
		font-weight: bold;
		min-width: 40px;
	}

	.job-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		max-height: 400px;
		overflow-y: auto;
	}

	.job-item {
		padding: 0.75rem;
		border-radius: 4px;
		background-color: #f8f9fa;
		cursor: pointer;
		transition: background-color 0.2s;
		border-left: 3px solid transparent;
		text-align: left;
		display: block;
		width: 100%;
		border: none;
		font-family: inherit;
		font-size: inherit;
	}

	.job-item:hover {
		background-color: #e9ecef;
	}

	.job-item.selected {
		background-color: #e3f2fd;
		border-left-color: #3498db;
	}

	.job-item-header {
		display: flex;
		justify-content: space-between;
		margin-bottom: 0.25rem;
	}

	.job-filename {
		font-weight: bold;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 180px;
	}

	.job-item-details {
		display: flex;
		justify-content: space-between;
		font-size: 0.8rem;
		color: #7f8c8d;
	}

	.no-history {
		color: #95a5a6;
		text-align: center;
		padding: 1rem 0;
	}

	pre {
		white-space: pre-wrap;
		word-wrap: break-word;
		background-color: #f8f9fa;
		padding: 1rem;
		border-radius: 4px;
		border: 1px solid #e9ecef;
		max-height: 400px;
		overflow-y: auto;
		font-size: 0.9rem;
	}
	
	.result-metadata {
		margin-bottom: 1.5rem;
		padding: 1rem;
		background-color: #f8f9fa;
		border-radius: 4px;
		border: 1px solid #e9ecef;
	}
	
	.metadata-item {
		display: flex;
		margin-bottom: 0.5rem;
	}
	
	.metadata-item:last-child {
		margin-bottom: 0;
	}
	
	.label {
		font-weight: bold;
		width: 100px;
	}
	
	.result-content {
		margin-bottom: 1.5rem;
		overflow: hidden;
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	
	/* 분할 화면 스타일 */
	:global(.splitpanes) {
		height: 600px;
		background-color: #f5f5f7;
		border-radius: 6px;
		overflow: hidden;
	}
	
	:global(.splitpanes__pane) {
		background-color: white;
		padding: 15px;
		overflow: auto;
	}
	
	:global(.splitpanes__splitter) {
		background-color: #ddd;
		position: relative;
	}
	
	:global(.splitpanes__splitter:before) {
		content: '';
		position: absolute;
		left: 50%;
		top: 50%;
		transform: translate(-50%, -50%);
		background-color: #888;
		height: 30px;
		width: 4px;
		border-radius: 2px;
	}
	
	/* 탭 스타일 */
	.tabs {
		display: flex;
		border-bottom: 1px solid #ddd;
		margin-bottom: 15px;
	}
	
	.tab {
		padding: 10px 20px;
		cursor: pointer;
		border: 1px solid transparent;
		border-bottom: none;
		border-radius: 4px 4px 0 0;
		margin-right: 5px;
		transition: all 0.2s;
	}
	
	.tab:hover {
		background-color: #f0f0f0;
	}
	
	.tab.active {
		border-color: #ddd;
		background-color: white;
		margin-bottom: -1px;
	}
	
	/* PDF 뷰어 스타일 */
	.pdf-viewer {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 20px;
	}
	
	.pdf-controls {
		display: flex;
		justify-content: center;
		align-items: center;
		margin-bottom: 10px;
		gap: 10px;
	}
	
	.pdf-container {
		flex: 1;
		overflow: auto;
		background-color: #525659;
		display: flex;
		justify-content: center;
		padding: 20px;
	}
	
	.pdf-canvas {
		background-color: white;
		box-shadow: 0 0 10px rgba(0, 0, 0, 0.3);
	}
	
	.text-content, .json-content {
		max-height: 500px;
		overflow-y: auto;
		background-color: #fff;
		border: 1px solid #e9ecef;
		border-radius: 4px;
		padding: 1rem;
	}
	
	.raw-content {
		margin-top: 0.5rem;
		border: 1px solid #e9ecef;
		border-radius: 4px;
		overflow: hidden;
		padding: 1rem;
		max-height: 500px;
		overflow-y: auto;
	}
	
	.original-content {
		padding: 1rem;
		background-color: #f8f9fa;
		border: 1px solid #e9ecef;
		border-radius: 4px;
		max-height: 500px;
		overflow-y: auto;
	}
	
	.pdf-viewer {
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	
	.pdf-controls {
		display: flex;
		justify-content: center;
		align-items: center;
		gap: 1rem;
		padding: 0.5rem;
		background-color: #f8f9fa;
		border-bottom: 1px solid #e9ecef;
	}
	
	.pdf-button {
		padding: 0.25rem 0.75rem;
		background-color: #007bff;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
	}
	
	.pdf-button:disabled {
		background-color: #6c757d;
		cursor: not-allowed;
		opacity: 0.6;
	}
	
	.pdf-container {
		flex: 1;
		overflow: auto;
		padding: 1rem;
		display: flex;
		justify-content: center;
	}
	
	.pdf-canvas {
		border: 1px solid #ddd;
		box-shadow: 0 2px 5px rgba(0,0,0,0.1);
	}
	
	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1rem;
		background-color: #f8f9fa;
		border-bottom: 1px solid #e9ecef;
	}
	
	.panel-header h4 {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
	}
	
	.panel-actions {
		display: flex;
		gap: 0.5rem;
	}
	
	.action-button {
		background: none;
		border: none;
		cursor: pointer;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		transition: background-color 0.2s;
	}
	
	.action-button:hover {
		background-color: #e9ecef;
	}
	
	/* 탭 스타일 개선 */
	.tabs {
		display: flex;
		border-bottom: 1px solid #dee2e6;
		margin-bottom: 1rem;
	}
	
	.tab {
		padding: 0.5rem 1rem;
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		cursor: pointer;
		font-size: 0.9rem;
		color: #666;
		transition: all 0.2s ease-in-out;
		outline: none;
	}
	
	.tab:hover {
		color: #007bff;
		background-color: rgba(0, 123, 255, 0.05);
	}
	
	.html-view-btn {
		margin-left: auto;
		background-color: #f8f9fa;
		border: 1px solid #ddd;
		border-radius: 4px;
		padding: 0.25rem 0.75rem;
	}
	
	.html-view-btn:hover {
		background-color: #e9ecef;
	}
	
	.tab.active {
		color: #007bff;
		border-bottom: 2px solid #007bff;
		font-weight: 600;
	}

	/* 분할 화면 레이아웃 스타일 */
	:global(.splitpanes) {
		height: 600px !important;
		background-color: #fff;
		border: 1px solid #e9ecef;
		border-radius: 4px;
		overflow: hidden;
	}
	
	:global(.splitpanes__pane) {
		background-color: #fff;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}
	
	:global(.splitpanes__splitter) {
		background-color: #e9ecef;
		position: relative;
	}
	
	:global(.splitpanes__splitter:before) {
		content: '';
		position: absolute;
		left: 50%;
		top: 50%;
		transform: translate(-50%, -50%);
		background-color: #adb5bd;
		transition: background-color 0.3s;
	}
	
	:global(.splitpanes__splitter:hover:before) {
		background-color: #007bff;
	}
	
	:global(.splitpanes--vertical > .splitpanes__splitter:before) {
		width: 4px;
		height: 30px;
		border-radius: 2px;
	}
	
	:global(.splitpanes--horizontal > .splitpanes__splitter:before) {
		width: 30px;
		height: 4px;
		border-radius: 2px;
	}
	
	.result-content {
		margin-top: 1rem;
	}

/* 모달 스타일 */
.modal-backdrop {
	position: fixed;
	top: 0;
	left: 0;
	width: 100vw;
	height: 100vh;
	background: rgba(0,0,0,0.5);
	display: flex;
	justify-content: center;
	align-items: center;
	z-index: 2000;
}
.modal-dialog {
	width: 90vw;
	height: 90vh;
	max-width: none !important;
	max-height: none !important;
	outline: none;
	margin: 0;
	padding: 0;
	display: flex;
	align-items: stretch;
	justify-content: stretch;
}

.modal-content {
	width: 100%;
	height: 100%;
	max-width: none;
	max-height: none;
	display: flex;
	flex-direction: column;
}

.modal-split-body {
	width: 100%;
	height: 100%;
	min-width: 0;
	min-height: 0;
	padding: 0;
	display: flex;
	flex-direction: column;
}

:global(.splitpanes) {
	width: 100% !important;
	height: 100% !important;
	min-width: 0;
	min-height: 0;
}
:global(.splitpanes__pane) {
	width: 0;
	min-width: 320px;
	flex: 1 1 0;
	padding: 0;
	margin: 0;
	box-sizing: border-box;
	display: flex;
	flex-direction: column;
}
.modal-content {
	background: #fff;
	border-radius: 0;
	overflow: hidden;
	box-shadow: none;
	display: flex;
	flex-direction: column;
	height: 90vh;
	max-height: none;
}
.modal-header {
	display: flex;
	justify-content: space-between;
	align-items: center;
	padding: 1rem;
	border-bottom: 1px solid #eee;
}
.modal-header h3 {
	margin: 0;
	font-size: 1.15rem;
}
.close-button {
	background: none;
	border: none;
	font-size: 1.6rem;
	cursor: pointer;
	padding: 0 0.5rem;
}
.modal-body {
	padding: 1rem;
	overflow-y: auto;
}

.monaco-editor-container {
	height: 38vh;
	width: 100%;
	border: 1px solid #eee;
	border-radius: 4px;
	background: #f8f9fa;
	margin-bottom: 0.5rem;
	font-size: 1.15em;
}
.modal-split-body {
	height: 100%;
	min-height: 0;
	padding: 0;
	display: flex;
	flex-direction: column;
}
.editor-pane, .preview-pane {
	height: 100%;
	padding: 1.2rem 1.2rem 1.2rem 1.2rem;
	display: flex;
	flex-direction: column;
}
.editor-label {
	font-size: 1.15em;
	margin-bottom: 0.5em;
	font-weight: 600;
	color: #444;
}
.html-preview-iframe {
	width: 100%;
	height: 38vh;
	border: 1px solid #eee;
	background: #fff;
	border-radius: 4px;
}

@media (max-width: 600px) {
	.modal-dialog {
		width: 98vw;
	}
}

.error-message {
  background: #ffe0e0;
  color: #a00;
  padding: 0.7em 1em;
  border-radius: 4px;
  margin-bottom: 1em;
  font-weight: bold;
}


	@media (max-width: 768px) {

		.content-section {
			grid-template-columns: 1fr;
		}
		
		:global(.splitpanes) {
			height: 800px !important;
		}
		
		:global(.splitpanes--vertical) {
			flex-direction: column;
		}
	}
</style>


<h1>Document Parser System</h1>

<div class="container">
	<div class="upload-section">
		<h2>파일 업로드</h2>
		<div class="file-input">
			<input type="file" accept=".pdf,.docx,.xlsx,.txt,.md" on:change={handleFileSelect} />
			<button class="upload-button" on:click={handleUpload} disabled={uploading || !files.length}>
				{uploading ? '업로드 중...' : '업로드'}
			</button>
		</div>
		
		{#if files.length > 0}
			<div class="selected-file">
				<strong>선택된 파일:</strong> {files[0].name} ({formatFileSize(files[0].size)})
			</div>
		{/if}
	</div>

	{#if errorMessage}
  <div class="error-message">{errorMessage}</div>
{/if}
<div class="content-section">
		<div class="job-status-section">
			{#if error}
				<div class="error">
					<h3>오류:</h3>
					<p>{error}</p>
				</div>
			{/if}

			{#if currentJob}
				<div class="current-job">
					<h3>현재 작업:</h3>
					<p><strong>작업 ID:</strong> {currentJob.job_id ?? '-'}</p>
				</div>
			{/if}

			{#if jobStatus}
				<div class="job-status">
					<h3>작업 상태:</h3>
					<div class="status-details">
						<p><strong>상태:</strong> 
							{#if jobStatus.status === 'queued'}
								<span class="status queued">대기 중</span>
							{:else if jobStatus.status === 'processing'}
								<span class="status processing">처리 중</span>
							{:else if jobStatus.status === 'completed'}
								<span class="status completed">완료</span>
							{:else if jobStatus.status === 'failed'}
								<span class="status failed">실패</span>
							{:else}
								<span class="status unknown">{jobStatus.status ?? '알 수 없음'}</span>
							{/if}
						</p>
						
						{#if jobStatus.progress !== undefined}
							<div class="progress-container">
								<div class="progress-bar">
									<div class="progress-fill" style="width: {jobStatus.progress ?? 0}%"></div>
								</div>
								<span class="progress-text">{jobStatus.progress ?? 0}%</span>
							</div>
						{/if}
						
						{#if jobStatus.message}
							<p><strong>메시지:</strong> {jobStatus.message ?? '-'}</p>
						{/if}
						
						{#if (jobStatus.status === 'completed' || jobStatus.status === 'Completed') && !jobResult}
							<button 
								class="upload-button" 
								on:click={() => fetchJobResult(selectedJobId || (jobHistory.length > 0 ? jobHistory[jobHistory.length - 1].job_id : null))}
								style="margin-top: 1rem;"
							>
								결과 보기
							</button>
						{/if}
						
						{#if jobStatus && jobHistory.length > 0}
							<button 
								class="upload-button" 
								on:click={() => {
									const id = selectedJobId || (jobHistory.length > 0 ? jobHistory[jobHistory.length - 1].job_id : null);
									if (id) {
										fetchJobResult(id);
									} else {
										alert('불러올 작업 ID가 없습니다.');
									}
								}}
								style="margin-top: 0.5rem; background-color: #e74c3c;"
							>
								강제 결과 가져오기 (디버깅)
							</button>
						{/if}
					</div>
				</div>
			{/if}

			{#if jobResult}
				<div class="job-result">
					<h3>작업 결과:</h3>
					
					<div class="result-metadata">
						<div class="metadata-item">
							<span class="label">파일명:</span>
							<span class="value">{jobResult.result?.filename ?? '-'}</span>
						</div>
						<div class="metadata-item">
							<span class="label">파일 유형:</span>
							<span class="value">{jobResult.result?.file_type ?? '-'}</span>
						</div>
						<div class="metadata-item">
							<span class="label">파일 크기:</span>
							<span class="value">{jobResult.filesize ? formatFileSize(jobResult.filesize) : '-'}</span>
						</div>
						<div class="metadata-item">
							<span class="label">생성 날짜:</span>
							<span class="value">{jobResult.created_at ? formatDate(jobResult.created_at) : '-'}</span>
						</div>
					</div>
					
					<div class="result-content">
						<Splitpanes horizontal={false}>
							<!-- 원본 문서 패널 -->
							<Pane size={40}>
								<div class="panel-header">
									<h4>원본 문서</h4>
									<div class="panel-actions">
										<button 
											class="action-button" 
											title="원본 파일 다운로드"
											on:click={downloadOriginalFile}
											disabled={!jobResult || !jobResult.original_file}
										>
											<span class="icon">⬇️</span>
										</button>
									</div>
								</div>
								
								{#if jobResult.result?.file_type && jobResult.result.file_type.toLowerCase() === 'pdf'}
									<div class="pdf-viewer">
										<div class="pdf-controls">
											<button 
												class="pdf-button" 
												disabled={pdfPageNum <= 1} 
												on:click={() => {
													if (pdfPageNum > 1) {
														pdfPageNum--;
														renderPdfPage();
													}
												}}
											>
												이전
											</button>
											<span>{pdfPageNum} / {pdfPageCount}</span>
											<button 
												class="pdf-button" 
												disabled={pdfPageNum >= pdfPageCount} 
												on:click={() => {
													if (pdfPageNum < pdfPageCount) {
														pdfPageNum++;
														renderPdfPage();
													}
												}}
											>
												다음
											</button>
										</div>
										<div class="pdf-container">
											<canvas bind:this={pdfCanvas} class="pdf-canvas"></canvas>
										</div>
									</div>
								{:else if jobResult.original_file}
									<div class="original-content">
										<pre>{jobResult.original_file}</pre>
									</div>
								{:else}
									<div class="original-content">
										<p>원본 문서를 표시할 수 없습니다.</p>
									</div>
								{/if}
							</Pane>
							
							<!-- 파싱 결과 패널 -->
							<Pane size={60}>
								<div class="panel-header">
									<h4>파싱 결과</h4>
									<div class="panel-actions">
										<button 
											class="action-button" 
											title="파싱 결과 다운로드"
											on:click={downloadParsedResult}
											disabled={!jobResult || !jobResult.content}
										>
											<span class="icon">⬇️</span>
										</button>
									</div>
								</div>
								
								<!-- 탭 메뉴 -->
								<div class="tabs">
									<button 
										type="button"
										class="tab {activeTab === 'preview' ? 'active' : ''}" 
										on:click={() => changeTab('preview')}
										on:keydown={(e) => e.key === 'Enter' && changeTab('preview')}
										aria-selected={activeTab === 'preview'}
										role="tab"
									>
										미리보기
									</button>
									<button 
										type="button"
										class="tab {activeTab === 'json' ? 'active' : ''}" 
										on:click={() => changeTab('json')}
										on:keydown={(e) => e.key === 'Enter' && changeTab('json')}
										aria-selected={activeTab === 'json'}
										role="tab"
									>
										JSON
									</button>
									<button 
										type="button"
										class="tab {activeTab === 'raw' ? 'active' : ''}" 
										on:click={() => changeTab('raw')}
										on:keydown={(e) => e.key === 'Enter' && changeTab('raw')}
										aria-selected={activeTab === 'raw'}
										role="tab"
									>
										Raw
									</button>
									<button 
										type="button"
										class="tab html-view-btn" 
										on:click={() => showHtmlModal = true}
										on:keydown={(e) => e.key === 'Enter' && (showHtmlModal = true)}
									>
										HTML 보기
									</button>
								</div>
								
								<!-- 탭 내용 -->
								{#if activeTab === 'preview'}
									{#if jobResult?.result?.content?.text}
										<div class="text-content">
											<pre>{jobResult.result.content.text}</pre>
										</div>
									{:else}
										<p>텍스트 추출 결과가 없습니다.</p>
									{/if}

								{:else if activeTab === 'json'}
									<div class="json-content">
										<pre>{JSON.stringify(filterLargeFields(jobResult.result.content), null, 2)}</pre>
									</div>
								{:else if activeTab === 'raw'}
									<div class="raw-content">
										<pre>{JSON.stringify(filterLargeFields(jobResult), null, 2)}</pre>
									</div>
								{/if}
							</Pane>
						</Splitpanes>
					</div>
				</div>
			{/if}
		</div>

		<div class="job-history-section">
			<h3>작업 히스토리</h3>
			{#if jobHistory.length > 0}
				<div class="job-list">
					{#each jobHistory as job}
						<button 
							type="button"
							class="job-item {selectedJobId === job.job_id ? 'selected' : ''}"
							on:click={() => selectJob(job.job_id)}
							aria-pressed={selectedJobId === job.job_id}
						>
							<div class="job-item-header">
								<span class="job-filename">{job.filename ?? '-'}</span>
								<span class="job-size">{formatFileSize(job.filesize ?? 0)}</span>
							</div>
							<div class="job-item-details">
								<span class="job-id">ID: {job.job_id ? job.job_id.substring(0, 8) + '...' : '-'}</span>
								<span class="job-date">{job.created_at ? formatDate(job.created_at) : '-'}</span>
							</div>
						</button>
					{/each}
				</div>
			{:else}
				<p class="no-history">작업 히스토리가 없습니다.</p>
			{/if}
		</div>
	</div>
</div>

<!-- HTML 모달 팝업 -->
{#if showHtmlModal}
	<div 
		class="modal-backdrop" 
		on:click|self={() => showHtmlModal = false}
		on:keydown={(e) => e.key === 'Escape' && (showHtmlModal = false)}
		role="presentation"
	>
		<div 
			class="modal-dialog"
			role="dialog"
			aria-modal="true"
			aria-labelledby="html-modal-title"
			tabindex="-1"
			bind:this={htmlModalElement}
			use:modalFocus
		>
			<div class="modal-content">
				<div class="modal-header">
					<h3 id="html-modal-title">HTML 미리보기</h3>
					<button 
						class="close-button" 
						on:click={() => showHtmlModal = false}
						on:keydown={(e) => e.key === 'Enter' && (showHtmlModal = false)}
						aria-label="모달 닫기"
					>×</button>
				</div>
				{#if jobResult?.result?.content}
					<!-- 툴바/탭 컨트롤 -->
					<div class="html-modal-toolbar">
						<button class="toolbar-tab {htmlModalMode === 'original' ? 'active' : ''}" on:click={() => htmlModalMode = 'original'}>원본</button>
						<button class="toolbar-tab {htmlModalMode === 'edited' ? 'active' : ''}" on:click={() => htmlModalMode = 'edited'}>수정본</button>
						<button class="toolbar-tab {htmlModalMode === 'diff' ? 'active' : ''}" on:click={() => htmlModalMode = 'diff'}>비교(Diff)</button>
						<div class="toolbar-actions">
							<button class="toolbar-btn" title="HTML 및 이미지 다운로드" on:click={downloadHtmlWithImages}>💾 다운로드</button>
							<button class="toolbar-btn" title="초기화" disabled>⟳</button>
							<button class="toolbar-btn" title="복사" disabled>📋</button>
						</div>
					</div>
					<div class="modal-body modal-split-body">
						{#if htmlModalMode === 'original'}
							<Splitpanes>
								<Pane size={55}>
									<div class="editor-pane">
										<div class="editor-label">원본 HTML (읽기 전용)</div>
										<div bind:this={monacoHtmlOriginalContainer} class="monaco-editor-container"></div>
									</div>
								</Pane>
								<Pane size={45}>
									<div class="preview-pane">
										<div class="editor-label">미리보기</div>
										<iframe class="html-preview-iframe" srcdoc={originalHtmlValue} sandbox="allow-same-origin allow-scripts" title="HTML 미리보기"></iframe>
									</div>
								</Pane>
							</Splitpanes>
						{:else if htmlModalMode === 'edited'}
							<Splitpanes>
								<Pane size={55}>
									<div class="editor-pane">
										<div class="editor-label">수정본 HTML (수정 가능)</div>
										<div bind:this={monacoHtmlEditedContainer} class="monaco-editor-container"></div>
									</div>
								</Pane>
								<Pane size={45}>
									<div class="preview-pane">
										<div class="editor-label">미리보기</div>
										<iframe class="html-preview-iframe" srcdoc={editedHtmlValue} sandbox="allow-same-origin allow-scripts" title="HTML 미리보기"></iframe>
									</div>
								</Pane>
							</Splitpanes>
						{:else if htmlModalMode === 'diff'}
							<Splitpanes>
								<Pane size={60}>
									<div class="editor-pane">
										<div class="editor-label">원본 vs. 수정본 비교(Diff)</div>
										<div bind:this={monacoHtmlDiffContainer} class="monaco-editor-container"></div>
									</div>
								</Pane>
								<Pane size={40}>
									<div class="preview-pane">
										<div class="editor-label">수정본 미리보기</div>
										<iframe class="html-preview-iframe" srcdoc={editedHtmlValue} sandbox="allow-same-origin allow-scripts" title="HTML 미리보기"></iframe>
									</div>
								</Pane>
							</Splitpanes>
						{/if}
					</div>
				{:else}
					<div style="padding:2em; text-align:center; color:#888;">
						HTML 데이터를 불러올 수 없습니다.
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}