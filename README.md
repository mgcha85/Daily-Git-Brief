# Daily-Git-Brief

GitHub 트렌딩 레포지토리를 수집하고, README를 LLM으로 한국어 요약하여 대시보드로 제공하는 서비스입니다.

## Features

- 📊 **Daily GitHub Trends**: OSS Insight API를 통한 일별 트렌딩 레포지토리 수집
- 🇰🇷 **Korean Summaries**: DeepSeek LLM을 활용한 README 한국어 요약
- 📈 **Language Analytics**: 레포지토리별 언어 통계 및 일별 언어 트렌드 분석
- 💾 **DuckDB Storage**: 고성능 분석 쿼리를 위한 DuckDB 데이터베이스

## Tech Stack

- **Backend**: Rust + Axum
- **Frontend**: Svelte + Vite
- **Database**: DuckDB
- **Deployment**: Podman Compose
- **CI/CD**: GitHub Container Registry (GHCR)

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 20+
- Podman & Podman Compose

### Setup

```bash
# Clone the repository
git clone https://github.com/mgcha85/Daily-Git-Brief.git
cd Daily-Git-Brief

# Copy environment file
cp .env.example .env
# Edit .env with your API keys

# Run with Podman
podman-compose up --build -d
```

### Development

```bash
# Backend
cd backend
cargo run

# Frontend (in another terminal)
cd frontend
npm install
npm run dev
```

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `OSS_INSIGHT_BASE_URL` | OSS Insight API URL | `https://api.ossinsight.io` |
| `GITHUB_TOKEN` | GitHub API token (optional) | - |
| `DEEPSEEK_BASE_URL` | DeepSeek API URL | `https://api.deepseek.com` |
| `DEEPSEEK_API_KEY` | DeepSeek API key | **Required** |
| `LANGUAGE_THRESHOLD` | Minimum language % to track | `0.2` |
| `DATABASE_PATH` | DuckDB file path | `./data/daily_git_brief.duckdb` |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/trends` | Today's trending repos with Korean summaries |
| GET | `/api/trends?date=YYYY-MM-DD` | Trends for specific date |
| GET | `/api/languages/daily` | Daily language trends |
| GET | `/api/languages/weekly` | Weekly aggregated language trends |
| POST | `/api/collect` | Trigger manual data collection |

## License

MIT
