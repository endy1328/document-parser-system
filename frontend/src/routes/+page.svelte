<script lang="ts">
  import { onMount } from 'svelte';
  let file: File | null = null;
  let uploading = false;
  let result: any = null;
  let error: string | null = null;

  async function handleFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      file = input.files[0];
      result = null;
      error = null;
    }
  }

  async function uploadFile() {
    if (!file) return;
    uploading = true;
    result = null;
    error = null;
    const formData = new FormData();
    formData.append('file', file);
    try {
      const res = await fetch('http://localhost:8080/upload', {
        method: 'POST',
        body: formData
      });
      const data = await res.json();
      result = data;
      if (!res.ok || data.status !== 'success') {
        error = data.error || '업로드 실패';
      }
    } catch (e) {
      error = '네트워크 오류 또는 서버 응답 없음';
    } finally {
      uploading = false;
    }
  }
</script>

<style>
  .dropzone {
    border: 2px dashed #aaa;
    padding: 2rem;
    text-align: center;
    border-radius: 8px;
    margin-bottom: 1rem;
    background: #fafbfc;
    cursor: pointer;
    transition: border-color 0.2s;
  }
  .dropzone.dragover {
    border-color: #0070f3;
    background: #e3f0ff;
  }
  .result {
    margin-top: 1.5rem;
    white-space: pre-wrap;
    background: #f3f3f3;
    padding: 1rem;
    border-radius: 6px;
  }
  .error {
    color: #d00;
    margin-top: 1rem;
  }
</style>

<div class="dropzone"
     on:dragover|preventDefault={() => $$.ctx[0] = true}
     on:dragleave|preventDefault={() => $$.ctx[0] = false}
     on:drop|preventDefault={async (e) => {
       $$.ctx[0] = false;
       if (e.dataTransfer && e.dataTransfer.files.length > 0) {
         file = e.dataTransfer.files[0];
         result = null;
         error = null;
       }
     }}>
  <input type="file" accept="application/pdf" on:change={handleFileChange} style="display:none" bind:this={inputEl} />
  <p>{file ? `선택된 파일: ${file.name}` : 'PDF 파일을 드래그하거나 클릭하여 선택하세요.'}</p>
  <button type="button" on:click={() => inputEl.click()} disabled={uploading}>파일 선택</button>
</div>

<button type="button" on:click={uploadFile} disabled={!file || uploading}>
  {uploading ? '업로드 중...' : '업로드'}
</button>

{#if error}
  <div class="error">{error}</div>
{/if}

{#if result}
  <div class="result">
    <strong>결과(JSON):</strong>
    <pre>{JSON.stringify(result, null, 2)}</pre>
    {#if result.text}
      <hr />
      <strong>추출된 텍스트:</strong>
      <div>{result.text}</div>
    {/if}
  </div>
{/if}
