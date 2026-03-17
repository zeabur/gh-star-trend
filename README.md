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
| 每次可抓星星數 | ~6,000 | 無上限（GraphQL 補完） |
| 適合 | 小型 repo (<6k 星) | 任何大小的 repo |

#### 取得 Token

1. 到 [github.com/settings/tokens](https://github.com/settings/tokens)
2. 點 **Generate new token (classic)**
3. 不需要勾選任何 scope（只讀 public repo 不需要權限）
4. 點 **Generate token**，複製產生的 `ghp_xxxx...`

#### 設定 Token

```bash
# 方式一：.env 檔（推薦，放在執行目錄下）
echo "GITHUB_TOKEN=ghp_xxxxxxxxxxxx" > .env

# 方式二：環境變數
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx

# 方式三：參數
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

## 已知問題

- **REST API 40k 分頁限制**：GitHub REST API 最多只能翻到第 400 頁（40,000 星）。超過此限制時會自動切換到 GraphQL API 補完剩餘的星星，但 GraphQL 需要 token 才能使用。
- **大型 repo 首次抓取耗時**：超過 10 萬星的 repo 首次抓取可能需要數十分鐘（例如 300k 星約需 3,000+ 次 API 請求）。後續執行會使用快取，只抓取新增的部分。
- **進度顯示不準確**：REST API 階段的 total pages 有時顯示不正確（例如顯示 `page 200/100`），因為 GitHub 的 `Link` header 只在第一頁回傳總頁數估計值。不影響實際抓取。
- **GraphQL rate limit 中斷時不保存進度**：如果 GraphQL 抓取途中被 rate limit，目前不會保存已抓到的部分，需要下次重新從 REST cache 的位置開始補完。
- **取消星星（unstar）不會從快取中移除**：快取只會增加星星，不會偵測已被移除的星星。如需重置，刪除 `~/.gh-star-trend/` 下對應的 JSON 檔案。

## License

[MIT](LICENSE)
