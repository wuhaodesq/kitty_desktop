# kitty_desktop

`kitty_desktop` 是对 `kitty` 终端内核的桌面封装层项目，以上游仓库为基础：

- 上游内核仓库：<https://github.com/wuhaodesq/kitty.git>
- 技术栈：**Rust（与 kitty 技术方向保持一致）**
- 项目目标：不改写终端内核，专注桌面化能力（启动器、配置可视化、会话管理、集成入口）

## 当前实现进度

已完成 **Phase 1 核心服务最小化实现**：

- `core-adapter`：`kitty` 可执行文件探测、版本读取、启动参数拼装、启动 dry-run
- `config-service`：桌面配置 JSON 落盘与读取
- `session-service`：会话模板 JSON 落盘、更新、查询与列表
- CLI 子命令：`version` / `launch` / `config show|set` / `session list|save` / `shell run` / `doctor`

## 目录结构（当前）

```text
kitty_desktop/
├─ src/
│  ├─ lib.rs               # core-adapter + 公共导出
│  ├─ config_service.rs    # 配置服务
│  ├─ session_service.rs   # 会话服务
│  ├─ desktop_shell.rs     # 最小桌面壳层编排（启动/设置/退出）
│  └─ main.rs              # CLI 入口
├─ tests/
│  └─ core_adapter_cli.rs
├─ Cargo.toml
└─ README.md
```

## 快速开始

### 1) 检查 kitty 版本

```bash
cargo run -- version
```

### 2) 预览启动命令（不真正启动）

```bash
cargo run -- launch --directory . --title kitty_desktop --dry-run
```

### 3) 保存桌面默认配置

```bash
cargo run -- config set --directory /work --title dev --shell /bin/zsh
```

### 4) 查看配置

```bash
cargo run -- config show
```

### 5) 保存会话模板

```bash
cargo run -- session save --name dev --directory /work --title Dev -- --single-instance
```

### 6) 列出会话模板

```bash
cargo run -- session list
```


### 7) 进入最小 desktop shell 交互

```bash
cargo run -- shell run
```

支持命令：`settings` / `launch [session_name]` / `exit`。


### 8) 运行环境诊断

```bash
cargo run -- doctor
```

用于检查 kitty 可执行文件、配置加载、会话读取等关键健康项。

## 实现计划（基于上游 kitty）

### Phase 0：上游对齐与技术预研

- 拉取并固定上游基线版本（`wuhaodesq/kitty` commit/tag）
- 明确 kitty 可执行文件探测规则（PATH、安装位置、版本校验）
- 整理首期需要暴露的 kitty 配置项白名单

### Phase 1：MVP 可运行版本

- [x] Rust `core-adapter` 基础 PoC
- [x] 最小 `config-service`（配置表单落盘）
- [x] 最小 `session-service`（保存与恢复会话）
- [x] 最小 `desktop-shell`（启动 / 设置 / 退出）

### Phase 2：可用性增强

- 主题实时预览
- 快捷键冲突检测
- 启动失败诊断（路径、权限、参数错误）
- 增加常用模板（开发/运维/远程）
