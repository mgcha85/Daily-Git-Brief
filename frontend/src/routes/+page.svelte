<script lang="ts">
    import { onMount, onDestroy } from "svelte";

    interface LanguageInfo {
        language: string;
        percentage: number;
    }

    interface TrendingRepo {
        rank: number;
        repo_id: number;
        repo_name: string;
        github_url: string;
        primary_language: string | null;
        languages: LanguageInfo[];
        description: string | null;
        korean_summary: string | null;
        stars: number | null;
        forks: number | null;
        total_score: number | null;
    }

    interface ApiResponse {
        success: boolean;
        data: TrendingRepo[] | null;
        error: string | null;
    }

    let repos: TrendingRepo[] = [];
    let loading = true;
    let error: string | null = null;
    let selectedDate = new Date().toISOString().split("T")[0];
    let collecting = false;

    // SSE Progress State
    let progress = { message: "", current: 0, total: 0 };
    let eventSource: EventSource | null = null;

    const languageColors: Record<string, string> = {
        TypeScript: "#3178c6",
        JavaScript: "#f1e05a",
        Python: "#3572A5",
        Rust: "#dea584",
        Go: "#00ADD8",
        Java: "#b07219",
        "C++": "#f34b7d",
        C: "#555555",
        Shell: "#89e051",
        Ruby: "#701516",
        Swift: "#F05138",
        Kotlin: "#A97BFF",
        PHP: "#4F5D95",
        HTML: "#e34c26",
        CSS: "#563d7c",
        Vue: "#41b883",
        Svelte: "#ff3e00",
    };

    async function fetchTrends() {
        loading = true;
        error = null;

        try {
            const response = await fetch(`/api/trends?date=${selectedDate}`);
            const data: ApiResponse = await response.json();

            if (data.success && data.data) {
                repos = data.data;
            } else {
                error = data.error || "Failed to fetch trends";
            }
        } catch (e) {
            error = "Network error. Is the backend running?";
        } finally {
            loading = false;
        }
    }

    async function triggerCollection() {
        if (collecting && eventSource) return;

        collecting = true;
        progress = { message: "Initializing...", current: 0, total: 0 };

        try {
            const response = await fetch("/api/collect", { method: "POST" });

            if (response.status === 409) {
                startListening(); // Already running
                return;
            }

            const data = await response.json();
            if (data.success) {
                startListening();
            } else {
                alert("Collection failed: " + data.error);
                collecting = false;
            }
        } catch (e) {
            alert("Failed to trigger collection");
            collecting = false;
        }
    }

    function startListening() {
        if (eventSource) eventSource.close();

        eventSource = new EventSource("/api/collect/progress");

        eventSource.onmessage = (event) => {
            try {
                const status = JSON.parse(event.data);
                progress = {
                    message: status.message,
                    current: status.current_count,
                    total: status.total_count,
                };

                if (!status.is_running) {
                    collecting = false;
                    eventSource?.close();
                    eventSource = null;
                    fetchTrends();
                    alert("Collection Completed!");
                }
            } catch (e) {
                console.error("Failed to parse SSE message", e);
            }
        };

        eventSource.onerror = (err) => {
            console.error("SSE Error", err);
        };
    }

    function formatNumber(num: number | null): string {
        if (num === null) return "-";
        if (num >= 1000) return (num / 1000).toFixed(1) + "k";
        return num.toString();
    }

    function getLanguageColor(lang: string): string {
        return languageColors[lang] || "#8b949e";
    }

    onMount(fetchTrends);

    onDestroy(() => {
        if (eventSource) eventSource.close();
    });
</script>

<svelte:head>
    <title>Daily Git Brief - GitHub 트렌드 대시보드</title>
</svelte:head>

<div class="container">
    <section class="hero fade-in">
        <h1>🔥 오늘의 GitHub 트렌드</h1>
        <p>
            전 세계에서 가장 주목받는 오픈소스 프로젝트를 한국어 요약과 함께
            살펴보세요
        </p>
    </section>

    <section class="controls fade-in">
        <div class="controls-top">
            <div class="date-picker">
                <label for="date">날짜 선택:</label>
                <input
                    type="date"
                    id="date"
                    bind:value={selectedDate}
                    on:change={fetchTrends}
                    max={new Date().toISOString().split("T")[0]}
                />
            </div>
            <button
                class="btn btn-primary"
                on:click={triggerCollection}
                disabled={collecting}
            >
                {collecting ? "수집 중..." : "📥 데이터 수집"}
            </button>
        </div>

        {#if collecting}
            <div class="progress-container fade-in">
                <div class="progress-bar">
                    <div
                        class="fill"
                        style="width: {progress.total > 0
                            ? (progress.current / progress.total) * 100
                            : 0}%"
                    ></div>
                </div>
                <p class="progress-text">
                    {progress.message} ({progress.current}/{progress.total})
                </p>
            </div>
        {/if}
    </section>

    {#if loading}
        <div class="loading">
            <div class="spinner"></div>
        </div>
    {:else if error}
        <div class="error-card card fade-in">
            <h3>⚠️ 오류 발생</h3>
            <p>{error}</p>
            <button class="btn btn-secondary" on:click={fetchTrends}
                >다시 시도</button
            >
        </div>
    {:else if repos.length === 0}
        <div class="empty-card card fade-in">
            <h3>📭 데이터 없음</h3>
            <p>{selectedDate}의 트렌딩 데이터가 없습니다.</p>
            <p>
                위의 "데이터 수집" 버튼을 클릭하여 오늘의 데이터를 가져오세요.
            </p>
        </div>
    {:else}
        <div class="table-container fade-in">
            <table>
                <thead>
                    <tr>
                        <th style="width: 60px">#</th>
                        <th>레포지토리</th>
                        <th>언어</th>
                        <th style="width: 120px">⭐ Stars</th>
                        <th style="width: 100px">🍴 Forks</th>
                        <th style="min-width: 250px">한국어 요약</th>
                    </tr>
                </thead>
                <tbody>
                    {#each repos as repo (repo.repo_id)}
                        <tr>
                            <td class="rank">
                                <span
                                    class="rank-badge"
                                    class:top3={repo.rank <= 3}
                                >
                                    {repo.rank}
                                </span>
                            </td>
                            <td class="repo-info">
                                <a
                                    href={repo.github_url}
                                    target="_blank"
                                    class="repo-name"
                                >
                                    {repo.repo_name}
                                </a>
                                {#if repo.description}
                                    <p class="repo-desc">{repo.description}</p>
                                {/if}
                            </td>
                            <td class="languages">
                                {#if repo.primary_language}
                                    <span
                                        class="lang-badge primary"
                                        style="--lang-color: {getLanguageColor(
                                            repo.primary_language,
                                        )}"
                                    >
                                        {repo.primary_language}
                                    </span>
                                {/if}
                                {#each repo.languages.filter((l) => l.language !== repo.primary_language) as lang}
                                    <span
                                        class="lang-badge"
                                        style="--lang-color: {getLanguageColor(
                                            lang.language,
                                        )}"
                                        title="{lang.percentage.toFixed(1)}%"
                                    >
                                        {lang.language}
                                        <small
                                            >{lang.percentage.toFixed(
                                                0,
                                            )}%</small
                                        >
                                    </span>
                                {/each}
                            </td>
                            <td class="stat">{formatNumber(repo.stars)}</td>
                            <td class="stat">{formatNumber(repo.forks)}</td>
                            <td class="summary">
                                {repo.korean_summary || "-"}
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

<style>
    .hero {
        text-align: center;
        padding: var(--space-8) 0 var(--space-6);
    }

    .hero h1 {
        font-size: var(--font-size-3xl);
        margin-bottom: var(--space-3);
        background: var(--gradient-primary);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }

    .hero p {
        color: var(--color-text-secondary);
        font-size: var(--font-size-lg);
    }

    .controls {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
        margin-bottom: var(--space-6);
        padding: var(--space-4);
        background: var(--color-bg-secondary);
        border-radius: var(--radius-lg);
        border: 1px solid var(--color-border);
    }

    .controls-top {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;
    }

    .date-picker {
        display: flex;
        align-items: center;
        gap: var(--space-3);
    }

    .date-picker label {
        color: var(--color-text-secondary);
        font-size: var(--font-size-sm);
    }

    .date-picker input {
        background: var(--color-bg-tertiary);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        padding: var(--space-2) var(--space-3);
        color: var(--color-text-primary);
        font-family: inherit;
    }

    .date-picker input:focus {
        outline: none;
        border-color: var(--color-accent-blue);
    }

    .error-card,
    .empty-card {
        text-align: center;
        padding: var(--space-12);
    }

    .error-card h3,
    .empty-card h3 {
        margin-bottom: var(--space-4);
        font-size: var(--font-size-xl);
    }

    .error-card p,
    .empty-card p {
        color: var(--color-text-secondary);
        margin-bottom: var(--space-4);
    }

    .rank {
        text-align: center;
    }

    .rank-badge {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border-radius: var(--radius-full);
        background: var(--color-bg-tertiary);
        font-weight: 600;
        font-size: var(--font-size-sm);
    }

    .rank-badge.top3 {
        background: var(--gradient-primary);
        color: white;
    }

    .repo-info {
        max-width: 300px;
    }

    .repo-name {
        font-weight: 600;
        display: block;
        margin-bottom: var(--space-1);
    }

    .repo-desc {
        font-size: var(--font-size-xs);
        color: var(--color-text-muted);
        line-height: 1.4;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .languages {
        display: flex;
        flex-wrap: wrap;
        gap: var(--space-1);
    }

    .lang-badge {
        display: inline-flex;
        align-items: center;
        gap: var(--space-1);
        padding: var(--space-1) var(--space-2);
        font-size: var(--font-size-xs);
        border-radius: var(--radius-full);
        background: var(--color-bg-tertiary);
        color: var(--color-text-secondary);
        border-left: 3px solid var(--lang-color, var(--color-text-muted));
    }

    .lang-badge.primary {
        background: rgba(88, 166, 255, 0.1);
        color: var(--color-text-primary);
        font-weight: 500;
    }

    .lang-badge small {
        opacity: 0.7;
    }

    .stat {
        font-weight: 500;
        color: var(--color-text-secondary);
    }

    .summary {
        color: var(--color-text-secondary);
        font-size: var(--font-size-sm);
        max-width: 350px;
    }

    .progress-container {
        width: 100%;
    }
    .progress-bar {
        height: 8px;
        background: var(--color-bg-tertiary);
        border-radius: var(--radius-full);
        overflow: hidden;
        margin-bottom: var(--space-2);
    }
    .fill {
        height: 100%;
        background: var(--color-accent-blue);
        transition: width 0.3s ease;
    }
    .progress-text {
        font-size: var(--font-size-sm);
        color: var(--color-text-secondary);
        text-align: right;
    }
</style>
