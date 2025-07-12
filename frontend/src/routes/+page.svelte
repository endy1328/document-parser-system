<script>
	import { onMount, onDestroy } from 'svelte';
	import { Splitpanes, Pane } from 'svelte-splitpanes';
	import * as monaco from 'monaco-editor';
	import * as pdfjsLib from 'pdfjs-dist';

	let files = [];
	let uploading = false;
	let error = null;

	// 작업 관련 상태
	let currentJob = null;
	let jobHistory = [];
	let pollingInterval = null;
	let selectedJobId = null;

	// 작업 상태 및 결과
	let jobStatus = null;
	let jobResult = null;
	let parsedContent = null; // 파싱된 내용을 저장할 변수

	// UI 관련 상태
	let activeTab = 'preview'; // preview, json, raw
	let pdfDocument = null;
	let pdfPageNum = 1;
	let pdfPageCount = 0;
	let pdfCanvas;
	let jsonEditor;
	let jsonEditorContainer;
	let originalDocumentElement;
	let showOriginalDocument = true;

	// PDF 뷰어 초기화
	function initPdfViewer() {
		// PDF.js 워커 경로 설정
		pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdn.jsdelivr.net/npm/pdfjs-dist@3.4.120/build/pdf.worker.min.js';
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
	async function loadPdf(pdfData) {
		if (!pdfData) return;
		
		try {
			// PDF 로드
			const loadingTask = pdfjsLib.getDocument({ data: pdfData });
			pdfDocument = await loadingTask.promise;
			pdfPageCount = pdfDocument.numPages;
			pdfPageNum = 1;
			
			// 첫 페이지 렌더링
			renderPdfPage();
		} catch (e) {
			console.error('PDF 로드 오류:', e);
		}
	}

	// PDF 페이지 렌더링
	async function renderPdfPage() {
		if (!pdfDocument || !pdfCanvas) return;
		
		try {
			const page = await pdfDocument.getPage(pdfPageNum);
			const viewport = page.getViewport({ scale: 1.5 });
			
			const context = pdfCanvas.getContext('2d');
			pdfCanvas.height = viewport.height;
			pdfCanvas.width = viewport.width;
			
			const renderContext = {
				canvasContext: context,
				viewport: viewport
			};
			
			await page.render(renderContext).promise;
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
						content = JSON.stringify(jobResult.content, null, 2);
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
						content = parsedContent ? parsedContent.data : JSON.stringify(jobResult.content, null, 2);
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
	});

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
			
			if (response.ok) {
				jobStatus = await response.json();
				
				// 작업이 완료되면 결과 가져오기
				if (jobStatus.status === 'completed') {
					fetchJobResult(jobId);
					stopPolling(); // 완료되면 폴링 중지
				} else if (jobStatus.status === 'failed') {
					error = `작업 실패: ${jobStatus.message || '알 수 없는 오류'}`;
					stopPolling(); // 실패해도 폴링 중지
				}
			} else {
				console.error('작업 상태 가져오기 실패:', response.status);
			}
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
				if (jobResult && jobResult.content) {
					console.log('파일 유형:', jobResult.file_type);
					parsedContent = processContent(jobResult.content, jobResult.file_type);
					console.log('처리된 내용:', parsedContent);
					
					// PDF 파일인 경우 원본 문서 로드
					if (jobResult.file_type.toLowerCase() === 'pdf' && jobResult.original_file) {
						try {
							// base64 데이터를 바이너리로 변환
							const binaryData = atob(jobResult.original_file);
							const len = binaryData.length;
							const bytes = new Uint8Array(len);
							
							for (let i = 0; i < len; i++) {
								bytes[i] = binaryData.charCodeAt(i);
							}
							
							// PDF 로드
							setTimeout(() => {
								loadPdf(bytes.buffer);
							}, 100);
						} catch (e) {
							console.error('PDF 로드 오류:', e);
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
					</div>
				</div>
			{/if}

			{#if jobResult}
				<div class="job-result">
					<h3>작업 결과:</h3>
					
					<div class="result-metadata">
						<div class="metadata-item">
							<span class="label">파일명:</span>
							<span class="value">{jobResult.filename ?? '-'}</span>
						</div>
						<div class="metadata-item">
							<span class="label">파일 유형:</span>
							<span class="value">{jobResult.file_type ?? '-'}</span>
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
								
								{#if jobResult.file_type && jobResult.file_type.toLowerCase() === 'pdf'}
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
								</div>
								
								<!-- 탭 내용 -->
								{#if activeTab === 'preview'}
									{#if parsedContent}
										{#if parsedContent.type === 'text'}
											<div class="text-content">
												<pre>{parsedContent.data ?? ''}</pre>
											</div>
										{:else if parsedContent.type === 'html'}
											<div class="html-content">
												{@html parsedContent.data ?? ''}
											</div>
										{:else if parsedContent.type === 'markdown'}
											<div class="markdown-content">
												<pre>{parsedContent.data ?? ''}</pre>
											</div>
										{:else}
											<div class="json-content">
												<pre>{parsedContent.data ?? ''}</pre>
											</div>
										{/if}
									{:else}
										<p>내용을 표시할 수 없습니다.</p>
									{/if}
								{:else if activeTab === 'json'}
									<div bind:this={jsonEditorContainer} class="json-editor"></div>
								{:else if activeTab === 'raw'}
									<div class="raw-content">
										<pre>{JSON.stringify(jobResult, null, 2)}</pre>
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

<style>
	@import 'svelte-splitpanes/splitpanes.css';
	
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
		height: 100%;
		overflow: hidden;
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
	
	/* Monaco 에디터 스타일 */
	.json-editor {
		height: 500px;
		border: 1px solid #ddd;
		border-radius: 4px;
	}
	
	.text-content, .html-content, .markdown-content, .json-content {
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
		padding: 0.75rem 1.25rem;
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		cursor: pointer;
		font-weight: 500;
		color: #495057;
		transition: all 0.2s ease-in-out;
		outline: none;
	}
	
	.tab:hover {
		color: #007bff;
		background-color: rgba(0, 123, 255, 0.05);
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
