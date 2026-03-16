# gh-star-trend

分析 GitHub repo 的星星成長趨勢，計算每週斜率（簡單平均 + 線性回歸）。

## 安裝

```bash
cargo build --release
cp target/release/gh-star-trend /usr/local/bin/
```

或從 [Releases](../../releases) 下載預編譯的 binary。

## 使用

```bash
./gh-star-trend owner/repo [--token TOKEN] [--weeks N]
```

### 參數

| 參數 | 說明 |
|------|------|
| `owner/repo` | GitHub repo（必填） |
| `--token TOKEN` | GitHub Personal Access Token |
| `--weeks N` | 只分析最近 N 週 |

### GitHub Token

|  | 無 Token | 有 Token |
|--|----------|----------|
| 每小時 API 請求數 | 60 | 5,000 |
| 每次可抓星星數 | ~6,000 | ~500,000 |
| 適合 | 小型 repo (<6k 星) | 任何大小的 repo |

#### 取得 Token

1. 到 [github.com/settings/tokens](https://github.com/settings/tokens)
2. 點 **Generate new token (classic)**
3. 不需要勾選任何 scope（只讀 public repo 不需要權限）
4. 點 **Generate token**，複製產生的 `ghp_xxxx...`

#### 設定 Token

```bash
# 方式一：環境變數（推薦，寫進 .zshrc / .bashrc）
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx

# 方式二：參數
./gh-star-trend owner/repo --token ghp_xxxxxxxxxxxx
```

## 快取機制

資料會快取在 `~/.gh-star-trend/` 目錄下，每個 repo 一個 JSON 檔。

- 第一次執行會從頭抓，之後只抓新增的星星
- 記錄最後抓到的頁數，下次從該頁繼續
- 被 rate limit 中斷時會自動儲存進度，下次繼續
- 大 repo 可以分多次抓完（每小時 60 頁 = 6,000 星）

## 輸出範例

```
Cache: 12345 stars (last updated 2025-03-15 14:30), resuming from page 124
Fetched 42 new stars (total: 12387)

Repository: owner/repo
Total stars: 12,387
Tracked period: 2023-01-15 ~ 2025-03-15 (113 weeks)

=== Growth Analysis ===
Avg stars/week (all time):    109.6
Linear regression slope:      122.3 stars/week (R²=0.97)
Avg stars/week (last 4 wks):  85.2
Linear regression (last 4):   90.1 stars/week (R²=0.99)

=== Weekly Breakdown (last 10 weeks) ===
2025-W06:  +92
2025-W07:  +88
2025-W08:  +76
...
```

## 指標說明

| 指標 | 說明 |
|------|------|
| Avg stars/week | 總星數 ÷ 總週數 |
| Linear regression slope | 最小二乘法回歸斜率，代表每週累積星數的成長速度 |
| R² | 回歸擬合度，越接近 1 代表成長越線性穩定 |
| last 4 wks | 最近 4 週的同上指標，用來看近期趨勢 |
