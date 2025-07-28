```markdown
# rust_excel_reader

`rust_excel_reader` 是一個用於讀取和顯示 CSV 或 Excel 文件內容的桌面應用程式，基於 Rust 語言開發。該應用程式使用了現代的 GUI 框架 `eframe` 和 `egui`，並結合了異步編程技術，以提供流暢的用戶體驗。

## 特性

- **文件選擇**：用戶可以通過文件對話框輕鬆選擇要讀取的 CSV 或 Excel 文件。
- **異步加載數據**：選擇文件後，應用會異步加載數據，確保界面不會因為數據加載而卡頓。
- **懶加載**：當用戶滾動到接近底部時，自動加載更多行數據，提升使用體驗。
- **錯誤處理**：在數據加載過程中，如果出現錯誤，應用會在界面上顯示相應的錯誤信息，方便用戶排查問題。

## 專案結構

### Cargo.toml

`Cargo.toml` 文件定義了專案的基本信息，包括：

- **專案名稱**：`rust_excel_reader`
- **版本**：`0.1.0`
- **Rust 版本**：使用 2021 版的 Rust 編輯
- **依賴庫**：
  - GUI 框架：`egui` 和 `eframe`
  - 文件處理：`rfd` 和 `calamine`
  - 日期時間處理：`chrono`
  - CSV 讀寫：`csv` 和 `csv-async`
  - 編碼處理：`encoding_rs` 和 `encoding_rs_io`
  - 異步運行時：`tokio` 和 `tokio-util`
  - 單例模式：`once_cell`

### Cargo.lock

`Cargo.lock` 文件是 Rust 專案的依賴管理文件，列出了專案所需的所有包及其版本、來源和依賴關係。這些包涵蓋了圖形處理、異步編程、數據序列化、網絡通信等多個功能，顯示出該專案的複雜性和多樣性。

### 源碼結構

專案採用模組化設計，提高代碼的可維護性和可讀性：

```
src/
├── main.rs           # 應用程式入口點
├── app.rs            # 主應用邏輯和用戶界面
├── data_loader.rs    # 數據加載協調器
├── csv_loader.rs     # CSV 文件處理
├── excel_loader.rs   # Excel 文件處理
└── font_setup.rs     # 字體配置（支援中文字符）
```

#### 核心模組說明

- **main.rs**: 應用程式的入口點，負責初始化和啟動
- **app.rs**: 包含主要的應用邏輯、狀態管理和 UI 渲染
- **data_loader.rs**: 協調異步數據加載操作
- **csv_loader.rs**: 專門處理 CSV 文件的讀取和解析
- **excel_loader.rs**: 專門處理 Excel 文件的讀取和多工作表支援
- **font_setup.rs**: 配置字體以支援中文和其他 Unicode 字符

## 安裝與運行

1. 確保已安裝 Rust 環境，可以參考 [Rust 官方網站](https://www.rust-lang.org/) 進行安裝。
2. 克隆此專案：
   ```bash
   git clone https://github.com/your_username/rust_excel_reader.git
   cd rust_excel_reader
   ```
3. 使用 Cargo 編譯並運行應用：
   ```bash
   cargo run
   ```

## 貢獻

歡迎任何形式的貢獻！如果您有建議或發現問題，請提交問題或拉取請求。

## 授權

本專案採用 MIT 授權，詳情請參見 [LICENSE](LICENSE) 文件。

---

感謝您使用 `rust_excel_reader`！希望這個應用能夠幫助您輕鬆地讀取和處理 CSV 和 Excel 文件。
```