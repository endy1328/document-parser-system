<svelte:head>
	<title>Document Parser System</title>
	<meta name="description" content="비동기 문서 파싱 시스템" />
	<link rel="preconnect" href="https://fonts.googleapis.com">
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
	<link href="https://fonts.googleapis.com/css2?family=Noto+Sans+KR:wght@400;500;700&display=swap" rel="stylesheet">
</svelte:head>

<header>
	<div class="header-content">
		<div class="logo">Document Parser System</div>
		<nav>
			<ul>
				<li><a href="/" class="active">홈</a></li>
				<li><a href="/about">소개</a></li>
			</ul>
		</nav>
	</div>
</header>

<main>
	<slot />
</main>

<footer>
	<div class="footer-content">
		<p>© 2025 Document Parser System - Phase 3</p>
	</div>
</footer>

<style>
	:global(body) {
		font-family: 'Noto Sans KR', sans-serif;
		background-color: #f5f5f5;
		color: #333;
		margin: 0;
		padding: 0;
	}

	:global(*) {
		box-sizing: border-box;
	}

	header {
		background-color: #2c3e50;
		color: white;
		padding: 1rem 0;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}

	.header-content {
		max-width: 1200px;
		margin: 0 auto;
		padding: 0 1rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.logo {
		font-size: 1.5rem;
		font-weight: bold;
	}

	nav ul {
		display: flex;
		list-style: none;
		margin: 0;
		padding: 0;
		gap: 1.5rem;
	}

	nav a {
		color: white;
		text-decoration: none;
		padding: 0.5rem 0;
		transition: color 0.2s;
	}

	nav a:hover {
		color: #3498db;
	}

	nav a.active {
		border-bottom: 2px solid #3498db;
	}

	main {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem 1rem;
		min-height: calc(100vh - 140px);
	}

	footer {
		background-color: #2c3e50;
		color: #ecf0f1;
		padding: 1rem 0;
		margin-top: 2rem;
	}

	.footer-content {
		max-width: 1200px;
		margin: 0 auto;
		padding: 0 1rem;
		text-align: center;
		font-size: 0.9rem;
	}
</style>
