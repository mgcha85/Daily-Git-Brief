<script lang="ts">
    import { onMount } from "svelte";
    import Chart from "chart.js/auto";

    interface LanguageTrend {
        date: string;
        language: string;
        normalized_percentage: number;
        repo_count: number;
    }

    interface ApiResponse {
        success: boolean;
        data: LanguageTrend[] | null;
        error: string | null;
    }

    let dailyTrends: LanguageTrend[] = [];
    let weeklyTrends: LanguageTrend[] = [];
    let loading = true;
    let error: string | null = null;
    let selectedDate = new Date().toISOString().split("T")[0];
    let activeTab: "daily" | "weekly" = "daily";
    let chartCanvas: HTMLCanvasElement;
    let chart: Chart | null = null;

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

    function getColor(lang: string, alpha = 1): string {
        const color = languageColors[lang] || "#8b949e";
        if (alpha === 1) return color;

        const r = parseInt(color.slice(1, 3), 16);
        const g = parseInt(color.slice(3, 5), 16);
        const b = parseInt(color.slice(5, 7), 16);
        return `rgba(${r}, ${g}, ${b}, ${alpha})`;
    }

    async function fetchTrends() {
        loading = true;
        error = null;

        try {
            const endpoint =
                activeTab === "daily"
                    ? `/api/languages/daily?date=${selectedDate}`
                    : `/api/languages/weekly?date=${selectedDate}`;

            const response = await fetch(endpoint);
            const data: ApiResponse = await response.json();

            if (data.success && data.data) {
                if (activeTab === "daily") {
                    dailyTrends = data.data;
                } else {
                    weeklyTrends = data.data;
                }
                updateChart();
            } else {
                error = data.error || "Failed to fetch language trends";
            }
        } catch (e) {
            error = "Network error. Is the backend running?";
        } finally {
            loading = false;
        }
    }

    function updateChart() {
        const trends = activeTab === "daily" ? dailyTrends : weeklyTrends;
        const topTrends = trends.slice(0, 10);

        if (chart) {
            chart.destroy();
        }

        if (!chartCanvas || topTrends.length === 0) return;

        const ctx = chartCanvas.getContext("2d");
        if (!ctx) return;

        chart = new Chart(ctx, {
            type: "doughnut",
            data: {
                labels: topTrends.map((t) => t.language),
                datasets: [
                    {
                        data: topTrends.map((t) => t.normalized_percentage),
                        backgroundColor: topTrends.map((t) =>
                            getColor(t.language, 0.8),
                        ),
                        borderColor: topTrends.map((t) => getColor(t.language)),
                        borderWidth: 2,
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: true,
                plugins: {
                    legend: {
                        position: "right",
                        labels: {
                            color: "#f0f6fc",
                            font: { family: "Inter", size: 12 },
                            padding: 15,
                        },
                    },
                    tooltip: {
                        callbacks: {
                            label: (context) => {
                                const trend = topTrends[context.dataIndex];
                                return `${trend.language}: ${trend.normalized_percentage.toFixed(1)}% (${trend.repo_count} repos)`;
                            },
                        },
                    },
                },
            },
        });
    }

    function setTab(tab: "daily" | "weekly") {
        activeTab = tab;
        fetchTrends();
    }

    onMount(fetchTrends);
</script>

<svelte:head>
    <title>언어 트렌드 - Daily Git Brief</title>
</svelte:head>

<div class="container">
    <section class="hero fade-in">
        <h1>📊 언어 트렌드 분석</h1>
        <p>트렌딩 레포지토리에서 사용된 프로그래밍 언어 통계</p>
    </section>

    <section class="controls fade-in">
        <div class="tabs">
            <button
                class="tab"
                class:active={activeTab === "daily"}
                on:click={() => setTab("daily")}
            >
                일별
            </button>
            <button
                class="tab"
                class:active={activeTab === "weekly"}
                on:click={() => setTab("weekly")}
            >
                주간
            </button>
        </div>
        <div class="date-picker">
            <label for="date">날짜:</label>
            <input
                type="date"
                id="date"
                bind:value={selectedDate}
                on:change={fetchTrends}
                max={new Date().toISOString().split("T")[0]}
            />
        </div>
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
    {:else}
        <div class="content fade-in">
            <div class="chart-section card">
                <h2>상위 10개 언어</h2>
                <div class="chart-wrapper">
                    <canvas bind:this={chartCanvas}></canvas>
                </div>
            </div>

            <div class="table-section">
                <div class="table-container">
                    <table>
                        <thead>
                            <tr>
                                <th style="width: 60px">#</th>
                                <th>언어</th>
                                <th>점유율</th>
                                <th>레포 수</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each activeTab === "daily" ? dailyTrends : weeklyTrends as trend, i}
                                <tr>
                                    <td class="rank">
                                        <span
                                            class="rank-badge"
                                            class:top3={i < 3}>{i + 1}</span
                                        >
                                    </td>
                                    <td>
                                        <span
                                            class="lang-indicator"
                                            style="--lang-color: {getColor(
                                                trend.language,
                                            )}"
                                        >
                                            {trend.language}
                                        </span>
                                    </td>
                                    <td>
                                        <div class="progress-bar">
                                            <div
                                                class="progress-fill"
                                                style="width: {trend.normalized_percentage}%; background: {getColor(
                                                    trend.language,
                                                )}"
                                            ></div>
                                            <span class="progress-label"
                                                >{trend.normalized_percentage.toFixed(
                                                    1,
                                                )}%</span
                                            >
                                        </div>
                                    </td>
                                    <td class="repo-count"
                                        >{trend.repo_count}</td
                                    >
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            </div>
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
        background: linear-gradient(135deg, #3fb950, #58a6ff);
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
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--space-6);
        padding: var(--space-4);
        background: var(--color-bg-secondary);
        border-radius: var(--radius-lg);
        border: 1px solid var(--color-border);
    }

    .tabs {
        display: flex;
        gap: var(--space-2);
    }

    .tab {
        padding: var(--space-2) var(--space-4);
        border: 1px solid var(--color-border);
        background: var(--color-bg-tertiary);
        color: var(--color-text-secondary);
        border-radius: var(--radius-md);
        cursor: pointer;
        transition: all var(--transition-fast);
    }

    .tab:hover {
        background: var(--color-bg-hover);
        color: var(--color-text-primary);
    }

    .tab.active {
        background: var(--color-accent-blue);
        border-color: var(--color-accent-blue);
        color: white;
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

    .content {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--space-6);
    }

    @media (max-width: 900px) {
        .content {
            grid-template-columns: 1fr;
        }
    }

    .chart-section {
        display: flex;
        flex-direction: column;
    }

    .chart-section h2 {
        font-size: var(--font-size-lg);
        margin-bottom: var(--space-4);
    }

    .chart-wrapper {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 300px;
    }

    .error-card {
        text-align: center;
        padding: var(--space-12);
    }

    .rank-badge {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        border-radius: var(--radius-full);
        background: var(--color-bg-tertiary);
        font-weight: 600;
        font-size: var(--font-size-xs);
    }

    .rank-badge.top3 {
        background: var(--gradient-primary);
        color: white;
    }

    .lang-indicator {
        display: inline-flex;
        align-items: center;
        gap: var(--space-2);
        font-weight: 500;
    }

    .lang-indicator::before {
        content: "";
        width: 12px;
        height: 12px;
        border-radius: var(--radius-full);
        background: var(--lang-color);
    }

    .progress-bar {
        position: relative;
        height: 24px;
        background: var(--color-bg-tertiary);
        border-radius: var(--radius-full);
        overflow: hidden;
        min-width: 150px;
    }

    .progress-fill {
        height: 100%;
        border-radius: var(--radius-full);
        transition: width var(--transition-normal);
    }

    .progress-label {
        position: absolute;
        right: var(--space-3);
        top: 50%;
        transform: translateY(-50%);
        font-size: var(--font-size-xs);
        font-weight: 600;
    }

    .repo-count {
        font-weight: 500;
        color: var(--color-text-secondary);
    }
</style>
