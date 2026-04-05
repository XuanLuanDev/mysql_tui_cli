# MySQL Remote TUI CLI

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![MySQL](https://img.shields.io/badge/mysql-4479A1.svg?style=for-the-badge&logo=mysql&logoColor=white)

A lightweight and powerful Command-Line Interface (CLI) tool for interacting seamlessly with MySQL databases, written entirely in **Rust**.
> *Một công cụ Command-Line Interface (CLI) nhẹ nhàng nhưng cực kỳ mạnh mẽ để tương tác trực tiếp với cơ sở dữ liệu MySQL, được viết bằng ngôn ngữ **Rust**.*

The application features an interactive Terminal User Interface (TUI) that brings syntax highlighting, dynamic keyword auto-completion, multi-line query editing, and instant tabular results straight to your console.
> *Ứng dụng cung cấp bảng kết nối chuyên nghiệp, môi trường viết mã Console/Terminal quen thuộc nhưng tích hợp tự động nhận diện cú pháp (Syntax Highlight) và gợi ý chữ (Auto-Suggest), kèm theo hệ thống vẽ bảng dữ liệu ngay trong màn hình TUI.*


---

## ⚡ Features | Tính năng nổi bật

- **Intuitive TUI Context:** Connect easily using a structured, multi-field terminal login form—designed just like professional database tools.
> *- **Giao diện trực quan (TUI):** Form đăng nhập trực quan phân chia các ô nhập Host, Port, Username... trực tiếp trên terminal.*

- **Smart SQL Editor:** A multi-line query editor that features regex syntax highlighting and a floating popup that auto-suggests SQL keywords (`Tab` to complete) as you type.
> *- **Smart SQL Editor:** Trình biên tập đa dòng tiện lợi. Tốc độ gõ được tối ưu hóa với tính năng tự đánh dấu màu (Syntax Highlight) và Gợi ý hoàn thành chữ (Auto-complete popup) siêu việt.*

- **Blazing Fast:** Instant startup and query execution backed by Rust's speed and memory safety.
> *- **Siêu tốc độ:** Khởi chạy ngay lập tức nhờ vào khả năng kiểm soát bộ nhớ tối ưu của Rust.*

- **Cross-Platform:** Fully compatible with Windows, macOS, and Linux out of the box without complex dependencies.
> *- **Đa nền tảng:** Có thể chạy tệp thực thi ngay trên Windows, macOS, Linux mà không phải tải theo các tệp rườm rà.*

---

## 🚀 Installation | Cài đặt phần mềm

### 1. Pre-compiled Binary | File chạy sẵn (Khuyên dùng)
1. Head over to the [Releases](../../releases) section on GitHub. *(Tới tab Releases trên GitHub)*
2. Download the compressed executable matching your OS: *(Tải bộ cài thuộc hệ điều hành tương ứng)*
   - **Windows:** `mysql_tui_cli-windows-amd64.exe`
   - **Linux:** `mysql_tui_cli-linux-amd64`
   - **macOS:** `mysql_tui_cli-macos-amd64`
3. Optionally add it to your system's `$PATH` for global terminal execution. *(Bạn có thể copy file vào thư viện chung để gọi câu lệnh tại bất cứ đâu!)*

### 2. Build from Source | Khởi chạy từ mã nguồn
**Requirements (Cần chuẩn bị):** [Rust & Cargo](https://rustup.rs/) (v1.70.0+)

```bash
git clone https://github.com/your-username/mysql_tui_cli.git
cd mysql_tui_cli

# Run directly in development mode 
# (Khởi động ứng dụng ngay lập tức qua máy chủ phát triển)
cargo run

# Compile an optimized standard release binary 
# (Biên dịch ra file tốc độ chuẩn)
cargo build --release
```

---

## 📖 Usage Guide | Hướng dẫn kết nối
1. Launch the app to view the **Connection Form**. *(Mở ứng dụng sẽ thấy Màn hình Chào Mừng và Form Đăng Nhập)*.
2. Supply your Host, Port, Username, Password, and Database Name. Use the strictly-typed form controls (`Tab` to navigate, `Enter` to connect). *(Điền vào đơn điền thông tin, dùng phím `Tab` để nhảy dòng, nhấn `Enter` ở nút Connect).*
3. Inside the **Main Editor**, you can type query commands (e.g., `SELECT * FROM users;`). The application will highlight your text and drop down auto-complete hints. *(Sau khi đăng nhập, gõ câu SQL cần tìm kiếm, và sử dụng gợi ý xuất hiện quanh chuột).*
4. Press `F5` to execute queries. The results table will be painted at the bottom alongside any caught errors. *(Cuối cùng ấn `F5` để yêu cầu cơ sở dữ liệu trả kết quả. Bảng dữ liệu sẽ vẽ dưới màn hình).*
5. Hit `Esc` or `Ctrl+C` inside the general handler to quit the app safely. *(Phím `Esc` được dùng để thoát ứng dụng).*

---

## ⌨️ Shortcuts Master Guide | Cẩm nang Phím Tắt

| Phím (Key) | Mô tả (Description) | Khu vực (Context) |
| :--- | :--- | :--- |
| `Tab` / `Shift+Tab` | Jump between input fields / Chuyển qua lại giữa các ô nhập liệu | Connection Login |
| `F5` | Execute SQL Query / Xuất lệnh truy vấn dữ liệu | Main Editor |
| `Esc` / `Ctrl+C` | Cancel & Exit application / Thoát ứng dụng | All |
| `Up` / `Down` | Move text cursor freely / Di chuyển con trỏ chữ tự do | Main Editor |
| `Up` / `Down` | Navigate Autocomplete popup / Cuộn danh sách chữ gợi ý | Auto-Complete Popup |
| `Tab` | Confirm Autocomplete selection / Chốt từ khóa đang gợi ý | Auto-Complete Popup |
| `Ctrl + Up` / `Down` | Scroll the Results Data Table / Cuộn bảng dữ liệu bên dưới | Results Table |
| `Alt + Up` / `Down` | Scroll Database/Table Sidebar / Cuộn danh sách ở Cột trái | Sidebar Explorer |
| `Alt + Enter` | Select Database and fetch Tables / Chọn Database để sạc mảng Table | Databases Explorer |
| `Alt + Backspace` | Go back to Database selection / Trở lại danh sách Database cũ | Tables Explorer |

---

## 🛠️ Built With | Nền tảng Công Nghệ
- [Rust](https://www.lang.org/) - Fast, safe systems programming language / *Ngôn ngữ ưu tiên quản lý an toàn hệ thống siêu tốc*.
- [Ratatui](https://ratatui.rs/) - Premier library for terminal user interfaces / *Bộ engine dựng layout Console số liệu cho Rust*.
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation / *Thư viện tiếp nhận phím bấm độc lập hệ điều hành*.
- [MySQL Crate](https://crates.io/crates/mysql) - Pure rust MySQL driver / *Trình quản lý cơ sở dữ liệu kết nối thẳng của hệ thống Rust*.

---

## 🤝 Contribution | Đóng góp
Bug reports and pull requests are welcome on GitHub. Mọi phản hồi, khiếu nại (Issues) hay đóng góp tính năng mới (Pull Requests) luôn được chào đón tại kho lưu trữ này!

## 📜 License | Giấy phép
This project is licensed under the [MIT License](LICENSE). Dự án được triển khai chung dưới giấy phép thương mại mở MIT.
