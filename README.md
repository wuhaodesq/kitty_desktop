# kitty_desktop

`kitty_desktop` 是对 `kitty` 终端内核的桌面封装层项目，以上游仓库为基础：

- 上游内核仓库：<https://github.com/wuhaodesq/kitty.git>
- 技术栈：**Rust（与 kitty 技术方向保持一致）**
- 项目目标：不改写终端内核，专注桌面化能力（启动器、配置可视化、会话管理、集成入口）

## 当前实现进度

已从第一步重新开始，落地 **Rust 版 core-adapter PoC**：

- `kitty` 可执行文件探测（PATH）
- 版本读取（`kitty --version`）
- 启动参数拼装（工作目录、title、config、session、shell、extra args）
- Rust CLI 入口（支持 `version` / `launch --dry-run`）

## 目录结构（当前）

```text
kitty_desktop/
├─ src/
│  ├─ lib.rs               # core-adapter：探测、版本读取、启动命令拼装
│  └─ main.rs              # CLI 入口（version/launch）
├─ tests/
│  └─ core_adapter_cli.rs
├─ Cargo.toml
└─ README.md
```

## 快速开始（Rust PoC）

### 1) 检查版本

```bash
cargo run -- version
```

### 2) 预览启动命令（不真正启动）

```bash
cargo run -- launch --directory . --title kitty_desktop --dry-run
```

### 3) 带额外参数预览

```bash
cargo run -- launch --dry-run -- --single-instance
```

## 实现计划（基于上游 kitty）

### Phase 0：上游对齐与技术预研

- 拉取并固定上游基线版本（`wuhaodesq/kitty` commit/tag）
- 明确 kitty 可执行文件探测规则（PATH、安装位置、版本校验）
- 整理首期需要暴露的 kitty 配置项白名单

### Phase 1：MVP 可运行版本

- [x] Rust `core-adapter` 基础 PoC
- [ ] 最小 `desktop-shell`（启动 / 设置 / 退出）
- [ ] 最小 `config-service`（配置表单落盘）
- [ ] 最小 `session-service`（保存与恢复会话）
